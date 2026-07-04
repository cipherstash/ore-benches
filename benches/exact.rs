use cipherstash_client::{
    encryption::ScopedCipher,
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
    AutoStrategy,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{
    bench_assert, extract_indexes_used, init_scoped_cipher, sample_plaintext_string,
    write_metadata_file, EncryptedQuery, EncryptedQueryBuilder, ScenarioMetadata,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::runtime::Runtime;

// (sql_template, scenario_name). The search term is derived at runtime from
// a sample decrypt of the target table (see `sample_plaintext_string`) — the
// previous hardcoded "Bob Johnson" matched zero rows at every tier because
// `fake::Name<EN>` doesn't generate that exact combination.
// `EncryptedQuery::execute` decodes the encrypted column via the custom
// `dbbenches::EqlV2Encrypted` sqlx type, so the bench projects `value`
// directly. The earlier `value::jsonb` shape worked too but was a footgun
// in the presence of an `ORDER BY value` (projection-pushdown folds the
// cast into the sort key, killing index-for-sort); SELECTing `value` raw
// keeps the bench scenarios free of that interaction. See
// `docs/reference/query-performance.md` §4 in the EQL repo.
static QUERY_TEMPLATES: &[(&str, &str)] = &[
    (
        "SELECT id, value FROM {TABLE} WHERE value = $1 LIMIT 1",
        "eql_cast",
    ),
    (
        "SELECT id, value FROM {TABLE} WHERE eql_v2.hmac_256(value) = eql_v2.hmac_256($1::jsonb) LIMIT 1",
        "eql_hash",
    ),
];

async fn build_query(
    cipher: Arc<ScopedCipher<AutoStrategy>>,
    query: &str,
    x: &str,
    table_name: &str,
) -> EncryptedQuery {
    let column_config = ColumnConfig::build("value")
        .casts_as(ColumnType::Text)
        .add_index(Index::new_unique());

    let identifier = Identifier::new(table_name, "value");

    EncryptedQueryBuilder::new(column_config, identifier)
        .index_type(Index::new_unique().index_type)
        .statement(query)
        .build_query(x, cipher)
        .await
        .expect("Failed to build encrypted query")
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS").unwrap_or_else(|_| "unknown".to_string());

    // Determine table suffix based on TARGET_ROWS
    let table_suffix = match target_rows.as_str() {
        "10000" | "100000" | "1000000" | "10000000" => format!("_{}", target_rows),
        _ => String::new(), // fallback to base table for unknown values
    };
    let table_name = format!("string_encrypted{}", table_suffix);

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

    // Derive the search term from a real row in the table so the bench
    // measures actual equality-query cost (index hit + 1-row return) rather
    // than the previous "0 rows / LIMIT 1 early exit" path.
    let search_term: String = rt.block_on(async {
        sample_plaintext_string(&pool, Arc::clone(&cipher), &table_name)
            .await
            .expect("failed to sample plaintext from table — is it populated?")
    });
    eprintln!(
        "exact bench: using sampled search term `{}` from `{}`",
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

    // Capture per-scenario metadata (exact SQL, bound parameter as JSON,
    // EXPLAIN plan, indexes the planner used) once at startup, before the
    // criterion loop runs. Writes `results/query/exact_metadata_<rows>.json`.
    // We only emit metadata for the non-decrypt variant — the decrypt
    // variant uses the same query plan, just with an extra client-side
    // decrypt pass.
    let metadata = rt.block_on(async {
        let mut out = Vec::with_capacity(queries.len());
        for (i, query) in queries.iter().enumerate() {
            let (_, scenario) = QUERY_TEMPLATES[i];
            let bench_id = format!("EXACT/exact/{}/{}", scenario, target_rows);
            let explain = query.explain(&pool).await.expect("EXPLAIN failed");
            let indexes_used = extract_indexes_used(&explain);
            let parameters = vec![query.parameter_json().expect("serialise parameter")];
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
            });
        }
        out
    });
    write_metadata_file("exact", &target_rows, metadata)
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
                // String_encrypted column holds Utf8Str — decrypt to Vec<String>.
                // Was previously typed as Vec<i32> which would have panicked on
                // try_from(Plaintext::Text → i32), but never triggered
                // because the bench search term matched zero rows.
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
