//! EQL v3 sibling of `benches/ore.rs` — range queries against
//! `integer_encrypted_v3_<N>` (`eql_v3.integer_ord`, ORE block term).
//!
//! Scenario ids match the v2 bench exactly. Bound parameters are
//! STORED-shape v3 payloads (see benches/exact_v3.rs — no scalar query wire
//! shape exists in v3); the domain's comparison operators carry
//! `RIGHTARG = jsonb` overloads that cast in-plan.
//!
//! The selective scenarios stay DISABLED, mirroring the v2 bench: the
//! selectivity limitation (planner can't estimate a bound-parameter
//! encrypted comparison, falls back to DEFAULT_INEQ_SEL) is a property of
//! functional-extractor predicates generally — v3's `=`-only RESTRICT
//! hints don't change range estimation. Re-check via the metadata sidecar
//! if a v3 selectivity fix lands (EQL issue #230).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{
    bench_assert,
    v3::{encrypt_stored_v3, V3EncryptedQuery},
    init_scoped_cipher, init_tracing, write_metadata_file_in, ScenarioMetadata,
};
use cipherstash_client::{
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::runtime::Runtime;

static QUERY_TEMPLATES: &[(&str, i32, &str)] = &[
    // ── Non-selective baselines (≈50% selectivity → Seq Scan + LIMIT) ──
    (
        "SELECT id, value FROM {TABLE} WHERE value > $1 LIMIT 10",
        5000,
        "range_gt_10",
    ),
    (
        "SELECT id, value FROM {TABLE} WHERE value > $1 LIMIT 100",
        5000,
        "range_gt_100",
    ),
    (
        "SELECT id, value FROM {TABLE} WHERE value < $1 LIMIT 10",
        5000,
        "range_lt_10",
    ),
    (
        "SELECT id, value FROM {TABLE} WHERE value < $1 LIMIT 100",
        5000,
        "range_lt_100",
    ),
    // ── Ordered range (extractor in ORDER BY, matches the functional
    //    ord_term index expression → Index Scan, no Sort node) ──
    (
        "SELECT id, value FROM {TABLE} \
         WHERE value < $1 \
         ORDER BY eql_v3.ord_term(value) LIMIT 10",
        5000,
        "range_lt_ordered_10",
    ),
];

fn criterion_benchmark(c: &mut Criterion) {
    init_tracing();
    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS").unwrap_or_else(|_| "unknown".to_string());
    let table_suffix = match target_rows.as_str() {
        "10000" | "100000" | "1000000" | "10000000" => format!("_{}", target_rows),
        _ => String::new(),
    };
    let table_name = format!("integer_encrypted_v3{}", table_suffix);

    let (pool, cipher) = rt.block_on(async {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");
        let cipher = init_scoped_cipher()
            .await
            .expect("Failed to initialize ScopedCipher");
        (pool, cipher)
    });

    let column_config = ColumnConfig::build("value")
        .casts_as(ColumnType::Int)
        .add_index(Index::new_ore());
    let identifier = Identifier::new(&table_name, "value");

    let queries: Vec<V3EncryptedQuery> = rt.block_on(async {
        let mut queries = Vec::with_capacity(QUERY_TEMPLATES.len());
        for (query_template, x, _) in QUERY_TEMPLATES {
            let query_str = query_template.replace("{TABLE}", &table_name);
            let param = encrypt_stored_v3(
                Arc::clone(&cipher),
                &column_config,
                &identifier,
                *x,
                "integer_ord",
            )
            .await
            .expect("failed to encrypt+convert query threshold");
            queries.push(V3EncryptedQuery::new(param, query_str, Arc::clone(&cipher)));
        }
        queries
    });

    let metadata = rt.block_on(async {
        let mut out = Vec::with_capacity(queries.len());
        for (i, query) in queries.iter().enumerate() {
            let (_, _, scenario) = QUERY_TEMPLATES[i];
            let bench_id = format!("ORE/ore/{}/{}", scenario, target_rows);
            let (explain, indexes_used, rows_returned) = query
                .capture_metadata(&pool)
                .await
                .expect("metadata capture failed");
            out.push(ScenarioMetadata {
                id: bench_id,
                query: query.statement.clone(),
                parameters: vec![query.parameter_json().expect("serialise parameter")],
                explain,
                indexes_used,
                rows_returned,
            });
        }
        out
    });
    write_metadata_file_in("results/query/v3", "ore", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("ORE");
    group.sample_size(10);

    for (i, query) in queries.into_iter().enumerate() {
        let (_, _, scenario) = QUERY_TEMPLATES[i];
        let exec_id = format!("ORE/ore/{}/{}", scenario, target_rows);
        let decrypt_id = format!("ORE/ore_decrypt/{}/{}", scenario, target_rows);

        let exec_id_inner = exec_id.clone();
        group.bench_function(format!("ore/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _: Vec<_> = bench_assert(query.execute(&pool).await, &exec_id_inner);
            })
        });

        let decrypt_id_inner = decrypt_id.clone();
        group.bench_function(format!("ore_decrypt/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _r: Vec<i32> = black_box(bench_assert(
                    query.execute_and_decrypt(&pool).await,
                    &decrypt_id_inner,
                ));
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
