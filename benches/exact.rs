use cipherstash_client::{
    encryption::ScopedCipher,
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
    AutoStrategy,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{
    extract_indexes_used, init_scoped_cipher, write_metadata_file, EncryptedQuery,
    EncryptedQueryBuilder, ScenarioMetadata,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::runtime::Runtime;

static QUERY_TEMPLATES: &[(&str, &str, &str)] = &[
    ("SELECT value FROM {TABLE} WHERE value = $1 LIMIT 1", "Bob Johnson", "eql_cast"),
    ("SELECT value FROM {TABLE} WHERE eql_v2.hmac_256(value) = eql_v2.hmac_256($1::jsonb) LIMIT 1", "Bob Johnson", "eql_hash"),
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

    let target_rows = std::env::var("TARGET_ROWS")
        .unwrap_or_else(|_| "unknown".to_string());

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

    let queries = rt.block_on(async {
        let mut queries = Vec::with_capacity(QUERY_TEMPLATES.len());
        for (query_template, x, _) in QUERY_TEMPLATES {
            let query_str = query_template.replace("{TABLE}", &table_name);
            let query = build_query(Arc::clone(&cipher), &query_str, *x, &table_name).await;
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
            let (_, _, scenario) = QUERY_TEMPLATES[i];
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
        let (_, _, scenario) = QUERY_TEMPLATES[i];

        group.bench_function(format!("exact/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _: Vec<_> = query.execute(&pool).await.unwrap();
            })
        });

        group.bench_function(format!("exact_decrypt/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _r: Vec<i32> = black_box(query.execute_and_decrypt(&pool).await.unwrap());
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
