//! EQL v3 sibling of `benches/match.rs` — substring/pattern matching against
//! `string_encrypted_v3_<N>` (`eql_v3.text_search`, bloom term).
//!
//! v3 has NO `~~`/LIKE operator: bloom matching is exposed as containment.
//! v2's `value LIKE $1` scenarios keep their ids but run `value @> $1`
//! (identical semantics — needle bloom bits contained in the value's bloom;
//! the report's semantics-changed map flags the SQL delta).
//!
//! **The index question.** v3 ships no GIN opclass; the static index DDL
//! creates `GIN (eql_v3.match_term(value))`, which relies on the term's
//! base type (`smallint[]`) resolving native `array_ops`. Whether the
//! planner ENGAGES it for the inlined `@>` is exactly what this bench
//! answers — check `indexes_used` in the metadata sidecar. At the 10k tier
//! the bench additionally runs every scenario with the GIN dropped
//! (`*_noindex` ids) to quantify what the index is worth; the index is
//! recreated afterwards.

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
use std::sync::Arc;
use tokio::runtime::Runtime;

static QUERY_TEMPLATES: &[(&str, &str, &str)] = &[
    (
        "SELECT id, value FROM {TABLE} WHERE value @> $1 LIMIT 10",
        "Bob",
        "eql_cast_firstname",
    ),
    (
        "SELECT id, value FROM {TABLE} WHERE value @> $1 LIMIT 10",
        "Johnson",
        "eql_cast_lastname",
    ),
    (
        "SELECT id, value FROM {TABLE} WHERE eql_v3.match_term(value) @> eql_v3.match_term($1::eql_v3.text_search) LIMIT 10",
        "Johnson",
        "eql_bloom",
    ),
];

async fn capture(
    query: &V3EncryptedQuery,
    pool: &sqlx::PgPool,
    bench_id: String,
) -> ScenarioMetadata {
    let (explain, indexes_used, rows_returned) = query
        .capture_metadata(pool)
        .await
        .expect("metadata capture failed");
    ScenarioMetadata {
        id: bench_id,
        query: query.statement.clone(),
        parameters: vec![query.parameter_json().expect("serialise parameter")],
        explain,
        indexes_used,
        rows_returned,
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    init_tracing();
    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS").unwrap_or_else(|_| "unknown".to_string());
    let table_suffix = match target_rows.as_str() {
        "10000" | "100000" | "1000000" | "10000000" => format!("_{}", target_rows),
        _ => String::new(),
    };
    let table_name = format!("string_encrypted_v3{}", table_suffix);
    let gin_index = format!("{}_match_gin_index", table_name);

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

    // Needle config matches the ingest config so text_search's required
    // terms are all present on the converted payload.
    let column_config = ColumnConfig::build("value")
        .casts_as(ColumnType::Text)
        .add_index(Index::new_unique())
        .add_index(Index::new_match())
        .add_index(Index::new_ore());
    let identifier = Identifier::new(&table_name, "value");

    let queries: Vec<V3EncryptedQuery> = rt.block_on(async {
        let mut queries = Vec::with_capacity(QUERY_TEMPLATES.len());
        for (query_template, needle, _) in QUERY_TEMPLATES {
            let query_str = query_template.replace("{TABLE}", &table_name);
            let param = encrypt_stored_v3(
                Arc::clone(&cipher),
                &column_config,
                &identifier,
                needle.to_string(),
                "text_search",
            )
            .await
            .expect("failed to encrypt+convert match needle");
            queries.push(V3EncryptedQuery::new(param, query_str, Arc::clone(&cipher)));
        }
        queries
    });

    let mut metadata = rt.block_on(async {
        let mut out = Vec::with_capacity(queries.len());
        for (i, query) in queries.iter().enumerate() {
            let (_, _, scenario) = QUERY_TEMPLATES[i];
            out.push(
                capture(
                    query,
                    &pool,
                    format!("MATCH/match/{}/{}", scenario, target_rows),
                )
                .await,
            );
        }
        out
    });

    let mut group = c.benchmark_group("MATCH");
    group.sample_size(10);

    for (i, query) in queries.iter().enumerate() {
        let (_, _, scenario) = QUERY_TEMPLATES[i];
        let exec_id = format!("MATCH/match/{}/{}", scenario, target_rows);
        let decrypt_id = format!("MATCH/match_decrypt/{}/{}", scenario, target_rows);

        let exec_id_inner = exec_id.clone();
        group.bench_function(format!("match/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _: Vec<_> = bench_assert(query.execute(&pool).await, &exec_id_inner);
            })
        });

        let decrypt_id_inner = decrypt_id.clone();
        group.bench_function(format!("match_decrypt/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _r: Vec<String> = black_box(bench_assert(
                    query.execute_and_decrypt(&pool).await,
                    &decrypt_id_inner,
                ));
            })
        });
    }

    // ── No-index variants (10k tier only) ──
    // Quantifies what the GIN-via-array_ops index is worth (and, if the GIN
    // never engages, proves it by matching timings). criterion executes
    // bench_functions as they are registered, so the drop → bench →
    // recreate sequencing below is safe.
    if target_rows == "10000" {
        rt.block_on(async {
            sqlx::query(&format!("DROP INDEX IF EXISTS {}", gin_index))
                .execute(&pool)
                .await
                .expect("drop match GIN index");
            sqlx::query(&format!("ANALYZE {}", table_name))
                .execute(&pool)
                .await
                .expect("ANALYZE after index drop");
        });

        let noindex_metadata = rt.block_on(async {
            let mut out = Vec::with_capacity(queries.len());
            for (i, query) in queries.iter().enumerate() {
                let (_, _, scenario) = QUERY_TEMPLATES[i];
                out.push(
                    capture(
                        query,
                        &pool,
                        format!("MATCH/match/{}_noindex/{}", scenario, target_rows),
                    )
                    .await,
                );
            }
            out
        });
        metadata.extend(noindex_metadata);

        for (i, query) in queries.iter().enumerate() {
            let (_, _, scenario) = QUERY_TEMPLATES[i];
            let exec_id = format!("MATCH/match/{}_noindex/{}", scenario, target_rows);
            let exec_id_inner = exec_id.clone();
            group.bench_function(
                format!("match/{}_noindex/{}", scenario, target_rows),
                |b| {
                    b.to_async(&rt).iter(|| async {
                        let _: Vec<_> =
                            bench_assert(query.execute(&pool).await, &exec_id_inner);
                    })
                },
            );
        }

        rt.block_on(async {
            sqlx::query(&format!(
                "CREATE INDEX {} ON {} USING GIN (eql_v3.match_term(value))",
                gin_index, table_name
            ))
            .execute(&pool)
            .await
            .expect("recreate match GIN index");
            sqlx::query(&format!("ANALYZE {}", table_name))
                .execute(&pool)
                .await
                .expect("ANALYZE after index recreate");
        });
    }

    group.finish();

    write_metadata_file_in("results/query/v3", "match", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
