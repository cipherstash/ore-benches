//! EQL v3 twin of `benches/exact.rs` — equality lookups against
//! `string_encrypted_v3_<N>` (column typed `eql_v3.text_search`).
//!
//! Probe flow: cipherstash-client 0.38 cannot emit a v3 scalar QUERY
//! payload (`from_v2_query` fails closed with UnsupportedQueryTarget), so
//! the probe value is encrypted as a STORAGE payload and converted with
//! `from_v2` (target `text_search`). The stored-shape probe carries the
//! same `hm` term a query payload would, and the SQL compares via the
//! `eql_v3.eq_term` extractor (`eql_hash`) or the inlinable `=` operator
//! (`eql_cast`), both of which reduce to
//! `eql_v3.eq_term(value) = eql_v3.eq_term($1)` and engage the
//! `hash (eql_v3.eq_term(value))` index.
//!
//! v3 columns are jsonb domains (no composite wrapper), so scenarios
//! project `value::jsonb` — safe here because no v3 scenario puts the raw
//! column in an ORDER BY (the v2 projection-pushdown sort-key trap needs
//! an `ORDER BY value` to bite).

use cipherstash_client::{
    encryption::ScopedCipher,
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
    AutoStrategy,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{
    bench_assert, extract_indexes_used, init_scoped_cipher,
    v3::{sample_plaintext_string_v3, EncryptedQueryBuilderV3, EncryptedQueryV3, TargetDomain},
    write_metadata_file, ScenarioMetadata,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::runtime::Runtime;

// (sql_template, scenario_name). Scenario names mirror the v2 exact bench
// so the reporters line the two versions up side by side.
static QUERY_TEMPLATES: &[(&str, &str)] = &[
    (
        "SELECT id, value::jsonb FROM {TABLE} WHERE value = $1::eql_v3.text_search LIMIT 1",
        "eql_cast",
    ),
    (
        "SELECT id, value::jsonb FROM {TABLE} \
         WHERE eql_v3.eq_term(value) = eql_v3.eq_term($1::eql_v3.text_search) LIMIT 1",
        "eql_hash",
    ),
];

async fn build_query(
    cipher: Arc<ScopedCipher<AutoStrategy>>,
    query: &str,
    x: &str,
    table_name: &str,
) -> EncryptedQueryV3 {
    // Same column config as encrypt_string_v3 — the probe must carry every
    // term text_search requires (hm + bf + ob) to pass the conversion.
    let column_config = ColumnConfig::build("value")
        .casts_as(ColumnType::Text)
        .add_index(Index::new_unique())
        .add_index(Index::new_match())
        .add_index(Index::new_ore());

    let identifier = Identifier::new(table_name, "value");
    let target = TargetDomain::parse("text_search").expect("text_search is a v3 domain");

    EncryptedQueryBuilderV3::new(column_config, identifier, target)
        .statement(query)
        .build_query(x.to_string(), cipher)
        .await
        .expect("Failed to build encrypted v3 query")
}

fn criterion_benchmark(c: &mut Criterion) {
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

    // Derive the search term from a real row so the bench measures actual
    // equality-query cost — same rationale as the v2 exact bench.
    let search_term: String = rt.block_on(async {
        sample_plaintext_string_v3(&pool, Arc::clone(&cipher), &table_name)
            .await
            .expect("failed to sample plaintext from table — is it populated?")
    });
    eprintln!(
        "exact_v3 bench: using sampled search term `{}` from `{}`",
        &search_term, &table_name
    );

    let queries = rt.block_on(async {
        let mut queries = Vec::with_capacity(QUERY_TEMPLATES.len());
        for (query_template, _) in QUERY_TEMPLATES {
            let query_str = query_template.replace("{TABLE}", &table_name);
            let query =
                build_query(Arc::clone(&cipher), &query_str, &search_term, &table_name).await;
            queries.push(query);
        }
        queries
    });

    let metadata = rt.block_on(async {
        let mut out = Vec::with_capacity(queries.len());
        for (i, query) in queries.iter().enumerate() {
            let (_, scenario) = QUERY_TEMPLATES[i];
            let bench_id = format!("EXACT_V3/exact/{}/{}", scenario, target_rows);
            let explain = query.explain(&pool).await.expect("EXPLAIN failed");
            let indexes_used = extract_indexes_used(&explain);
            let parameters = vec![query.parameter_json()];
            let rows = query
                .execute(&pool)
                .await
                .expect("execute for row-count failed");
            let rows_returned = rows.len() as u64;
            out.push(ScenarioMetadata {
                id: bench_id,
                query: query.statement.clone(),
                parameters,
                explain,
                indexes_used,
                rows_returned,
                version: 3,
            });
        }
        out
    });
    write_metadata_file("exact_v3", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("EXACT_V3");
    group.sample_size(10);

    for (i, query) in queries.into_iter().enumerate() {
        let (_, scenario) = QUERY_TEMPLATES[i];
        let exec_id = format!("EXACT_V3/exact/{}/{}", scenario, target_rows);
        let decrypt_id = format!("EXACT_V3/exact_decrypt/{}/{}", scenario, target_rows);

        let exec_id_inner = exec_id.clone();
        group.bench_function(format!("exact/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _: Vec<_> = bench_assert(query.execute(&pool).await, &exec_id_inner);
            })
        });

        let decrypt_id_inner = decrypt_id.clone();
        group.bench_function(format!("exact_decrypt/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                // Decryption rebuilds the v2 `ct` envelope per row (see
                // dbbenches::v3::v3_scalar_to_ciphertext) — same client-side
                // decrypt work as v2 plus the envelope rebuild.
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
