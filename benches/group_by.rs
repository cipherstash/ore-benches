use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{bench_assert, extract_indexes_used, write_metadata_file, ScenarioMetadata};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use sqlx::Row;
use tokio::runtime::Runtime;

// Realistic-cardinality GROUP BY scenarios. Each scenario runs against a
// low-cardinality categorical column (`CAT_001`..`CAT_250`, uniform random,
// ingested by `encrypt_category` / SQL `prepare:category_plaintext`). The
// 250-bucket cardinality is roughly an ISO 3166-1 country code distribution —
// large enough that the hash-aggregate table is interesting, small enough
// that the result-set emission cost stays negligible relative to the per-row
// HMAC work.
//
// Scenarios:
//
//   * `low_cardinality_groups_encrypted` — `SELECT count(*) FROM (SELECT 1
//     FROM category_encrypted GROUP BY eql_v2.hmac_256(value)) g`. The
//     extractor-form `GROUP BY` is the EQL recipe (cheap, in-memory
//     HashAggregate); wrapping in `count(*)` emits exactly one row so the
//     bench is unaffected by result-set marshalling. Companion plaintext
//     scenario uses the same shape on an unindexed TEXT column to give the
//     EQL overhead vs bare-PG aggregate baseline.
//
//   * `top_n_groups_encrypted` — `SELECT eql_v2.hmac_256(value), count(*)
//     FROM category_encrypted GROUP BY 1 ORDER BY count(*) DESC LIMIT 10`.
//     The dashboard-style "top N categories by frequency" pattern. Returns
//     10 rows always (one per top bucket), so the bench measures the
//     HashAggregate + sort, not result emission. Plaintext companion runs
//     the same shape on the TEXT column.
//
// Earlier versions of this bench grouped by the full ~1-2 KB ciphertext
// payload of high-cardinality (`fake::Name<EN>`, ~99% unique) data; both the
// time and the result-set shape were dominated by emit cost, which made the
// bench look like "GROUP BY on encrypted data is slow" when really it was
// "emitting millions of rows per query is slow". The current shapes isolate
// what callers actually pay for the GROUP BY itself.
//
// QUERY_TEMPLATES entries: (sql_template, scenario_name, base_table_name).
// `{TABLE}` is replaced with `<base_table_name>_<TARGET_ROWS>`.
static QUERY_TEMPLATES: &[(&str, &str, &str)] = &[
    (
        "SELECT count(*) FROM \
         (SELECT 1 FROM {TABLE} GROUP BY eql_v2.hmac_256(value)) g",
        "low_cardinality_groups_encrypted",
        "category_encrypted",
    ),
    (
        "SELECT count(*) FROM \
         (SELECT 1 FROM {TABLE} GROUP BY value) g",
        "low_cardinality_groups_plaintext",
        "category_plaintext",
    ),
    (
        "SELECT eql_v2.hmac_256(value), count(*) FROM {TABLE} \
         GROUP BY 1 ORDER BY count(*) DESC LIMIT 10",
        "top_n_groups_encrypted",
        "category_encrypted",
    ),
    (
        "SELECT value, count(*) FROM {TABLE} \
         GROUP BY 1 ORDER BY count(*) DESC LIMIT 10",
        "top_n_groups_plaintext",
        "category_plaintext",
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

    // Build per-scenario queries up front so we can also run EXPLAIN once
    // each before the criterion loop opens. The EXPLAIN pass writes a
    // `results/query/group_by_metadata_<rows>.json` sidecar capturing the
    // exact SQL, the planner's chosen plan, and any indexes the planner
    // picked. See lib.rs::write_metadata_file for the schema.
    let scenarios: Vec<(String, String)> = QUERY_TEMPLATES
        .iter()
        .map(|(query_template, scenario, base_table)| {
            let table_name = format!("{}{}", base_table, table_suffix);
            let query_str = query_template.replace("{TABLE}", &table_name);
            let bench_id = format!("GROUP_BY/group_by/{}/{}", scenario, target_rows);
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
            });
        }
        out
    });

    write_metadata_file("group_by", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("GROUP_BY");
    group.sample_size(10);

    for (bench_id, query_str) in scenarios {
        // bench_id is e.g. "GROUP_BY/group_by/low_cardinality_groups_encrypted/100000".
        // criterion's group.bench_function joins the group name ("GROUP_BY")
        // with the function name we pass; strip the leading "GROUP_BY/" so
        // it doesn't get doubled.
        let function_name = bench_id
            .strip_prefix("GROUP_BY/")
            .expect("bench_id missing GROUP_BY/ prefix")
            .to_string();
        let scenario_id = bench_id.clone();
        group.bench_function(function_name, |b| {
            b.to_async(&rt).iter(|| async {
                let rows =
                    bench_assert(sqlx::query(&query_str).fetch_all(&pool).await, &scenario_id);
                // Drain results to force aggregation to materialise. The
                // count-wrapped scenarios return a single i64; the top-N
                // scenarios return up to 10 (group-key bytes, count) rows
                // and we just sum the counts.
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
