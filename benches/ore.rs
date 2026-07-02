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
// **Selective scenarios — all disabled (see EQL issue #230).** A genuinely
// selective range predicate exposes a planner limitation. The comparison
// value is a bound parameter, not a plan-time constant, so the planner
// cannot estimate the selectivity of the encrypted ORE comparison — it falls
// back to `DEFAULT_INEQ_SEL` (33%) and picks a Seq Scan. That is correct
// when the predicate really is non-selective, and pathological when it is
// selective (a near-full scan of the table). `ANALYZE` does not rescue it:
// the histogram it builds on `eql_v2.ore_block_u64_8_256(value)` is unusable
// without a constant on the other side of the `>`.
//
// Every selective scenario — `range_selective_gt_100`,
// `range_highly_selective_gt_10`, and both `*_count` variants — is therefore
// commented out below: at the 10M tier each degrades into a near-full seq
// scan (seconds per iteration). The count variants are the worst case —
// no LIMIT to cut the scan short. Re-enable them once #230 lands a
// selectivity fix: https://github.com/cipherstash/encrypt-query-language/issues/230
//
// **Ordered range** uses extractor ORDER BY (`ORDER BY
// eql_v2.ore_block_u64_8_256(val)`) matching the functional index expression —
// rows stream out of the index already sorted (Index Scan, no Sort node).
// The pair scenario `range_lt_natural_ordered_10` (`ORDER BY value` with no
// extractor) was previously included as a "what the sort-key trap costs"
// baseline, but it ran in seconds at the 10M tier and is the deliberate
// anti-pattern documented in `docs/reference/query-performance.md` §4 of
// the EQL repo — a public bench shouldn't headline pathological cases.
//
// Note on projection — every scenario projects `value` directly rather than
// `value::jsonb`. The cast is harmless under extractor ORDER BY (the sort
// key is `ore_block_u64_8_256(value)`, structurally distinct from the
// projected `(value)::jsonb`, so there's no folding), but it pushes the
// trap into reach for any future scenario that adds ORDER BY on the column
// itself: projection-pushdown folds the cast into the Sort Key
// (`Sort Key: ((value)::jsonb)`), preventing any index on `eql_v2_encrypted`
// from satisfying the sort. The custom `dbbenches::EqlV2Encrypted` sqlx
// Decode means we can SELECT `value` raw without losing harness typing.
static QUERY_TEMPLATES: &[(&str, i32, &str)] = &[
    // ── Non-selective baselines (≈50% selectivity → Seq Scan + LIMIT) ──
    (
        "SELECT id, value FROM {TABLE} WHERE value > $1 LIMIT 10",
        5000,
        "range_gt_10",
    ),
    (
        "SELECT id, value FROM {TABLE} WHERE value > $1 LIMIT 100",
        5000,
        "range_gt_100",
    ),
    (
        "SELECT id, value FROM {TABLE} WHERE value < $1 LIMIT 10",
        5000,
        "range_lt_10",
    ),
    (
        "SELECT id, value FROM {TABLE} WHERE value < $1 LIMIT 100",
        5000,
        "range_lt_100",
    ),
    // ── Selective predicates (i32 tail) ──
    //
    // DISABLED — `range_selective_gt_100` (threshold 2_140_000_000, ~0.17%
    // selectivity). The planner cannot estimate the selectivity of a
    // selective encrypted ORE range predicate with a bound-parameter
    // operand, falls back to `DEFAULT_INEQ_SEL` (33%), and picks a Seq Scan
    // that scans most of the table — ~2 s per iteration at the 10M tier.
    // Re-enable once EQL issue #230 lands a selectivity fix:
    // https://github.com/cipherstash/encrypt-query-language/issues/230
    // (
    //     "SELECT id, value FROM {TABLE} WHERE value > $1 LIMIT 100",
    //     2_140_000_000,
    //     "range_selective_gt_100",
    // ),
    // DISABLED — `range_highly_selective_gt_10` (threshold 2_147_000_000,
    // ~0.011% selectivity). Same root cause as `range_selective_gt_100`
    // above (EQL issue #230): the planner mis-estimates the selective
    // encrypted predicate and seq-scans most of the table at the 10M tier.
    // (
    //     "SELECT id, value FROM {TABLE} WHERE value > $1 LIMIT 10",
    //     2_147_000_000,
    //     "range_highly_selective_gt_10",
    // ),
    // ── Ordered range (extractor in ORDER BY) ──
    // Sort key matches the functional index expression syntactically, so rows
    // stream out of the index already sorted — no Sort node in the plan.
    //
    // The companion `range_lt_natural_ordered_10` scenario (`ORDER BY value`,
    // no extractor) was removed: Postgres can't structurally match `value`
    // against the functional `ore_block_u64_8_256(value)` index, so the plan
    // falls back to Sort + bitmap/seq scan and runs in seconds (62 s at 10M).
    // That's the deliberate anti-pattern documented in the EQL perf guide
    // (`docs/reference/query-performance.md` §4); a public bench shouldn't
    // ship pathological cases as headline numbers. Also note the
    // projection-pushdown sort-key trap discussed in §4: `SELECT col::jsonb
    // ... ORDER BY col` folds the cast into the sort key, defeating even the
    // direct btree opclass on `eql_v2_encrypted`. This bench projects `value`
    // raw (decoded by the dbbenches `EqlV2Encrypted` sqlx type) to avoid
    // ever conflating the two failure modes.
    (
        "SELECT id, value FROM {TABLE} \
         WHERE value < $1 \
         ORDER BY eql_v2.ore_block_u64_8_256(value) LIMIT 10",
        5000,
        "range_lt_ordered_10",
    ),
];

// Count-style selective scenarios — `SELECT count(*) WHERE value <op>
// threshold`, no LIMIT. Both variants (`range_selective_gt_count`,
// `range_highly_selective_gt_count`) are DISABLED for the same reason as the
// selective LIMIT scenarios above (EQL issue #230): the planner cannot
// estimate the encrypted predicate's selectivity, picks a Seq Scan, and
// scans the whole table — and with no LIMIT to cut it short, a count over
// the 10M tier is the worst case. The array is left empty (rather than the
// count machinery removed) so the scenarios restore by un-commenting the
// tuples once #230 lands a selectivity fix.
static COUNT_QUERY_TEMPLATES: &[(&str, i32, &str)] = &[
    // (
    //     "SELECT count(*) FROM {TABLE} WHERE value > $1",
    //     2_140_000_000,
    //     "range_selective_gt_count",
    // ),
    // (
    //     "SELECT count(*) FROM {TABLE} WHERE value > $1",
    //     2_147_000_000,
    //     "range_highly_selective_gt_count",
    // ),
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

    let target_rows = std::env::var("TARGET_ROWS").unwrap_or_else(|_| "unknown".to_string());

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
                version: 2,
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
                version: 2,
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
