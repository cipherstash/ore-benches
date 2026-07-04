//! EQL v3 sibling of `benches/combo.rs` — composite-predicate scenarios over
//! `combo_encrypted_v3_<N>` (`name` text_search, `age` integer_ord,
//! `category` text_eq).
//!
//! v2's `name LIKE $1` becomes `name @> $1` (v3 has no LIKE — identical
//! bloom-containment semantics); the extractor references move to the v3
//! functions (`eql_v3.ord_term(age)`, `eql_v3.eq_term(category)`). Scenario
//! ids are unchanged.

use cipherstash_client::{
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{
    bench_assert, init_scoped_cipher, init_tracing,
    v3::{encrypt_stored_v3, V3EncryptedQuery},
    write_metadata_file_in, ScenarioMetadata,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use std::sync::Arc;
use tokio::runtime::Runtime;

static QUERY_TEMPLATES: &[(&str, &str)] = &[
    (
        "SELECT id FROM {TABLE} \
         WHERE name @> $1 \
         ORDER BY eql_v3.ord_term(age) LIMIT 10",
        "bloom_ore_order_limit",
    ),
    (
        "SELECT eql_v3.eq_term(category), count(*) FROM {TABLE} \
         WHERE name @> $1 \
         GROUP BY 1",
        "filtered_group_by",
    ),
    (
        "SELECT eql_v3.eq_term(category), count(*) FROM {TABLE} \
         WHERE name @> $1 \
         GROUP BY 1 ORDER BY count(*) DESC LIMIT 10",
        "top_n_filtered_group_by",
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
    let table_name = format!("combo_encrypted_v3{}", table_suffix);

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

    // Same pattern as the v2 bench (and match_v3). The needle payload
    // carries hm+ob+bf of "Bob"; only the bloom term participates in `@>`.
    let pattern = "Bob";
    let name_config = ColumnConfig::build("name")
        .casts_as(ColumnType::Text)
        .add_index(Index::new_unique())
        .add_index(Index::new_match())
        .add_index(Index::new_ore());
    let name_ident = Identifier::new(&table_name, "name");

    let queries: Vec<V3EncryptedQuery> = rt.block_on(async {
        let mut queries = Vec::with_capacity(QUERY_TEMPLATES.len());
        for (query_template, _) in QUERY_TEMPLATES {
            let query_str = query_template.replace("{TABLE}", &table_name);
            let param = encrypt_stored_v3(
                Arc::clone(&cipher),
                &name_config,
                &name_ident,
                pattern.to_string(),
                "text_search",
            )
            .await
            .expect("failed to encrypt+convert name pattern");
            queries.push(V3EncryptedQuery::new(param, query_str, Arc::clone(&cipher)));
        }
        queries
    });

    // Combo result shapes don't match V3EncryptedQuery::execute's typed
    // tuple, so metadata row counts and the iter loop use raw sqlx.
    let metadata = rt.block_on(async {
        let mut out = Vec::with_capacity(queries.len());
        for (i, query) in queries.iter().enumerate() {
            let (_, scenario) = QUERY_TEMPLATES[i];
            let bench_id = format!("COMBO/combo/{}/{}", scenario, target_rows);
            let explain = query.explain(&pool).await.expect("EXPLAIN failed");
            let indexes_used = dbbenches::extract_indexes_used(&explain);
            let rows = sqlx::query(&query.statement)
                .bind(Json(&query.param))
                .fetch_all(&pool)
                .await
                .expect("execute for row-count failed");
            out.push(ScenarioMetadata {
                id: bench_id,
                query: query.statement.clone(),
                parameters: vec![query.parameter_json().expect("serialise parameter")],
                explain,
                indexes_used,
                rows_returned: rows.len() as u64,
            });
        }
        out
    });
    write_metadata_file_in("results/query/v3", "combo", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("COMBO");
    group.sample_size(10);

    for (i, query) in queries.into_iter().enumerate() {
        let (_, scenario) = QUERY_TEMPLATES[i];
        let exec_id = format!("COMBO/combo/{}/{}", scenario, target_rows);

        let exec_id_inner = exec_id.clone();
        group.bench_function(format!("combo/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let rows = bench_assert(
                    sqlx::query(&query.statement)
                        .bind(Json(&query.param))
                        .fetch_all(&pool)
                        .await,
                    &exec_id_inner,
                );
                black_box(rows.len());
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
