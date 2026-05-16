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
use dbbenches::{
    bench_assert, extract_indexes_used, init_scoped_cipher, init_tracing, write_metadata_file,
    EncryptedQuery, EncryptedQueryBuilder, ScenarioMetadata,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use std::sync::Arc;
use tokio::runtime::Runtime;

// Post-EQL-2.3 (with the `<` / `<=` / `>` / `>=` operator inlining), bare-form
// range predicates on `eql_v2_encrypted` reduce to
// `eql_v2.ore_block_u64_8_256(a) <op> eql_v2.ore_block_u64_8_256(b)` and
// structurally match a functional btree index on
// `eql_v2.ore_block_u64_8_256(value)`. Whether the planner *uses* that index
// depends on predicate selectivity — see the two scenario families below.
//
// **Non-selective baselines** (threshold 5000 against `Faker.fake::<i32>()`
// data — uniform across the full i32 range, so 5000 sits very close to the
// median, giving ~50% selectivity). With a LIMIT, the planner correctly picks
// `Seq Scan + LIMIT` over a bitmap index scan: at 50% selectivity it expects
// to find LIMIT matches within the first handful of pages, cheaper than the
// index-then-heap-fetch roundtrip. So these scenarios show empty
// `indexes_used: []` in the metadata sidecar — which is the planner choosing
// correctly, not the index failing to engage. The scenarios remain in the
// suite because they're the natural form a caller would write, and the
// timing tells us what that case actually costs.
//
// **Selective scenarios with LIMIT** (thresholds 2_140_000_000 / 2_147_000_000
// — out at the i32 tail). Selectivity drops to ~0.17% and ~0.011% respectively.
// The planner picks Index Scan at every tier (10k → 10M) when stats are
// current: walking the b-tree from the top and returning the first LIMIT rows
// is cheaper than scanning the whole table.
//
// **Stats matter.** Without an `ANALYZE` after the table is re-ingested, the
// planner defaults to `~14%` selectivity for `>` comparisons and picks Seq
// Scan even for highly selective predicates. The bench's `prepare:_table`
// task now runs `ANALYZE <table>` after index creation specifically to avoid
// this silent-fallback failure mode.
//
// **Selective scenarios without LIMIT** (`*_count` — `SELECT count(*) WHERE
// value <op> threshold`). No LIMIT means the planner must process every
// matching row to compute the count; with a selective predicate this
// strongly favours Index Scan over Seq Scan at every tier. Companion to the
// `_LIMIT` variants — removes LIMIT-related cost-model edge cases.
//
// **Hybrid ordered range** uses extractor ORDER BY (`ORDER BY
// eql_v2.ore_block_u64_8_256(val)`) matching the functional index expression —
// rows stream out of the index already sorted (Index Scan, no Sort node). The
// natural-form variant (`ORDER BY value`) is the §4 sort-key trap and was
// dropped from this bench in an earlier pass — its cost (Top-N Sort over the
// full post-WHERE bitmap) is documented in the guide already.
static QUERY_TEMPLATES: &[(&str, i32, &str)] = &[
    // ── Non-selective baselines (≈50% selectivity → Seq Scan + LIMIT) ──
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
    // ── Selective predicates (~0.17% / ~0.011% selectivity → Index Scan) ──
    // 2_140_000_000 sits 7.5M values short of i32::MAX — ~0.17% of the i32
    // range matches `value > 2_140_000_000` on uniform random data.
    (
        "SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 100",
        2_140_000_000,
        "range_selective_gt_100",
    ),
    // 2_147_000_000 sits 483k values short of i32::MAX — ~0.011% selectivity.
    // Even at 10k rows this returns ~1 row, but the planner can decide that
    // before scanning; index engages reliably across tiers.
    (
        "SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 10",
        2_147_000_000,
        "range_highly_selective_gt_10",
    ),
    // ── Hybrid ordered range (extractor in ORDER BY) ──
    (
        "SELECT id,value::jsonb FROM {TABLE} \
         WHERE value < $1 \
         ORDER BY eql_v2.ore_block_u64_8_256(value) LIMIT 10",
        5000,
        "range_lt_hybrid_ordered_10",
    ),
];

// Count-style selective scenarios — no LIMIT, so the planner must process
// every matching row, which strongly favours Index Scan on the functional
// btree once selectivity is low. These reliably engage the index at every
// tier (10k → 10M) and are the canonical "yes the ORE index is doing real
// work" demonstration. SELECT count(*) returns a single row; the bench
// loop drains it and black_boxes the result.
static COUNT_QUERY_TEMPLATES: &[(&str, i32, &str)] = &[
    (
        "SELECT count(*) FROM {TABLE} WHERE value > $1",
        2_140_000_000,
        "range_selective_gt_count",
    ),
    (
        "SELECT count(*) FROM {TABLE} WHERE value > $1",
        2_147_000_000,
        "range_highly_selective_gt_count",
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
    // Wire up cipherstash-client / zerokms-protocol trace! emissions to
    // stderr when RUST_LOG is set. No-op when unset.
    init_tracing();

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

    // Count-style scenarios reuse the same build_query path (to encrypt
    // the threshold as the bound parameter) but produce a different SQL
    // shape — they're executed via raw sqlx in the iter loop below
    // because EncryptedQuery::execute is typed for `Vec<(i32,
    // Json<EqlCiphertext>)>`, which doesn't match a `SELECT count(*)`
    // result.
    let count_queries = rt.block_on(async {
        let mut queries = Vec::with_capacity(COUNT_QUERY_TEMPLATES.len());
        for (query_template, x, _) in COUNT_QUERY_TEMPLATES {
            let query_str = query_template.replace("{TABLE}", &table_name);
            let query = build_query(Arc::clone(&cipher), &query_str, *x, &table_name).await;
            queries.push(query);
        }
        queries
    });

    // Capture per-scenario metadata (exact SQL, bound parameter, EXPLAIN
    // plan, indexes used) before the criterion loop. Writes
    // `results/query/ore_metadata_<rows>.json`.
    let metadata = rt.block_on(async {
        let mut out = Vec::with_capacity(queries.len() + count_queries.len());
        for (i, query) in queries.iter().enumerate() {
            let (_, _, scenario) = QUERY_TEMPLATES[i];
            let bench_id = format!("ORE/ore/{}/{}", scenario, target_rows);
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

        // Count-scenarios: same metadata shape, but use raw sqlx (the
        // count(*) return type doesn't match EncryptedQuery::execute).
        for (i, query) in count_queries.iter().enumerate() {
            let (_, _, scenario) = COUNT_QUERY_TEMPLATES[i];
            let bench_id = format!("ORE/ore/{}/{}", scenario, target_rows);
            let explain = query.explain(&pool).await.expect("EXPLAIN failed");
            let indexes_used = extract_indexes_used(&explain);
            let parameters = vec![query.parameter_json().expect("serialise parameter")];
            let rows = sqlx::query(&query.statement)
                .bind(Json(&query.eql))
                .fetch_all(&pool)
                .await
                .expect("count(*) execute for row-count failed");
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
    write_metadata_file("ore", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("ORE");
    group.sample_size(10);
    // All remaining scenarios run sub-ms to single-digit-ms per iteration, so
    // criterion's default measurement budget is plenty. (Earlier versions of
    // this bench needed a 30 s budget for the natural-form ordered range
    // scenario; that scenario is gone — see the comment on `QUERY_TEMPLATES`.)

    for (i, query) in queries.into_iter().enumerate() {
        let (_, _, scenario) = QUERY_TEMPLATES[i];
        let exec_id = format!("ORE/ore/{}/{}", scenario, target_rows);
        let decrypt_id = format!("ORE/ore_decrypt/{}/{}", scenario, target_rows);

        let exec_id_inner = exec_id.clone();
        group.bench_function(format!("ore/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _: Vec<_> = bench_assert(query.execute(&pool).await, &exec_id_inner);
            })
        });

        let decrypt_id_inner = decrypt_id.clone();
        group.bench_function(format!("ore_decrypt/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _r: Vec<i32> = black_box(bench_assert(
                    query.execute_and_decrypt(&pool).await,
                    &decrypt_id_inner,
                ));
            })
        });
    }

    // Count-style scenarios — single iter loop each, raw sqlx (count(*)
    // return type doesn't match EncryptedQuery::execute). No `_decrypt`
    // variant — there's nothing to decrypt in a count result.
    for (i, query) in count_queries.into_iter().enumerate() {
        let (_, _, scenario) = COUNT_QUERY_TEMPLATES[i];
        let exec_id = format!("ORE/ore/{}/{}", scenario, target_rows);
        let exec_id_inner = exec_id.clone();
        group.bench_function(format!("ore/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let rows = bench_assert(
                    sqlx::query(&query.statement)
                        .bind(Json(&query.eql))
                        .fetch_all(&pool)
                        .await,
                    &exec_id_inner,
                );
                black_box(rows.len());
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
