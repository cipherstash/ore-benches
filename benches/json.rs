use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{extract_indexes_used, write_metadata_file, ScenarioMetadata};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use sqlx::Row;
use tokio::runtime::Runtime;

// JSON / ste_vec query benches. Three shapes that exercise three distinct
// planner paths — all using the extractor recipes the EQL query-performance
// guide recommends:
//
//   field_eq        — `WHERE eql_v2.hmac_256_terms(col) @> $1::jsonb`,
//                     parameter `[{"s":"<sel>","hm":"<hash>"}]`. Engages the
//                     `GIN (eql_v2.hmac_256_terms(value))` index. One index
//                     covers field-level equality across every selector that
//                     carries an `hm` term, vs the per-selector
//                     `hash (eql_v2.hmac_256(col, '<sel>'))` recipe which
//                     needs one index per hot path.
//
//   field_extract   — `SELECT eql_v2.jsonb_path_query(col, '<sel>') FROM tbl LIMIT n`.
//                     No index. Measures the per-row cost of the inlinable
//                     `jsonb_path_query` body: `jsonb_array_elements((val).data -> 'sv')
//                     WHERE elem ->> 's' = selector`. Inlining means the body
//                     folds into the calling query — each row pays an array
//                     walk, not a plpgsql function call.
//
//   field_group_by  — `SELECT eql_v2.hmac_256(col, '<sel>'), count(*) FROM tbl
//                     GROUP BY 1`. No index needed; HashAggregate over the
//                     32-byte HMAC group key fits in default `work_mem`. Same
//                     extractor-form recipe as §5 of the perf guide, scaled
//                     down to a field-level lookup inside an ste_vec doc.
//
// Setup: at startup the bench reads one sample row from the target table,
// extracts a (selector_hash, hmac_value) pair from `sv[0]`, and uses those in
// the queries. The pick is data-driven — the bench works against any
// ste_vec-shaped table without static knowledge of which selectors are hot —
// at the cost of the pick being arbitrary across runs. For comparison purposes
// the (selector, hmac) pair is stable within a single run.

#[derive(Debug)]
struct Selector {
    sel: String,
    hmac: String,
}

async fn pick_selector(pool: &sqlx::PgPool, table: &str) -> Selector {
    // (value).data -> 'sv' is the array of ste_vec elements; element 0's `s`
    // is a selector hash and `hm` is the HMAC term for that selector's value.
    // We pick the first sv element that carries `hm` so the chosen selector
    // is HMAC-indexed (i.e. addressable via hmac_256_terms / hmac_256(col, sel)).
    let row = sqlx::query(&format!(
        "SELECT (value).data -> 'sv' -> 0 ->> 's' AS sel,
                (value).data -> 'sv' -> 0 ->> 'hm' AS hmac
         FROM {table}
         WHERE (value).data -> 'sv' -> 0 ->> 'hm' IS NOT NULL
         LIMIT 1"
    ))
    .fetch_optional(pool)
    .await
    .expect("query for sample selector failed");

    let row = row.unwrap_or_else(|| {
        panic!(
            "No sv elements carry `hm` in `{table}`. The bench's field-level scenarios \
             depend on the post-2.3 ste_vec shape (sv elements emit `hm` rather than `b3`), \
             but the cipherstash-client version pinned in this repo \
             (currently 0.34.1-alpha.4) still emits the pre-2.3 shape. Resolve by bumping \
             `cipherstash-client` to a version that emits the new ste_vec element shape, \
             then re-ingest via `mise run prepare:json_ste_vec_small <rows>`. See \
             U-004 in EQL's v2.3 upgrade notes for the payload change."
        )
    });

    Selector {
        sel: row.get::<String, _>("sel"),
        hmac: row.get::<String, _>("hmac"),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS")
        .unwrap_or_else(|_| "unknown".to_string());

    let table_suffix = match target_rows.as_str() {
        "10000" | "100000" | "1000000" | "10000000" => format!("_{}", target_rows),
        _ => String::new(),
    };
    let table_name = format!("json_ste_vec_small_encrypted{}", table_suffix);

    let (pool, selector) = rt.block_on(async {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let selector = pick_selector(&pool, &table_name).await;
        eprintln!(
            "json bench picked selector `{}` (hmac ends in `{}`) from `{}`",
            &selector.sel,
            // last 8 chars of hmac, just to make runs visually distinguishable.
            // Full hmac stays in `selector.hmac` for the field_eq parameter.
            selector.hmac.chars().rev().take(8).collect::<String>().chars().rev().collect::<String>(),
            table_name
        );
        (pool, selector)
    });

    // Pre-build the field_eq parameter: a jsonb array with one
    // `{"s": <sel>, "hm": <hmac>}` element that matches at least the source
    // row (and likely several more if the value isn't unique).
    let needle = format!(
        r#"[{{"s":"{}","hm":"{}"}}]"#,
        selector.sel, selector.hmac
    );

    let q_field_eq = format!(
        "SELECT id FROM {table_name} \
         WHERE eql_v2.hmac_256_terms(value) @> $1::jsonb \
         LIMIT 10"
    );

    let q_field_extract = format!(
        "SELECT eql_v2.jsonb_path_query(value, '{}') FROM {table_name} LIMIT 1000",
        selector.sel
    );

    let q_field_group_by = format!(
        "SELECT eql_v2.hmac_256(value, '{}'), count(*) FROM {table_name} GROUP BY 1",
        selector.sel
    );

    // Capture per-scenario metadata before the criterion loop. Writes
    // `results/query/json_metadata_<rows>.json`.
    let metadata_scenarios = rt.block_on(async {
        let mut out: Vec<ScenarioMetadata> = Vec::with_capacity(3);

        async fn explain_one(
            pool: &sqlx::PgPool,
            query: &str,
            bind: Option<&str>,
        ) -> serde_json::Value {
            let explain_sql = format!("EXPLAIN (FORMAT JSON) {}", query);
            let row: (Json<serde_json::Value>,) = if let Some(b) = bind {
                sqlx::query_as(&explain_sql)
                    .bind(b)
                    .fetch_one(pool)
                    .await
                    .expect("EXPLAIN failed")
            } else {
                sqlx::query_as(&explain_sql)
                    .fetch_one(pool)
                    .await
                    .expect("EXPLAIN failed")
            };
            row.0 .0
        }

        let explain = explain_one(&pool, &q_field_eq, Some(&needle)).await;
        let indexes_used = extract_indexes_used(&explain);
        out.push(ScenarioMetadata {
            id: format!("JSON/json/field_eq/{}", target_rows),
            query: q_field_eq.clone(),
            parameters: vec![serde_json::Value::String(needle.clone())],
            explain,
            indexes_used,
        });

        let explain = explain_one(&pool, &q_field_extract, None).await;
        let indexes_used = extract_indexes_used(&explain);
        out.push(ScenarioMetadata {
            id: format!("JSON/json/field_extract/{}", target_rows),
            query: q_field_extract.clone(),
            parameters: Vec::new(),
            explain,
            indexes_used,
        });

        let explain = explain_one(&pool, &q_field_group_by, None).await;
        let indexes_used = extract_indexes_used(&explain);
        out.push(ScenarioMetadata {
            id: format!("JSON/json/field_group_by/{}", target_rows),
            query: q_field_group_by.clone(),
            parameters: Vec::new(),
            explain,
            indexes_used,
        });

        out
    });
    write_metadata_file("json", &target_rows, metadata_scenarios)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("JSON");
    group.sample_size(10);

    group.bench_function(format!("json/field_eq/{}", target_rows), |b| {
        b.to_async(&rt).iter(|| async {
            let rows = sqlx::query(&q_field_eq)
                .bind(&needle)
                .fetch_all(&pool)
                .await
                .expect("field_eq query failed");
            black_box(rows.iter().map(|r| r.get::<i32, _>(0)).sum::<i32>())
        })
    });

    group.bench_function(format!("json/field_extract/{}", target_rows), |b| {
        b.to_async(&rt).iter(|| async {
            let rows = sqlx::query(&q_field_extract)
                .fetch_all(&pool)
                .await
                .expect("field_extract query failed");
            black_box(rows.len())
        })
    });

    group.bench_function(format!("json/field_group_by/{}", target_rows), |b| {
        b.to_async(&rt).iter(|| async {
            let rows = sqlx::query(&q_field_group_by)
                .fetch_all(&pool)
                .await
                .expect("field_group_by query failed");
            black_box(rows.iter().map(|r| r.get::<i64, _>(1)).sum::<i64>())
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
