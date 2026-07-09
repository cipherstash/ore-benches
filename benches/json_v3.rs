//! EQL v3 sibling of `benches/json.rs` — encrypted-JSONB (SteVec) queries
//! against `json_ste_vec_small_encrypted_v3_<N>` (`public.json`).
//!
//! Every scenario goes through the **named EQL v3 JSON functions** — the
//! surface a real EQL caller (or the ORM/proxy that rewrites their query)
//! emits — rather than casting the column to raw `jsonb` and using native
//! `@>` / `->`. Scenario ids match the v2 bench so the v2↔v3 comparison
//! joins by id. The SQL surface maps as:
//!
//!   contains/functional
//!     v2: eql_v2.jsonb_array(value) @> eql_v2.jsonb_array($1::…)
//!     v3: eql_v3.jsonb_contains(value, $1::jsonb)
//!     `eql_v3.jsonb_contains(a, b)` inlines to
//!     `eql_v3.jsonb_array(a) @> eql_v3.jsonb_array(b)`, so it engages the
//!     static `GIN (eql_v3.jsonb_array(value))` index (the documented
//!     whole-document containment recipe). Needle = the sampled row's whole
//!     document, normalised to `s` + term entries.
//!
//!   field_eq/bare       eql_v3.jsonb_path_query_first(value, '<sel>') = $1::jsonb::public.jsonb_entry
//!     The "natural" caller form: `jsonb_path_query_first` returns the
//!     `public.jsonb_entry` leaf and `=` on that domain inlines to
//!     `eql_v3.eq_term(a) = eql_v3.eq_term(b)` — structurally identical to
//!     field_eq/functional, so it engages the same per-selector functional
//!     index built at startup.
//!
//!   field_eq/extractor  eql_v3.jsonb_contains(value, $1::jsonb)  (single-entry needle)
//!     Same containment function/index as contains/functional, but the
//!     needle addresses a single selector: `{"sv":[{"s":…,"hm":…}]}`.
//!
//!   field_eq/functional eql_v3.eq_term(eql_v3.jsonb_path_query_first(value, '<sel>'))
//!                          = eql_v3.eq_term($1::jsonb::public.jsonb_entry)
//!     The explicit per-selector functional form.
//!
//! The field_eq needles cast `$1::jsonb::public.jsonb_entry`, not
//! `$1::public.jsonb_entry` — the `::jsonb` intermediate keeps the bound
//! parameter's type the built-in `jsonb`; a direct bind-cast to the domain
//! makes sqlx try to resolve `public.jsonb_entry` as the *parameter* type and
//! fail with `type "public.jsonb_entry" does not exist`. Don't drop it.
//!
//!   field_order/functional
//!     ORDER BY eql_v3.ore_cllw(eql_v3.jsonb_path_query_first(value, '<sel>')) LIMIT 10
//!     `eql_v3.ore_cllw` returns `eql_v3_internal.ore_cllw`, whose
//!     DEFAULT-FOR-TYPE btree opclass turns `ORDER BY … LIMIT` into an index
//!     scan on the per-selector functional index.
//!
//!   field_gt/functional  — the encrypted equivalent of `x -> 'y' > 10`
//!     WHERE eql_v3.jsonb_path_query_first(value, '<sel>') > $1::jsonb::public.jsonb_entry LIMIT 10
//!     The `>` operator on public.jsonb_entry inlines to
//!     `eql_v3.ore_cllw(a) > eql_v3.ore_cllw(b)` (a CLLW-ORE comparison), so
//!     the predicate reuses the same `field_order_idx` — no extra index. The
//!     threshold `$1` is a sampled oc leaf; an unselective bound may still
//!     seq-scan (mirrors the scalar ORE `range_gt` scenarios).
//!
//! Field access is always `eql_v3.jsonb_path_query_first(value, '<sel>')`
//! (the scalar EQL path-query — `jsonb_path_query` is SETOF and can't sit in
//! an index expression), never a raw `value -> '<sel>'`. The selector is the
//! sv element's deterministic `s` hash, sampled at startup.
//!
//! Per-selector functional indexes (eq_term / ore_cllw over
//! `jsonb_path_query_first(value, sel)`) are built at startup once the
//! selector is sampled, mirroring the v2 bench's create_field_indexes.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{
    bench_assert, extract_indexes_used, init_tracing, write_metadata_file_in, ScenarioMetadata,
};
use serde_json::Value as JsonValue;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use sqlx::Row;
use tokio::runtime::Runtime;

#[derive(Debug)]
struct Needles {
    /// `{"sv":[{"s":..,"hm"|"oc":..}, ...]}` — the whole sampled document
    /// normalized to the containment-needle shape (drives contains/functional).
    document_query: String,
    hm_pick: Option<HmPick>,
    ore_pick: Option<OrePick>,
}

#[derive(Debug)]
struct HmPick {
    selector: String,
    /// `eql_v3.jsonb_path_query_first(value, '<selector>')::jsonb` of the
    /// sampled row — the merged leaf entry, castable to public.jsonb_entry.
    /// Drives the field_eq/bare + field_eq/functional needles.
    sample_field_value: JsonValue,
    /// Single-entry containment needle for field_eq/extractor.
    hmac_term: String,
}

#[derive(Debug)]
struct OrePick {
    selector: String,
    /// `eql_v3.jsonb_path_query_first(value, '<selector>')::jsonb` of a sampled
    /// row — an oc-bearing leaf used as the range threshold for field_gt
    /// (`field > this`), castable to public.jsonb_entry.
    threshold: JsonValue,
}

async fn sample_needles(pool: &sqlx::PgPool, table: &str) -> Needles {
    // v3 payloads are the public.json jsonb domain — cast to raw jsonb for
    // *sampling* (setup, not a measured query) and walk `-> 'sv'` directly.
    let rows = sqlx::query(&format!(
        "SELECT elem ->> 's'  AS sel,
                elem ->> 'hm' AS hmac,
                elem          AS sv_elem
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

    // Normalize every entry to `s` + one term for the whole-document
    // containment needle (what eql_v3.jsonb_array reduces the column to).
    let mut query_entries: Vec<JsonValue> = Vec::with_capacity(rows.len());
    let mut hm_pick: Option<HmPick> = None;
    let mut ore_pick: Option<OrePick> = None;

    for row in &rows {
        let sel: String = row.get("sel");
        let hmac: Option<String> = row.get("hmac");
        let sv_elem: Json<JsonValue> = row.get("sv_elem");
        let obj = sv_elem.0.as_object().expect("sv element is an object");

        if let Some(h) = obj.get("hm") {
            query_entries.push(serde_json::json!({"s": sel, "hm": h}));
        } else if let Some(oc) = obj.get("oc") {
            query_entries.push(serde_json::json!({"s": sel, "oc": oc}));
        }

        if hm_pick.is_none() {
            if let Some(h) = hmac.as_deref() {
                // Source the leaf needle through the same EQL path-query the
                // measured queries use, so needle and column leaf agree
                // byte-for-byte under eq_term (guarantees ≥1 match).
                let field_row = sqlx::query(&format!(
                    "SELECT eql_v3.jsonb_path_query_first(value::jsonb, '{sel}')::jsonb
                              AS sample_field_value
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
                    hmac_term: format!(r#"{{"sv":[{{"s":"{}","hm":"{}"}}]}}"#, sel, h),
                });
            }
        }

        if ore_pick.is_none() && obj.contains_key("oc") {
            // Sample this row's oc leaf through the same EQL path-query the
            // measured queries use, as the field_gt range threshold.
            let field_row = sqlx::query(&format!(
                "SELECT eql_v3.jsonb_path_query_first(value::jsonb, '{sel}')::jsonb
                          AS threshold
                 FROM {table}
                 LIMIT 1"
            ))
            .fetch_one(pool)
            .await
            .expect("query for ore threshold value failed");
            let threshold: Json<JsonValue> = field_row.get("threshold");
            ore_pick = Some(OrePick {
                selector: sel.clone(),
                threshold: threshold.0,
            });
        }
    }

    Needles {
        document_query: serde_json::to_string(&serde_json::json!({"sv": query_entries}))
            .expect("serialise document query needle"),
        hm_pick,
        ore_pick,
    }
}

/// Build per-selector functional indexes (selector known only after
/// sampling). btree for both — see the v2 json bench for the hash-vs-btree
/// build-cost rationale. Both index the EQL path-query expression
/// `eql_v3.jsonb_path_query_first(value, '<sel>')` so the field_eq/field_order
/// queries (which call the same function) match.
async fn create_field_indexes(pool: &sqlx::PgPool, table: &str, needles: &Needles) {
    eprintln!("json_v3 bench: building per-selector functional indexes...");

    if let Some(p) = needles.hm_pick.as_ref() {
        sqlx::query(&format!("DROP INDEX IF EXISTS {table}_field_eq_idx"))
            .execute(pool)
            .await
            .expect("drop stale field_eq index");
        sqlx::query(&format!(
            "CREATE INDEX {table}_field_eq_idx ON {table} \
             USING btree (eql_v3.eq_term(eql_v3.jsonb_path_query_first(value, '{}')))",
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
             USING btree (eql_v3.ore_cllw(eql_v3.jsonb_path_query_first(value, '{}')))",
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
    init_tracing();
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

    let hm_field_value_json = needles
        .hm_pick
        .as_ref()
        .map(|p| serde_json::to_string(&p.sample_field_value).expect("serialise hm field value"));

    // --- Query strings ---
    // Every predicate is a named EQL v3 JSON function. `value` (public.json)
    // coerces to the functions' `jsonb` parameter via the domain's base type.

    let q_contains_functional = format!(
        "SELECT id FROM {table_name} \
         WHERE eql_v3.jsonb_contains(value, $1::jsonb) LIMIT 10"
    );

    let q_field_eq_bare = needles.hm_pick.as_ref().map(|p| {
        let selector = &p.selector;
        format!(
            "SELECT id FROM {table_name} \
             WHERE eql_v3.jsonb_path_query_first(value, '{selector}') \
                 = $1::jsonb::public.jsonb_entry LIMIT 10"
        )
    });

    let q_field_eq_extractor = needles.hm_pick.as_ref().map(|_| {
        format!(
            "SELECT id FROM {table_name} \
             WHERE eql_v3.jsonb_contains(value, $1::jsonb) LIMIT 10"
        )
    });

    let q_field_eq_functional = needles.hm_pick.as_ref().map(|p| {
        let selector = &p.selector;
        format!(
            "SELECT id FROM {table_name} \
             WHERE eql_v3.eq_term(eql_v3.jsonb_path_query_first(value, '{selector}')) \
                 = eql_v3.eq_term($1::jsonb::public.jsonb_entry) LIMIT 10"
        )
    });

    let q_field_order_functional = needles.ore_pick.as_ref().map(|p| {
        let selector = &p.selector;
        format!(
            "SELECT id FROM {table_name} \
             ORDER BY eql_v3.ore_cllw(eql_v3.jsonb_path_query_first(value, '{selector}')) LIMIT 10"
        )
    });

    // Encrypted-JSON field range — the equivalent of `x -> 'y' > 10`. The
    // `>` operator on public.jsonb_entry inlines to
    // `eql_v3.ore_cllw(a) > eql_v3.ore_cllw(b)`, so the LHS matches the same
    // `field_order_idx` (ore_cllw over jsonb_path_query_first) — no extra
    // index. Threshold needle is a sampled oc leaf; the planner may still
    // prefer a seq scan when the bound is unselective (see `indexes_used`).
    let q_field_gt_functional = needles.ore_pick.as_ref().map(|p| {
        let selector = &p.selector;
        format!(
            "SELECT id FROM {table_name} \
             WHERE eql_v3.jsonb_path_query_first(value, '{selector}') \
                 > $1::jsonb::public.jsonb_entry LIMIT 10"
        )
    });

    let ore_threshold_json = needles
        .ore_pick
        .as_ref()
        .map(|p| serde_json::to_string(&p.threshold).expect("serialise ore threshold"));

    // --- Metadata sidecar ---

    let has_hm = needles.hm_pick.is_some();
    let has_ore = needles.ore_pick.is_some();

    let metadata = rt.block_on(async {
        let mut out: Vec<ScenarioMetadata> = Vec::with_capacity(6);

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
            }
        }

        out.push(
            capture(
                &pool,
                format!("JSON/json/contains/functional/{}", target_rows),
                &q_contains_functional,
                Some(&needles.document_query),
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
                    format!("JSON/json/field_eq/bare/{}", target_rows),
                    q_bare,
                    Some(hm_field),
                )
                .await,
            );
            out.push(
                capture(
                    &pool,
                    format!("JSON/json/field_eq/extractor/{}", target_rows),
                    q_extractor,
                    Some(hm_pick.hmac_term.as_str()),
                )
                .await,
            );
            out.push(
                capture(
                    &pool,
                    format!("JSON/json/field_eq/functional/{}", target_rows),
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

        if let Some(q) = q_field_order_functional.as_deref() {
            out.push(
                capture(
                    &pool,
                    format!("JSON/json/field_order/functional/{}", target_rows),
                    q,
                    None,
                )
                .await,
            );
        } else {
            eprintln!(
                "json_v3 bench: skipping field_order/functional — no sv element on the \
                 sampled row carries `oc`."
            );
        }

        if let (Some(q), Some(threshold)) =
            (q_field_gt_functional.as_deref(), ore_threshold_json.as_deref())
        {
            out.push(
                capture(
                    &pool,
                    format!("JSON/json/field_gt/functional/{}", target_rows),
                    q,
                    Some(threshold),
                )
                .await,
            );
        }

        out
    });
    write_metadata_file_in("results/query/v3", "json", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    // --- Bench loop ---

    let mut group = c.benchmark_group("JSON");
    group.sample_size(10);

    {
        let id = format!("JSON/json/contains/functional/{}", target_rows);
        let q = q_contains_functional.clone();
        let needle = needles.document_query.clone();
        group.bench_function(format!("json/contains/functional/{}", target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let rows =
                    bench_assert(sqlx::query(&q).bind(&needle).fetch_all(&pool).await, &id);
                black_box(rows.len())
            })
        });
    }

    if has_hm {
        let hm_field = hm_field_value_json
            .clone()
            .expect("hm_field_value_json present when has_hm");
        let hmac_term = needles
            .hm_pick
            .as_ref()
            .expect("hm_pick present when has_hm")
            .hmac_term
            .clone();

        if let Some(q) = q_field_eq_bare.clone() {
            let id = format!("JSON/json/field_eq/bare/{}", target_rows);
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
            let id = format!("JSON/json/field_eq/extractor/{}", target_rows);
            let needle = hmac_term.clone();
            group.bench_function(format!("json/field_eq/extractor/{}", target_rows), |b| {
                b.to_async(&rt).iter(|| async {
                    let rows =
                        bench_assert(sqlx::query(&q).bind(&needle).fetch_all(&pool).await, &id);
                    black_box(rows.len())
                })
            });
        }

        if let Some(q) = q_field_eq_functional.clone() {
            let id = format!("JSON/json/field_eq/functional/{}", target_rows);
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
            let id = format!("JSON/json/field_order/functional/{}", target_rows);
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

        if let (Some(q), Some(threshold)) =
            (q_field_gt_functional.clone(), ore_threshold_json.clone())
        {
            let id = format!("JSON/json/field_gt/functional/{}", target_rows);
            group.bench_function(format!("json/field_gt/functional/{}", target_rows), |b| {
                b.to_async(&rt).iter(|| async {
                    let rows =
                        bench_assert(sqlx::query(&q).bind(&threshold).fetch_all(&pool).await, &id);
                    black_box(rows.len())
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
