//! EQL v3 twin of `benches/ore.rs` — range + ordered-range queries against
//! `integer_encrypted_v3_<N>` (column typed `eql_v3.integer_ord_ore`),
//! plus the v3-only OPE ordering scenarios against
//! `integer_ope_encrypted_v3_<N>` (column typed `eql_v3.integer_ord_ope`).
//!
//! Scenario parity with v2:
//!
//!   * The four non-selective range baselines (`range_gt_10/100`,
//!     `range_lt_10/100`) carry over unchanged in intent — bare-form
//!     `value <op> $1::eql_v3.integer_ord_ore` inlines to
//!     `eql_v3.ord_term(a) <op> eql_v3.ord_term(b)` and structurally
//!     matches the `btree (eql_v3.ord_term(value))` index. Whether the
//!     planner uses it is a selectivity question, exactly as in v2 (see
//!     the long discussion in benches/ore.rs).
//!   * `range_lt_ordered_10` keeps the extractor ORDER BY
//!     (`ORDER BY eql_v3.ord_term(value)`) so rows stream out of the index
//!     already sorted. The natural-form `ORDER BY value` anti-pattern
//!     stays excluded, as in v2.
//!   * The selective scenarios remain disabled for the same planner
//!     selectivity mis-estimate (EQL issue #230) — the bound-parameter
//!     operand hides selectivity from the planner in v3 exactly as in v2.
//!
//! Probe flow: the threshold (5000) is encrypted as a STORAGE payload and
//! converted (target `integer_ord_ore`) because no v3 scalar QUERY wire shape
//! exists — see benches/exact_v3.rs.
//!
//! `_ord_ope` scenarios (CIP-3348; was the CIP-3280 stub): eql_v3 ships
//! `integer_ord_ope` (wire key `op`, extractor `eql_v3.ord_ope_term`,
//! ordering by native bytea comparison of the OPE-CLLW ciphertext — no
//! per-row plpgsql compare). cipherstash-client 0.38.1 emits the scalar
//! `op` term (CIP-3280), so OPE_QUERY_TEMPLATES below run against the
//! `integer_ope_encrypted_v3_<N>` tables (populated by the
//! encrypt_int_ope_v3 ingest bin) over a
//! `btree (eql_v3.ord_ope_term(value))` index. The `ope_*` scenarios keep
//! the ORE thresholds so the two ordering implementations line up
//! side-by-side in the reports.

use cipherstash_client::{
    encryption::ScopedCipher,
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
    AutoStrategy,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{
    bench_assert, extract_indexes_used, init_scoped_cipher, init_tracing,
    v3::{EncryptedQueryBuilderV3, EncryptedQueryV3, TargetDomain},
    write_metadata_file, ScenarioMetadata,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::runtime::Runtime;

// (sql_template, probe_value, scenario_name). Thresholds and scenario names
// mirror benches/ore.rs so the reporters line the versions up.
static QUERY_TEMPLATES: &[(&str, i32, &str)] = &[
    (
        "SELECT id, value::jsonb FROM {TABLE} \
         WHERE value > $1::eql_v3.integer_ord_ore LIMIT 10",
        5000,
        "range_gt_10",
    ),
    (
        "SELECT id, value::jsonb FROM {TABLE} \
         WHERE value > $1::eql_v3.integer_ord_ore LIMIT 100",
        5000,
        "range_gt_100",
    ),
    (
        "SELECT id, value::jsonb FROM {TABLE} \
         WHERE value < $1::eql_v3.integer_ord_ore LIMIT 10",
        5000,
        "range_lt_10",
    ),
    (
        "SELECT id, value::jsonb FROM {TABLE} \
         WHERE value < $1::eql_v3.integer_ord_ore LIMIT 100",
        5000,
        "range_lt_100",
    ),
    (
        "SELECT id, value::jsonb FROM {TABLE} \
         WHERE value < $1::eql_v3.integer_ord_ore \
         ORDER BY eql_v3.ord_term(value) LIMIT 10",
        5000,
        "range_lt_ordered_10",
    ),
];

// OPE twins of the two headline ORE shapes: the non-selective range
// baseline and the extractor-ordered stream. Same thresholds, `ope_`
// scenario-name prefix, run against the `integer_ope_encrypted_v3*`
// tables ({OPE_TABLE}).
static OPE_QUERY_TEMPLATES: &[(&str, i32, &str)] = &[
    (
        "SELECT id, value::jsonb FROM {OPE_TABLE} \
         WHERE value > $1::eql_v3.integer_ord_ope LIMIT 10",
        5000,
        "ope_range_gt_10",
    ),
    (
        "SELECT id, value::jsonb FROM {OPE_TABLE} \
         WHERE value < $1::eql_v3.integer_ord_ope \
         ORDER BY eql_v3.ord_ope_term(value) LIMIT 10",
        5000,
        "ope_range_lt_ordered_10",
    ),
];

/// Which ordering implementation a scenario drives — selects the client
/// index config (so the probe carries the right term) and the v3 target
/// domain the storage payload converts into.
#[derive(Clone, Copy)]
enum Ordering {
    Ore,
    Ope,
}

impl Ordering {
    fn index(self) -> Index {
        match self {
            Self::Ore => Index::new_ore(),
            Self::Ope => Index::new_ope(),
        }
    }

    fn target(self) -> TargetDomain {
        let name = match self {
            Self::Ore => "integer_ord_ore",
            Self::Ope => "integer_ord_ope",
        };
        TargetDomain::parse(name).expect("known v3 ordering domain")
    }
}

async fn build_query(
    cipher: Arc<ScopedCipher<AutoStrategy>>,
    query: &str,
    x: i32,
    table_name: &str,
    ordering: Ordering,
) -> EncryptedQueryV3 {
    let column_config = ColumnConfig::build("value")
        .casts_as(ColumnType::Int)
        .add_index(ordering.index());

    let identifier = Identifier::new(table_name, "value");

    EncryptedQueryBuilderV3::new(column_config, identifier, ordering.target())
        .statement(query)
        .build_query(x, cipher)
        .await
        .expect("Failed to build encrypted v3 query")
}

fn criterion_benchmark(c: &mut Criterion) {
    init_tracing();

    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS").unwrap_or_else(|_| "unknown".to_string());

    let table_suffix = match target_rows.as_str() {
        "10000" | "100000" | "1000000" | "10000000" => format!("_{}", target_rows),
        _ => String::new(),
    };
    let table_name = format!("integer_encrypted_v3{}", table_suffix);
    let ope_table_name = format!("integer_ope_encrypted_v3{}", table_suffix);

    // (bound sql, probe, scenario, ordering) — the ORE templates followed
    // by their OPE twins, all in the one ORE_V3 criterion group so the
    // reports show the two ordering implementations side-by-side.
    let scenarios: Vec<(String, i32, &str, Ordering)> = QUERY_TEMPLATES
        .iter()
        .map(|(tpl, x, scenario)| {
            (
                tpl.replace("{TABLE}", &table_name),
                *x,
                *scenario,
                Ordering::Ore,
            )
        })
        .chain(OPE_QUERY_TEMPLATES.iter().map(|(tpl, x, scenario)| {
            (
                tpl.replace("{OPE_TABLE}", &ope_table_name),
                *x,
                *scenario,
                Ordering::Ope,
            )
        }))
        .collect();

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
        let mut queries = Vec::with_capacity(scenarios.len());
        for (query_str, x, _, ordering) in &scenarios {
            let query_table = match ordering {
                Ordering::Ore => &table_name,
                Ordering::Ope => &ope_table_name,
            };
            let query =
                build_query(Arc::clone(&cipher), query_str, *x, query_table, *ordering).await;
            queries.push(query);
        }
        queries
    });

    let metadata = rt.block_on(async {
        let mut out = Vec::with_capacity(queries.len());
        for (i, query) in queries.iter().enumerate() {
            let (_, _, scenario, _) = &scenarios[i];
            let bench_id = format!("ORE_V3/ore/{}/{}", scenario, target_rows);
            let explain = query.explain(&pool).await.expect("EXPLAIN failed");
            let indexes_used = extract_indexes_used(&explain);
            let parameters = vec![query.parameter_json()];
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
                version: 3,
            });
        }
        out
    });
    write_metadata_file("ore_v3", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("ORE_V3");
    group.sample_size(10);

    for (i, query) in queries.into_iter().enumerate() {
        let (_, _, scenario, _) = &scenarios[i];
        let exec_id = format!("ORE_V3/ore/{}/{}", scenario, target_rows);
        let decrypt_id = format!("ORE_V3/ore_decrypt/{}/{}", scenario, target_rows);

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

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
