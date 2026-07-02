//! EQL v3 twin of `benches/group_by.rs` — realistic-cardinality GROUP BY
//! against `category_encrypted_v3_<N>` (column typed `eql_v3.text_eq`,
//! ~250 distinct buckets).
//!
//! Scenarios mirror the v2 encrypted shapes with `eql_v3.eq_term` in place
//! of `eql_v2.hmac_256`:
//!
//!   * `low_cardinality_groups_encrypted` — count(*)-wrapped extractor
//!     GROUP BY (isolates HashAggregate cost from emit cost).
//!   * `top_n_groups_encrypted` — top-10-categories dashboard shape.
//!
//! The plaintext baselines (`*_plaintext`) are NOT duplicated here: the
//! `category_plaintext_<N>` tables are version-independent and the v2
//! group_by bench already measures them — compare the v3 encrypted numbers
//! against those same baselines in the report.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{bench_assert, extract_indexes_used, write_metadata_file, ScenarioMetadata};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use sqlx::Row;
use tokio::runtime::Runtime;

// (sql_template, scenario_name, base_table_name). `{TABLE}` is replaced
// with `<base_table_name>_<TARGET_ROWS>`.
static QUERY_TEMPLATES: &[(&str, &str, &str)] = &[
    (
        "SELECT count(*) FROM \
         (SELECT 1 FROM {TABLE} GROUP BY eql_v3.eq_term(value)) g",
        "low_cardinality_groups_encrypted",
        "category_encrypted_v3",
    ),
    (
        "SELECT eql_v3.eq_term(value), count(*) FROM {TABLE} \
         GROUP BY 1 ORDER BY count(*) DESC LIMIT 10",
        "top_n_groups_encrypted",
        "category_encrypted_v3",
    ),
];

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS").unwrap_or_else(|_| "unknown".to_string());

    let table_suffix = match target_rows.as_str() {
        "10000" | "100000" | "1000000" | "10000000" => format!("_{}", target_rows),
        _ => String::new(),
    };

    let pool = rt.block_on(async {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database")
    });

    let scenarios: Vec<(String, String)> = QUERY_TEMPLATES
        .iter()
        .map(|(query_template, scenario, base_table)| {
            let table_name = format!("{}{}", base_table, table_suffix);
            let query_str = query_template.replace("{TABLE}", &table_name);
            let bench_id = format!("GROUP_BY_V3/group_by/{}/{}", scenario, target_rows);
            (bench_id, query_str)
        })
        .collect();

    let metadata = rt.block_on(async {
        let mut out = Vec::with_capacity(scenarios.len());
        for (bench_id, query_str) in &scenarios {
            let explain_sql = format!("EXPLAIN (FORMAT JSON) {}", query_str);
            let plan: (Json<serde_json::Value>,) = sqlx::query_as(&explain_sql)
                .fetch_one(&pool)
                .await
                .expect("EXPLAIN failed for bench scenario");
            let explain = plan.0 .0;
            let indexes_used = extract_indexes_used(&explain);
            let rows = sqlx::query(query_str)
                .fetch_all(&pool)
                .await
                .expect("execute for row-count failed");
            let rows_returned = rows.len() as u64;
            out.push(ScenarioMetadata {
                id: bench_id.clone(),
                query: query_str.clone(),
                parameters: Vec::new(),
                explain,
                indexes_used,
                rows_returned,
                version: 3,
            });
        }
        out
    });

    write_metadata_file("group_by_v3", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("GROUP_BY_V3");
    group.sample_size(10);

    for (bench_id, query_str) in scenarios {
        let function_name = bench_id
            .strip_prefix("GROUP_BY_V3/")
            .expect("bench_id missing GROUP_BY_V3/ prefix")
            .to_string();
        let scenario_id = bench_id.clone();
        group.bench_function(function_name, |b| {
            b.to_async(&rt).iter(|| async {
                let rows =
                    bench_assert(sqlx::query(&query_str).fetch_all(&pool).await, &scenario_id);
                // Drain results to force aggregation to materialise — the
                // count-wrapped scenario returns a single i64; top-N returns
                // up to 10 (group key, count) rows.
                if rows.len() == 1 {
                    black_box(rows[0].get::<i64, _>(0));
                } else {
                    black_box(rows.iter().map(|r| r.get::<i64, _>(1)).sum::<i64>());
                }
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
