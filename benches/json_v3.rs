//! EQL v3 twin of `benches/json.rs` — JSON / ste_vec query benches against
//! `json_ste_vec_small_encrypted_v3_<N>` (column typed `eql_v3.json`).
//!
//! Scenario mapping from v2 (same names, v3 recipes):
//!
//!   1. `contains/functional` — whole-document containment. v2 used
//!      `eql_v2.jsonb_array(a) @> eql_v2.jsonb_array(b)`; the canonical v3
//!      recipe is the typed needle form
//!      `WHERE value @> $1::jsonb::eql_v3.jsonb_query`, which inlines to
//!      a native `jsonb @>` over
//!      `eql_v3.to_ste_vec_query(value)::jsonb` and engages the single
//!      `GIN ((to_ste_vec_query(value))::jsonb jsonb_path_ops)` index. The
//!      needle is the sampled row's own normalized query shape
//!      (`SELECT eql_v3.to_ste_vec_query(value)`), so it matches at least
//!      the source row.
//!
//!   2. `field_eq/bare` — `(value -> '<sel>'::text) = $1::jsonb::eql_v3.jsonb_entry`.
//!      Unlike v2 (where `->` was plpgsql and unmatchable), the v3 `->` and
//!      `=` are inlinable SQL: the predicate reduces to
//!      `eql_v3.eq_term(value -> '<sel>') = eql_v3.eq_term($1)` and engages
//!      the per-selector btree built at startup (see create_field_indexes).
//!      `field_eq/extractor` — single-field needle `{"sv":[{s,hm}]}`
//!      through the same `@> $1::jsonb::eql_v3.jsonb_query` GIN recipe as
//!      containment (one index covers every selector, hm- and oc-bearing).
//!      `field_eq/functional` — the explicit extractor form of bare.
//!
//!   3. `field_order/functional` — `ORDER BY eql_v3.ore_cllw(value ->
//!      '<sel>'::text) LIMIT 10`, engaging a per-selector btree (the
//!      `eql_v3.ore_cllw_ops` opclass is DEFAULT for the type). No bare
//!      ORDER BY scenario, same reasoning as v2: sort keys are never
//!      rewritten by the planner.
//!
//! The needle picker mirrors v2: one `hm`-bearing sv element drives
//! field_eq/*, one orderable element drives field_order/*. In v3 the only
//! per-entry orderable tag is `oc` (ORE CLLW) — the root-scalar `ob` tag
//! does not exist at sv-entry level (the from_v2 conversion enforces the
//! `hm` XOR `oc` entry contract).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{bench_assert, extract_indexes_used, write_metadata_file, ScenarioMetadata};
use serde_json::Value as JsonValue;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use sqlx::Row;
use tokio::runtime::Runtime;

/// Sampled needles, picked once at startup from the target v3 table.
#[derive(Debug)]
struct Needles {
    /// The sampled row's normalized containment needle
    /// (`eql_v3.to_ste_vec_query(value)::jsonb`) — every sv element reduced
    /// to `s` + its `hm`/`oc` term.
    containment_needle: JsonValue,
    /// Selector + payloads for the field_eq/* scenarios (None when no sv
    /// element carries `hm`).
    hm_pick: Option<HmPick>,
    /// Selector for the field_order/* scenarios (None when no sv element
    /// carries `oc`).
    ore_pick: Option<OrePick>,
}

#[derive(Debug)]
struct HmPick {
    /// Deterministic selector hash (inlined into SQL — the planner needs a
    /// literal for functional-index matching).
    selector: String,
    /// `(value -> '<selector>')::jsonb` — an entry payload for the
    /// field_eq bare / functional needles (`::eql_v3.jsonb_entry`).
    sample_field_value: JsonValue,
    /// `{"sv":[{"s":"<sel>","hm":"<hash>"}]}` — the field_eq/extractor
    /// needle (`::eql_v3.jsonb_query`).
    hmac_needle: String,
}

#[derive(Debug)]
struct OrePick {
    /// Deterministic selector hash for the `oc`-bearing field.
    selector: String,
}

async fn sample_needles(pool: &sqlx::PgPool, table: &str) -> Needles {
    // The containment needle is sampled pre-normalized via the same
    // to_ste_vec_query the GIN index and @> operator use.
    let needle_row = sqlx::query(&format!(
        "SELECT eql_v3.to_ste_vec_query(value)::jsonb AS needle FROM {table} LIMIT 1"
    ))
    .fetch_one(pool)
    .await
    .expect("containment needle sample failed — is the table populated?");
    let containment_needle: Json<JsonValue> = needle_row.get("needle");

    // Scan the first row's sv array for one hm-bearing and one oc-bearing
    // element. `value::jsonb` downcasts the eql_v3.json domain so the
    // native jsonb operators apply (the v3 `->` selector operator would
    // otherwise capture a typed text RHS).
    let rows = sqlx::query(&format!(
        "SELECT elem ->> 's'  AS sel,
                elem ->> 'hm' AS hmac,
                elem ->> 'oc' AS oc,
                ord
         FROM (
           SELECT value::jsonb -> 'sv' AS sv_array
           FROM {table}
           LIMIT 1
         ) source,
         LATERAL jsonb_array_elements(sv_array) WITH ORDINALITY AS j(elem, ord)
         ORDER BY ord"
    ))
    .fetch_all(pool)
    .await
    .expect("query for sv elements failed");

    if rows.is_empty() {
        panic!("table `{table}` is empty");
    }

    let mut hm_pick: Option<HmPick> = None;
    let mut ore_pick: Option<OrePick> = None;

    for row in &rows {
        let sel: String = row.get("sel");
        let hmac: Option<String> = row.get("hmac");
        let oc: Option<String> = row.get("oc");

        if hm_pick.is_none() {
            if let Some(h) = hmac {
                // `->` with a typed text selector resolves the v3 operator
                // and returns the matching entry (root meta merged in);
                // cast to jsonb for decoding.
                let field_row = sqlx::query(&format!(
                    "SELECT (value -> '{sel}'::text)::jsonb AS sample_field_value
                     FROM {table}
                     LIMIT 1"
                ))
                .fetch_one(pool)
                .await
                .expect("query for hm sample field value failed");
                let sample_field_value: Json<JsonValue> = field_row.get("sample_field_value");
                hm_pick = Some(HmPick {
                    selector: sel.clone(),
                    sample_field_value: sample_field_value.0,
                    hmac_needle: format!(r#"{{"sv":[{{"s":"{}","hm":"{}"}}]}}"#, sel, h),
                });
            }
        }

        if ore_pick.is_none() && oc.is_some() {
            ore_pick = Some(OrePick { selector: sel });
        }

        if hm_pick.is_some() && ore_pick.is_some() {
            break;
        }
    }

    Needles {
        containment_needle: containment_needle.0,
        hm_pick,
        ore_pick,
    }
}

/// Build the per-selector functional indexes the `field_eq/*` and
/// `field_order/functional` scenarios measure. Per-selector expressions
/// embed the sampled selector hash, so — as in the v2 bench — they cannot
/// live in the static sql/indexes/v3/*.sql files.
async fn create_field_indexes(pool: &sqlx::PgPool, table: &str, needles: &Needles) {
    eprintln!("json_v3 bench: building per-selector functional indexes...");

    if let Some(p) = needles.hm_pick.as_ref() {
        sqlx::query(&format!("DROP INDEX IF EXISTS {table}_field_eq_idx"))
            .execute(pool)
            .await
            .expect("drop stale field_eq index");
        // btree, not hash — same build-scalability reasoning as the v2
        // bench (hash index builds degrade badly at the 10M tier).
        sqlx::query(&format!(
            "CREATE INDEX {table}_field_eq_idx ON {table} \
             USING btree (eql_v3.eq_term(value -> '{}'::text))",
            p.selector
        ))
        .execute(pool)
        .await
        .expect("create field_eq functional index");
    }

    if let Some(p) = needles.ore_pick.as_ref() {
        sqlx::query(&format!("DROP INDEX IF EXISTS {table}_field_order_idx"))
            .execute(pool)
            .await
            .expect("drop stale field_order index");
        sqlx::query(&format!(
            "CREATE INDEX {table}_field_order_idx ON {table} \
             USING btree (eql_v3.ore_cllw(value -> '{}'::text))",
            p.selector
        ))
        .execute(pool)
        .await
        .expect("create field_order functional index");
    }

    sqlx::query(&format!("ANALYZE {table}"))
        .execute(pool)
        .await
        .expect("ANALYZE after index creation");
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS").unwrap_or_else(|_| "unknown".to_string());

    let table_suffix = match target_rows.as_str() {
        "10000" | "100000" | "1000000" | "10000000" => format!("_{}", target_rows),
        _ => String::new(),
    };
    let table_name = format!("json_ste_vec_small_encrypted_v3{}", table_suffix);

    let (pool, needles) = rt.block_on(async {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let needles = sample_needles(&pool, &table_name).await;
        eprintln!(
            "json_v3 bench picked from `{}` — hm: {} | oc: {}",
            &table_name,
            needles
                .hm_pick
                .as_ref()
                .map(|p| p.selector.as_str())
                .unwrap_or("<none>"),
            needles
                .ore_pick
                .as_ref()
                .map(|p| p.selector.as_str())
                .unwrap_or("<none>"),
        );

        create_field_indexes(&pool, &table_name, &needles).await;

        (pool, needles)
    });

    // Needles bind as text ($1) and are cast in SQL
    // (::jsonb::eql_v3.jsonb_query / ::jsonb::eql_v3.jsonb_entry).
    let containment_needle_json =
        serde_json::to_string(&needles.containment_needle).expect("serialise containment needle");
    let hm_field_value_json = needles
        .hm_pick
        .as_ref()
        .map(|p| serde_json::to_string(&p.sample_field_value).expect("serialise hm field value"));

    // --- Query strings ---

    let q_contains_functional = format!(
        "SELECT id FROM {table_name} \
         WHERE value @> $1::jsonb::eql_v3.jsonb_query LIMIT 10"
    );

    let q_field_eq_bare = needles.hm_pick.as_ref().map(|p| {
        let selector = &p.selector;
        format!(
            "SELECT id FROM {table_name} \
             WHERE (value -> '{selector}'::text) = $1::jsonb::eql_v3.jsonb_entry LIMIT 10"
        )
    });

    // Same SQL shape as contains/functional — the scenario differs by
    // needle (single {s,hm} element vs the whole document's terms).
    let q_field_eq_extractor = needles.hm_pick.as_ref().map(|_| {
        format!(
            "SELECT id FROM {table_name} \
             WHERE value @> $1::jsonb::eql_v3.jsonb_query LIMIT 10"
        )
    });

    let q_field_eq_functional = needles.hm_pick.as_ref().map(|p| {
        let selector = &p.selector;
        format!(
            "SELECT id FROM {table_name} \
             WHERE eql_v3.eq_term(value -> '{selector}'::text) \
                 = eql_v3.eq_term($1::jsonb::eql_v3.jsonb_entry) LIMIT 10"
        )
    });

    let q_field_order_functional = needles.ore_pick.as_ref().map(|p| {
        let selector = &p.selector;
        format!(
            "SELECT id FROM {table_name} \
             ORDER BY eql_v3.ore_cllw(value -> '{selector}'::text) LIMIT 10"
        )
    });

    // --- Metadata sidecar ---

    let has_hm = needles.hm_pick.is_some();
    let has_ore = needles.ore_pick.is_some();

    let metadata = rt.block_on(async {
        let mut out: Vec<ScenarioMetadata> = Vec::with_capacity(5);

        async fn capture(
            pool: &sqlx::PgPool,
            id: String,
            query: &str,
            bind: Option<&str>,
        ) -> ScenarioMetadata {
            let explain_sql = format!("EXPLAIN (FORMAT JSON) {}", query);
            let (Json(explain),): (Json<JsonValue>,) = if let Some(b) = bind {
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
            let indexes_used = extract_indexes_used(&explain);

            let rows: Vec<sqlx::postgres::PgRow> = if let Some(b) = bind {
                sqlx::query(query)
                    .bind(b)
                    .fetch_all(pool)
                    .await
                    .expect("row-count execute failed")
            } else {
                sqlx::query(query)
                    .fetch_all(pool)
                    .await
                    .expect("row-count execute failed")
            };

            let parameters = bind
                .map(|b| vec![JsonValue::String(b.to_string())])
                .unwrap_or_default();

            ScenarioMetadata {
                id,
                query: query.to_string(),
                parameters,
                explain,
                indexes_used,
                rows_returned: rows.len() as u64,
                version: 3,
            }
        }

        out.push(
            capture(
                &pool,
                format!("JSON_V3/json/contains/functional/{}", target_rows),
                &q_contains_functional,
                Some(&containment_needle_json),
            )
            .await,
        );

        if let (Some(hm_pick), Some(q_bare), Some(q_extractor), Some(q_functional)) = (
            needles.hm_pick.as_ref(),
            q_field_eq_bare.as_deref(),
            q_field_eq_extractor.as_deref(),
            q_field_eq_functional.as_deref(),
        ) {
            let hm_field = hm_field_value_json
                .as_deref()
                .expect("hm_field_value_json present when hm_pick is Some");

            out.push(
                capture(
                    &pool,
                    format!("JSON_V3/json/field_eq/bare/{}", target_rows),
                    q_bare,
                    Some(hm_field),
                )
                .await,
            );

            out.push(
                capture(
                    &pool,
                    format!("JSON_V3/json/field_eq/extractor/{}", target_rows),
                    q_extractor,
                    Some(hm_pick.hmac_needle.as_str()),
                )
                .await,
            );

            out.push(
                capture(
                    &pool,
                    format!("JSON_V3/json/field_eq/functional/{}", target_rows),
                    q_functional,
                    Some(hm_field),
                )
                .await,
            );
        } else {
            eprintln!(
                "json_v3 bench: skipping field_eq/* scenarios — no sv element on the \
                 sampled row carries `hm`."
            );
        }

        if has_ore {
            if let Some(q) = q_field_order_functional.as_deref() {
                out.push(
                    capture(
                        &pool,
                        format!("JSON_V3/json/field_order/functional/{}", target_rows),
                        q,
                        None,
                    )
                    .await,
                );
            }
        } else {
            eprintln!(
                "json_v3 bench: skipping field_order/functional — no sv element on the \
                 sampled row carries `oc`."
            );
        }

        out
    });
    write_metadata_file("json_v3", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    // --- Bench loop ---

    let mut group = c.benchmark_group("JSON_V3");
    group.sample_size(10);

    {
        let id = format!("JSON_V3/json/contains/functional/{}", target_rows);
        let q = q_contains_functional.clone();
        let needle = containment_needle_json.clone();
        group.bench_function(format!("json/contains/functional/{}", target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let rows = bench_assert(sqlx::query(&q).bind(&needle).fetch_all(&pool).await, &id);
                black_box(rows.len())
            })
        });
    }

    if has_hm {
        let hm_field = hm_field_value_json
            .clone()
            .expect("hm_field_value_json present when has_hm");
        let hmac_needle = needles
            .hm_pick
            .as_ref()
            .expect("hm_pick present when has_hm")
            .hmac_needle
            .clone();

        if let Some(q) = q_field_eq_bare.clone() {
            let id = format!("JSON_V3/json/field_eq/bare/{}", target_rows);
            let needle = hm_field.clone();
            group.bench_function(format!("json/field_eq/bare/{}", target_rows), |b| {
                b.to_async(&rt).iter(|| async {
                    let rows =
                        bench_assert(sqlx::query(&q).bind(&needle).fetch_all(&pool).await, &id);
                    black_box(rows.len())
                })
            });
        }

        if let Some(q) = q_field_eq_extractor.clone() {
            let id = format!("JSON_V3/json/field_eq/extractor/{}", target_rows);
            let needle = hmac_needle.clone();
            group.bench_function(format!("json/field_eq/extractor/{}", target_rows), |b| {
                b.to_async(&rt).iter(|| async {
                    let rows =
                        bench_assert(sqlx::query(&q).bind(&needle).fetch_all(&pool).await, &id);
                    black_box(rows.len())
                })
            });
        }

        if let Some(q) = q_field_eq_functional.clone() {
            let id = format!("JSON_V3/json/field_eq/functional/{}", target_rows);
            let needle = hm_field.clone();
            group.bench_function(format!("json/field_eq/functional/{}", target_rows), |b| {
                b.to_async(&rt).iter(|| async {
                    let rows =
                        bench_assert(sqlx::query(&q).bind(&needle).fetch_all(&pool).await, &id);
                    black_box(rows.len())
                })
            });
        }
    }

    if has_ore {
        if let Some(q) = q_field_order_functional.clone() {
            let id = format!("JSON_V3/json/field_order/functional/{}", target_rows);
            group.bench_function(
                format!("json/field_order/functional/{}", target_rows),
                |b| {
                    b.to_async(&rt).iter(|| async {
                        let rows = bench_assert(sqlx::query(&q).fetch_all(&pool).await, &id);
                        black_box(rows.len())
                    })
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
