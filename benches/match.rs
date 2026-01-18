use cipherstash_client::{
    credentials::ServiceCredentials,
    encryption::ScopedCipher,
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{init_scoped_cipher, EncryptedQuery, EncryptedQueryBuilder};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::runtime::Runtime;

static QUERIES: &[(&str, &str, &str)] = &[
    ("SELECT id,value::jsonb FROM string_encrypted WHERE value LIKE $1 LIMIT 10", "Bob", "eql_cast_firstname"),
    ("SELECT id,value::jsonb FROM string_encrypted WHERE value LIKE $1 LIMIT 10", "Johnson", "eql_cast_lastname"),
    ("SELECT id,value::jsonb FROM string_encrypted WHERE eql_v2.bloom_filter(value) @> eql_v2.bloom_filter($1) LIMIT 10", "Johnson", "eql_bloom"),
];

async fn build_query(
    cipher: Arc<ScopedCipher<ServiceCredentials>>,
    query: &str,
    x: &str,
) -> EncryptedQuery {
    let column_config = ColumnConfig::build("value")
        .casts_as(ColumnType::Utf8Str)
        .add_index(Index::new_match());

    let identifier = Identifier::new("string_encrypted", "value");

    EncryptedQueryBuilder::new(column_config, identifier)
        .index_type(Index::new_match().index_type)
        .statement(query)
        .build_query(x, cipher)
        .await
        .expect("Failed to build encrypted query")
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS")
        .unwrap_or_else(|_| "unknown".to_string());

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
        let mut queries = Vec::with_capacity(QUERIES.len());
        for (query_str, x, _) in QUERIES {
            let query = build_query(Arc::clone(&cipher), query_str, x).await;
            queries.push(query);
        }
        queries
    });

    let mut group = c.benchmark_group("MATCH");
    group.sample_size(10);

    for (i, query) in queries.into_iter().enumerate() {
        let (_, _, scenario) = QUERIES[i];
        
        group.bench_function(format!("match/{}/rows_{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _: Vec<_> = query.execute(&pool).await.unwrap();
            })
        });

        group.bench_function(format!("match_decrypt/{}/rows_{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _r: Vec<String> = black_box(query.execute_and_decrypt(&pool).await.unwrap());
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
