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
// The ordered scenarios show three plan shapes side-by-side:
//
//   range_lt_ordered_10        — natural form: WHERE val < $1 ORDER BY val LIMIT 10
//                                 → Bitmap Index Scan via the inlined `<`, plus
//                                   a Top-N sort by `val` (the natural-form sort
//                                   key doesn't match the index expression
//                                   syntactically). Each comparison in the Sort
//                                   step uses the inlined ORE-term path, so the
//                                   Top-N is fast.
//
//   range_lt_hybrid_ordered_10 — natural WHERE, extractor ORDER BY:
//                                 ORDER BY eql_v2.ore_block_u64_8_256(val).
//                                 The sort key matches the index expression →
//                                 plain ordered Index Scan, no Sort node.
//
//   range_lt_ore_ordered_10    — fully extractor on both clauses. After the `<`
//                                 inlining the WHERE reduces to the same shape
//                                 as the hybrid, so the plan is identical to
//                                 hybrid. Kept for contrast / regression.
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
        "SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 ORDER BY value LIMIT 10",
        5000,
        "range_lt_ordered_10",
    ),
    (
        "SELECT id,value::jsonb FROM {TABLE} \
         WHERE value < $1 \
         ORDER BY eql_v2.ore_block_u64_8_256(value) LIMIT 10",
        5000,
        "range_lt_hybrid_ordered_10",
    ),
    (
        "SELECT id,value::jsonb FROM {TABLE} \
         WHERE eql_v2.ore_block_u64_8_256(value) < eql_v2.ore_block_u64_8_256($1::jsonb) \
         ORDER BY eql_v2.ore_block_u64_8_256(value) LIMIT 10",
        5000,
        "range_lt_ore_ordered_10",
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
    // Some scenarios — notably the natural-form `WHERE val < $1 ORDER BY val
    // LIMIT 10` — finish a single iteration in several hundred milliseconds
    // because the Top-N sort runs over the post-WHERE bitmap rather than
    // streaming from an ordered index (see U-005 in EQL's v2.3 upgrade
    // notes). Criterion's default 5 s `measurement_time` only fits a few
    // such samples, yielding very wide confidence intervals and false
    // "regressed" alerts against any stored baseline. 30 s gives the slow
    // scenarios room to settle while leaving fast ones (sub-ms to single
    // ms) plenty of headroom.
    group.warm_up_time(std::time::Duration::from_secs(5));
    group.measurement_time(std::time::Duration::from_secs(30));

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
