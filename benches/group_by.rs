use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use tokio::runtime::Runtime;

// Two flavours of the same GROUP BY against the string_encrypted_* tables:
//
// 1. "eql_cast" — natural form: `GROUP BY value`. The hash discriminator for
//    aggregation is provided by `eql_v2.hash_encrypted`, which is a plpgsql
//    function called once per row. Not inlinable.
//
// 2. "hmac_extractor" — explicit form: `GROUP BY eql_v2.hmac_256(value)`. The
//    extractor is an inlinable single-statement SQL function (post 2.3), so the
//    planner folds the body — `(val).data ->> 'hm'` — into the aggregation.
//
// PostgreSQL builds an in-memory hash table for GROUP BY in both cases (the
// functional hash index on `eql_v2.hmac_256(value)` is only useful for
// equality lookups, not aggregation), so this is really a comparison of
// per-row hashing cost: plpgsql function call vs. inlined SQL.
static QUERY_TEMPLATES: &[(&str, &str)] = &[
    (
        "SELECT count(*) FROM {TABLE} GROUP BY value",
        "eql_cast",
    ),
    (
        "SELECT count(*) FROM {TABLE} GROUP BY eql_v2.hmac_256(value)",
        "hmac_extractor",
    ),
];

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS")
        .unwrap_or_else(|_| "unknown".to_string());

    let table_suffix = match target_rows.as_str() {
        "10000" | "100000" | "1000000" | "10000000" => format!("_{}", target_rows),
        _ => String::new(),
    };
    let table_name = format!("string_encrypted{}", table_suffix);

    let pool = rt.block_on(async {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database")
    });

    let mut group = c.benchmark_group("GROUP_BY");
    group.sample_size(10);
    // The natural-form `GROUP BY value` scenario calls `eql_v2.hash_encrypted`
    // (plpgsql, per row) for the hash discriminator. At 10k rows that's
    // ~3.5 s per iteration; at 100k+ it scales roughly linearly. Criterion's
    // default 5 s `measurement_time` can't fit a single sample. Extend so
    // even the slow scenarios get the criterion-minimum 10 samples without
    // a "Unable to complete 10 samples" warning. Inflated for headroom at
    // 1M rows.
    group.warm_up_time(std::time::Duration::from_secs(5));
    group.measurement_time(std::time::Duration::from_secs(60));

    for (query_template, scenario) in QUERY_TEMPLATES {
        let query_str = query_template.replace("{TABLE}", &table_name);

        group.bench_function(format!("group_by/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let rows = sqlx::query(&query_str)
                    .fetch_all(&pool)
                    .await
                    .expect("group_by query failed");
                // Drain the result to force the aggregation to materialise.
                black_box(rows.iter().map(|r| r.get::<i64, _>(0)).sum::<i64>())
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
