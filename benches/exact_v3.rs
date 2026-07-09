//! EQL v3 sibling of `benches/exact.rs` — equality lookups against
//! `string_encrypted_v3_<N>` (`public.text_search`).
//!
//! Scenario ids are kept IDENTICAL to the v2 bench (`eql_cast`, `eql_hash`)
//! so the comparison report joins v2 vs v3 by (group, scenario, tier); the
//! version dimension is the results directory (`results/query/v3/`).
//!
//! v3 differences worth knowing when reading numbers:
//!   * The bound parameter is a STORED-shape v3 payload (encrypt Store +
//!     from_v2) — no v3 scalar query wire shape exists. The `=` operator's
//!     `(text_search, jsonb)` overload casts it in-plan.
//!   * `eql_hash` keeps its id but the index is a functional BTREE on
//!     `eql_v3.eq_term(value)` (v2 used hash — see the index DDL comments).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{
    bench_assert,
    v3::{encrypt_stored_v3, sample_plaintext_string_v3, V3EncryptedQuery},
    init_scoped_cipher, init_tracing, write_metadata_file_in, ScenarioMetadata,
};
use cipherstash_client::{
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::runtime::Runtime;

static QUERY_TEMPLATES: &[(&str, &str)] = &[
    (
        "SELECT id, value FROM {TABLE} WHERE value = $1 LIMIT 1",
        "eql_cast",
    ),
    (
        "SELECT id, value FROM {TABLE} WHERE eql_v3.eq_term(value) = eql_v3.eq_term($1::public.text_search) LIMIT 1",
        "eql_hash",
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
    let table_name = format!("string_encrypted_v3{}", table_suffix);

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

    // Sample a real row so the equality scenarios measure index hit + 1-row
    // return, not a 0-rows early exit (same rationale as the v2 bench).
    let search_term: String = rt.block_on(async {
        sample_plaintext_string_v3(&pool, Arc::clone(&cipher), &table_name)
            .await
            .expect("failed to sample plaintext from table — is it populated?")
    });
    eprintln!(
        "exact_v3 bench: using sampled search term `{}` from `{}`",
        &search_term, &table_name
    );

    // The needle config must match the ingest config (unique+match+ore) so
    // the converted payload satisfies text_search's required terms.
    let column_config = ColumnConfig::build("value")
        .casts_as(ColumnType::Text)
        .add_index(Index::new_unique())
        .add_index(Index::new_match())
        .add_index(Index::new_ore());
    let identifier = Identifier::new(&table_name, "value");

    let queries: Vec<V3EncryptedQuery> = rt.block_on(async {
        let mut queries = Vec::with_capacity(QUERY_TEMPLATES.len());
        for (query_template, _) in QUERY_TEMPLATES {
            let query_str = query_template.replace("{TABLE}", &table_name);
            let param = encrypt_stored_v3(
                Arc::clone(&cipher),
                &column_config,
                &identifier,
                search_term.clone(),
                "text_search",
            )
            .await
            .expect("failed to encrypt+convert query needle");
            queries.push(V3EncryptedQuery::new(param, query_str, Arc::clone(&cipher)));
        }
        queries
    });

    let metadata = rt.block_on(async {
        let mut out = Vec::with_capacity(queries.len());
        for (i, query) in queries.iter().enumerate() {
            let (_, scenario) = QUERY_TEMPLATES[i];
            let bench_id = format!("EXACT/exact/{}/{}", scenario, target_rows);
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
    write_metadata_file_in("results/query/v3", "exact", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("EXACT");
    group.sample_size(10);

    for (i, query) in queries.into_iter().enumerate() {
        let (_, scenario) = QUERY_TEMPLATES[i];
        let exec_id = format!("EXACT/exact/{}/{}", scenario, target_rows);
        let decrypt_id = format!("EXACT/exact_decrypt/{}/{}", scenario, target_rows);

        let exec_id_inner = exec_id.clone();
        group.bench_function(format!("exact/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _: Vec<_> = bench_assert(query.execute(&pool).await, &exec_id_inner);
            })
        });

        let decrypt_id_inner = decrypt_id.clone();
        group.bench_function(format!("exact_decrypt/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _r: Vec<String> = black_box(bench_assert(
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
