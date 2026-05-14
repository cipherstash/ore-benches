use cipherstash_client::{
    encryption::ScopedCipher,
    eql::Identifier,
    schema::{
        column::{Index, IndexType},
        ColumnConfig, ColumnType,
    },
    AutoStrategy,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{init_scoped_cipher, EncryptedQuery, EncryptedQueryBuilder};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::runtime::Runtime;

// Post-EQL-2.3 (with the `<` / `<=` / `>` / `>=` operator inlining), bare-form
// range predicates on `eql_v2_encrypted` reduce to
// `eql_v2.ore_block_u64_8_256(a) <op> eql_v2.ore_block_u64_8_256(b)` and
// structurally match a functional btree index on
// `eql_v2.ore_block_u64_8_256(value)` — so the natural-form scenarios below
// engage the index without rewriting.
//
// Ordered range queries use the **hybrid form**: natural-form WHERE, extractor
// ORDER BY (`ORDER BY eql_v2.ore_block_u64_8_256(val)`). The sort key matches
// the functional index expression, so the planner streams rows out of the
// index in order — plain Index Scan, no Sort node. See §4 of the EQL
// query-performance guide for the underlying rule.
//
// Two ordered scenarios that previously sat alongside the hybrid one are no
// longer benched:
//
//   * The natural-form variant (`ORDER BY value`) is the §4 sort-key trap —
//     the planner can't satisfy the ORDER BY from the index, so it inserts a
//     Top-N Sort over the full post-WHERE bitmap. The cost scales linearly
//     with the number of rows passing WHERE: at 100k it's ~880 ms, at 1M
//     it's ~8.8 s. Documented in the guide, so the bench doesn't need to
//     keep proving it.
//
//   * The fully-extractor variant (`WHERE ore_block(val) < ore_block($1)
//     ORDER BY ore_block(val)`) inlines to the same predicate shape as the
//     hybrid, so its plan and timing are identical — pure redundancy.
//
// The equality scenario from the previous bench (`WHERE value = $1`) is gone:
// the integer column carries only `ob`, not `hm`, so post-2.3 equality returns
// NULL → zero rows. See exact.rs for the meaningful equality benches.
static QUERY_TEMPLATES: &[(&str, i32, &str)] = &[
    (
        "SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 10",
        5000,
        "range_gt_10",
    ),
    (
        "SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 100",
        5000,
        "range_gt_100",
    ),
    (
        "SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 LIMIT 10",
        5000,
        "range_lt_10",
    ),
    (
        "SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 LIMIT 100",
        5000,
        "range_lt_100",
    ),
    (
        "SELECT id,value::jsonb FROM {TABLE} \
         WHERE value < $1 \
         ORDER BY eql_v2.ore_block_u64_8_256(value) LIMIT 10",
        5000,
        "range_lt_hybrid_ordered_10",
    ),
];

async fn build_query(
    cipher: Arc<ScopedCipher<AutoStrategy>>,
    query: &str,
    x: i32,
    table_name: &str,
) -> EncryptedQuery {
    let column_config = ColumnConfig::build("value")
        .casts_as(ColumnType::Int)
        .add_index(Index::new_ore());

    let identifier = Identifier::new(table_name, "value");

    EncryptedQueryBuilder::new(column_config, identifier)
        .index_type(IndexType::Ore)
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
    let table_name = format!("integer_encrypted{}", table_suffix);

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

    let mut group = c.benchmark_group("ORE");
    group.sample_size(10);
    // All remaining scenarios run sub-ms to single-digit-ms per iteration, so
    // criterion's default measurement budget is plenty. (Earlier versions of
    // this bench needed a 30 s budget for the natural-form ordered range
    // scenario; that scenario is gone — see the comment on `QUERY_TEMPLATES`.)

    for (i, query) in queries.into_iter().enumerate() {
        let (_, _, scenario) = QUERY_TEMPLATES[i];

        group.bench_function(format!("ore/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _: Vec<_> = query.execute(&pool).await.unwrap();
            })
        });

        group.bench_function(format!("ore_decrypt/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _r: Vec<i32> = black_box(query.execute_and_decrypt(&pool).await.unwrap());
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
