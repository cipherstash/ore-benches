use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{extract_indexes_used, write_metadata_file, ScenarioMetadata};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use sqlx::Row;
use tokio::runtime::Runtime;

// Two scenarios, same query shape, side-by-side encrypted vs plaintext.
//
//   count_groups_encrypted — `GROUP BY eql_v2.hmac_256(value)` against
//                            `string_encrypted_<N>` (encrypted column,
//                            inlinable HMAC extractor).
//
//   count_groups_plaintext — `GROUP BY value` against `string_plaintext_<N>`
//                            (plain TEXT column, no encryption). The
//                            baseline: "what would this cost without
//                            encryption?" The plaintext data is
//                            high-cardinality `md5(random()::text)` so the
//                            cardinality matches the encrypted side (fake
//                            random names give similar ~99% uniqueness).
//
// Both queries wrap the GROUP BY in `count(*)`:
//
//   SELECT count(*) FROM (SELECT 1 FROM tbl GROUP BY <key>) g
//
// rather than the bare `SELECT count(*) FROM tbl GROUP BY <key>` form. With
// effectively-unique rows the bare form emits ~one row per input row, so
// wall-clock is bottlenecked by result emission (server-side row
// construction, network round-trip, sqlx deserialisation, the bench's own
// iter-and-sum), not by aggregation work. The subquery wrapper keeps the
// inner HashAggregate identical but emits one row regardless of
// cardinality — both scenarios measure aggregation cost cleanly.
//
// The natural form (`GROUP BY value` against `eql_v2_encrypted`) was dropped
// from this bench earlier. The planner picks `GroupAggregate` + sort against
// the full ~1-2 KB ciphertext payload at scale; the cost is the planner's
// work_mem fallback, not anything EQL controls. See §5 of EQL's
// `docs/reference/query-performance.md`.
//
// QUERY_TEMPLATES entries: (sql_template, scenario_name, base_table_name).
// The bench substitutes `{TABLE}` with `<base_table_name>_<TARGET_ROWS>` so
// each scenario runs against its own table family.
static QUERY_TEMPLATES: &[(&str, &str, &str)] = &[
    (
        "SELECT count(*) FROM \
         (SELECT 1 FROM {TABLE} GROUP BY eql_v2.hmac_256(value)) g",
        "count_groups_encrypted",
        "string_encrypted",
    ),
    (
        "SELECT count(*) FROM \
         (SELECT 1 FROM {TABLE} GROUP BY value) g",
        "count_groups_plaintext",
        "string_plaintext",
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
            out.push(ScenarioMetadata {
                id: bench_id.clone(),
                query: query_str.clone(),
                parameters: Vec::new(),
                explain,
                indexes_used,
            });
        }
        out
    });

    write_metadata_file("group_by", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("GROUP_BY");
    group.sample_size(10);

    for (bench_id, query_str) in scenarios {
        // bench_id is e.g. "GROUP_BY/group_by/count_groups_encrypted/100000".
        // criterion's group.bench_function joins the group name ("GROUP_BY")
        // with the function name we pass; strip the leading "GROUP_BY/" so
        // it doesn't get doubled.
        let function_name = bench_id
            .strip_prefix("GROUP_BY/")
            .expect("bench_id missing GROUP_BY/ prefix")
            .to_string();
        group.bench_function(function_name, |b| {
            b.to_async(&rt).iter(|| async {
                let rows = sqlx::query(&query_str)
                    .fetch_all(&pool)
                    .await
                    .expect("group_by query failed");
                // Drain the single-row result to force the aggregation to materialise.
                black_box(rows.iter().map(|r| r.get::<i64, _>(0)).sum::<i64>())
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
