// JSON / ste_vec query benches.
//
// Three query pattern families that map to the patterns a JS / ORM caller
// would write against an encrypted JSON column, plus the recipe forms a
// human author would write knowing the EQL extractor functions.
//
//   1. JSON containment (whole-document)
//      contains/functional
//        WHERE eql_v2.ste_vec(value) @> eql_v2.ste_vec($1::jsonb::eql_v2_encrypted)
//        Engages the documented `GIN (eql_v2.ste_vec(value))` index — both
//        sides of @> resolve to eql_v2_encrypted[] which matches the GIN
//        opclass directly. The needle is a sampled row's value, so the
//        query matches at least that source row.
//
//        Note (filed as a follow-up): the bare form
//        `WHERE value @> $1::eql_v2_encrypted` does NOT engage the GIN
//        today. eql_v2."@>" is marked inlinable SQL but wraps
//        ste_vec_contains() which is PL/pgSQL — inlining stops at the
//        wrapper, leaving the planner with a black-box function call and
//        no path to the indexed expression. Result: seq scan + per-row
//        ste_vec_contains, pathologically slow even on 10k rows. The bench
//        does not include the bare form because it would never complete
//        at the 1M / 10M tiers; the asymmetry is itself an EQL bug worth
//        filing.
//
//   2. JSON selector + equality (`->` + `=`)
//      field_eq/bare
//        WHERE (value -> '<sel>'::text) = $1::jsonb::eql_v2_encrypted LIMIT 10
//        eql_v2."->" is plpgsql (not inlinable) so the planner cannot match
//        any functional index against the LHS — forces seq scan + per-row
//        sv walk. This is the "natural" form a JS/ORM caller would write.
//      field_eq/extractor
//        WHERE eql_v2.hmac_256_terms(value) @> $1::jsonb LIMIT 10
//        Uses the documented `GIN (eql_v2.hmac_256_terms(value))` index — one
//        index covers field-level equality across every selector with an `hm`
//        term. Needle is `[{"s":"<sel>","hm":"<hash>"}]`.
//      field_eq/functional
//        WHERE eql_v2.hmac_256(value, '<sel>') = eql_v2.hmac_256($1::eql_v2_encrypted) LIMIT 10
//        Per-selector functional form. Would engage `hash (eql_v2.hmac_256(col, '<sel>'))`
//        if one existed; benches/main only creates the `hmac_256_terms` GIN
//        (one index for all selectors), so this scenario serves as a baseline
//        showing the cost of the per-selector recipe without a matching index.
//
//   3. JSON selector + ORDER BY (`->` then ORDER BY)
//      field_order/bare
//        SELECT id FROM tbl ORDER BY (value -> '<sel>'::text) LIMIT 10
//        Same `->` non-inlining problem as field_eq/bare. ORDER BY on
//        eql_v2_encrypted uses ORE under the hood, but the planner can't
//        see through `->` to engage any functional ORE index.
//      field_order/functional
//        SELECT id FROM tbl ORDER BY <ore_extractor>(value -> '<sel>'::text) LIMIT 10
//        Direct ORE extractor. <ore_extractor> is selected at startup based
//        on which orderable tag the chosen sv element carries:
//          oc -> eql_v2.ore_cllw            (Standard mode, ORE CLLW)
//          op -> eql_v2.ope_cllw            (Compat  mode, OPE CLLW)
//          ob -> eql_v2.ore_block_u64_8_256 (Block ORE — root scalars only)
//
// Needle / selector picking happens once at startup against the target
// table. The bench picks one sv element with an orderable tag (for the order
// scenarios) and falls through to sv[0] otherwise — a single chosen
// selector is reused across all scenarios so the comparison is consistent.
//
// Shape compatibility: post-EQL 2.3 ste_vec elements emit `hm` for equality
// and one of `oc` (Standard) / `op` (Compat) for orderable terms. Pre-2.3
// columns carrying `b3` / `ocf` / `ocv` / `opf` / `opv` no longer satisfy
// the new extractor functions and the hm-dependent or order-dependent
// scenarios will skip at startup. Re-ingest under the new format to
// engage the bench.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{bench_assert, extract_indexes_used, write_metadata_file, ScenarioMetadata};
use serde_json::Value as JsonValue;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use sqlx::Row;
use tokio::runtime::Runtime;

/// Sampled needles, picked once at startup. The picker scans the first
/// row's `sv` array twice: once for an `hm`-bearing element (drives
/// field_eq/* scenarios) and once for an orderable-bearing element (drives
/// field_order/*). Each scenario family uses the selector that's actually
/// addressable for it — in the post-#1955 wire format these are typically
/// disjoint (`hm` on the array-prefix selector lookup element, `oc`/`op`
/// on value elements).
#[derive(Debug)]
struct Needles {
    /// Whole-row value (for the containment needle). No selector needed.
    sample_value: JsonValue,
    /// Selector + payload for the field_eq/* scenarios. None when no
    /// sv element carries `hm` (pre-2.3 / Compat-mode-without-hmac).
    hm_pick: Option<HmPick>,
    /// Selector + tag for the field_order/* scenarios. None when no
    /// sv element carries an orderable term.
    ore_pick: Option<OrePick>,
}

#[derive(Debug)]
struct HmPick {
    /// Deterministic selector hash for the `hm`-bearing field (inlined
    /// into SQL strings, not bound — the planner needs it as a literal
    /// for any functional index to match).
    selector: String,
    /// `value -> '<selector>'` for the chosen selector (for the
    /// field_eq/bare needle).
    sample_field_value: JsonValue,
    /// `[{"s":"<sel>","hm":"<hash>"}]` for the field_eq/extractor scenario.
    hmac_term: String,
}

#[derive(Debug)]
struct OrePick {
    /// Deterministic selector hash for the orderable-bearing field.
    selector: String,
    /// `value -> '<selector>'` for the chosen selector (for the
    /// field_order/bare needle).
    sample_field_value: JsonValue,
    /// Orderable tag on the chosen sv element ("ob" / "oc" / "op").
    /// Drives `ore_extractor_for` to pick the matching extractor for the
    /// field_order/functional scenario.
    ore_term: String,
}

/// Map an sv-element orderable tag to the EQL extractor function that
/// returns the matching typed value. The extractor accepts an
/// eql_v2_encrypted argument.
fn ore_extractor_for(tag: &str) -> Option<&'static str> {
    match tag {
        "ob" => Some("eql_v2.ore_block_u64_8_256"),
        "oc" => Some("eql_v2.ore_cllw"),
        "op" => Some("eql_v2.ope_cllw"),
        _ => None,
    }
}

async fn sample_needles(pool: &sqlx::PgPool, table: &str) -> Needles {
    // Sample one row's sv array, then pick (independently) the first
    // hm-bearing element for the field_eq/* scenarios and the first
    // orderable-bearing element for field_order/*. Post-#1955 these are
    // typically disjoint: the array-prefix selector lookup element carries
    // `hm`, the value elements carry `oc` (Standard) or `op` (Compat).
    //
    // `value -> 'sel'::text` separately because the result type is
    // eql_v2_encrypted (with sv element fields hoisted to root + source
    // row's meta `i`, `v`). Explicit ::text cast on the selector literal:
    // eql_v2."->" has multiple overloads (text, eql_v2_encrypted, integer)
    // and PostgreSQL's assignment-cast resolution will otherwise try to
    // coerce the literal into eql_v2_encrypted, producing
    // "malformed record literal".
    let rows = sqlx::query(&format!(
        "SELECT elem ->> 's'  AS sel,
                elem ->> 'hm' AS hmac,
                elem          AS sv_elem,
                sample_value,
                ord
         FROM (
           SELECT value::jsonb AS sample_value,
                  (value).data -> 'sv' AS sv_array
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

    let sample_value: Json<JsonValue> = rows[0].get("sample_value");

    let mut hm_pick: Option<HmPick> = None;
    let mut ore_pick: Option<OrePick> = None;

    for row in &rows {
        let sel: String = row.get("sel");
        let hmac: Option<String> = row.get("hmac");
        let sv_elem: Json<JsonValue> = row.get("sv_elem");

        let ore_tag = sv_elem
            .0
            .as_object()
            .and_then(|m| ["ob", "oc", "op"].iter().find(|t| m.contains_key(**t)))
            .map(|t| (*t).to_string());

        if hm_pick.is_none() {
            if let Some(h) = hmac {
                let field_row = sqlx::query(&format!(
                    "SELECT (value -> '{sel}'::text)::jsonb AS sample_field_value
                     FROM {table}
                     LIMIT 1"
                ))
                .fetch_one(pool)
                .await
                .expect("query for hm sample field value failed");
                let sample_field_value: Json<JsonValue> =
                    field_row.get("sample_field_value");
                hm_pick = Some(HmPick {
                    selector: sel.clone(),
                    sample_field_value: sample_field_value.0,
                    hmac_term: format!(r#"[{{"s":"{}","hm":"{}"}}]"#, sel, h),
                });
            }
        }

        if ore_pick.is_none() {
            if let Some(tag) = ore_tag {
                let field_row = sqlx::query(&format!(
                    "SELECT (value -> '{sel}'::text)::jsonb AS sample_field_value
                     FROM {table}
                     LIMIT 1"
                ))
                .fetch_one(pool)
                .await
                .expect("query for ore sample field value failed");
                let sample_field_value: Json<JsonValue> =
                    field_row.get("sample_field_value");
                ore_pick = Some(OrePick {
                    selector: sel.clone(),
                    sample_field_value: sample_field_value.0,
                    ore_term: tag,
                });
            }
        }

        if hm_pick.is_some() && ore_pick.is_some() {
            break;
        }
    }

    Needles {
        sample_value: sample_value.0,
        hm_pick,
        ore_pick,
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS").unwrap_or_else(|_| "unknown".to_string());

    let table_suffix = match target_rows.as_str() {
        "10000" | "100000" | "1000000" | "10000000" => format!("_{}", target_rows),
        _ => String::new(),
    };
    let table_name = format!("json_ste_vec_small_encrypted{}", table_suffix);

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
            "json bench picked from `{}` — hm: {} | ore: {}",
            &table_name,
            needles
                .hm_pick
                .as_ref()
                .map(|p| p.selector.as_str())
                .unwrap_or("<none>"),
            needles
                .ore_pick
                .as_ref()
                .map(|p| format!("{} ({})", p.selector, p.ore_term))
                .unwrap_or_else(|| "<none>".to_string()),
        );
        (pool, needles)
    });

    // Serialise sampled values to JSON strings for binding ($1 is bound as
    // text and cast to ::jsonb / ::eql_v2_encrypted in the SQL).
    let sample_value_json =
        serde_json::to_string(&needles.sample_value).expect("serialise sample value");
    let hm_field_value_json = needles
        .hm_pick
        .as_ref()
        .map(|p| serde_json::to_string(&p.sample_field_value).expect("serialise hm field value"));
    let ore_field_value_json = needles
        .ore_pick
        .as_ref()
        .map(|p| serde_json::to_string(&p.sample_field_value).expect("serialise ore field value"));

    // --- Query strings ---

    let q_contains_functional = format!(
        "SELECT id FROM {table_name} \
         WHERE eql_v2.ste_vec(value) \
             @> eql_v2.ste_vec($1::jsonb::eql_v2_encrypted) LIMIT 10"
    );

    let q_field_eq_bare = needles.hm_pick.as_ref().map(|p| {
        let selector = &p.selector;
        format!(
            "SELECT id FROM {table_name} \
             WHERE (value -> '{selector}'::text) = $1::jsonb::eql_v2_encrypted LIMIT 10"
        )
    });

    let q_field_eq_extractor = if needles.hm_pick.is_some() {
        Some(format!(
            "SELECT id FROM {table_name} \
             WHERE eql_v2.hmac_256_terms(value) @> $1::jsonb LIMIT 10"
        ))
    } else {
        None
    };

    let q_field_eq_functional = needles.hm_pick.as_ref().map(|p| {
        let selector = &p.selector;
        format!(
            "SELECT id FROM {table_name} \
             WHERE eql_v2.hmac_256(value, '{selector}') \
                 = eql_v2.hmac_256($1::jsonb::eql_v2_encrypted) LIMIT 10"
        )
    });

    let q_field_order_bare = needles.ore_pick.as_ref().map(|p| {
        let selector = &p.selector;
        format!(
            "SELECT id FROM {table_name} \
             ORDER BY (value -> '{selector}'::text) LIMIT 10"
        )
    });

    let q_field_order_functional = needles.ore_pick.as_ref().and_then(|p| {
        let selector = &p.selector;
        ore_extractor_for(&p.ore_term).map(|fn_name| {
            format!(
                "SELECT id FROM {table_name} \
                 ORDER BY {fn_name}(value -> '{selector}'::text) LIMIT 10"
            )
        })
    });

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
                Some(&sample_value_json),
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
                "json bench: skipping field_eq/* scenarios — no sv element on the sampled \
                 row carries `hm`. Re-ingest with a cipherstash-client release that emits \
                 `hm` at sv-element level (post-#1955)."
            );
        }

        if let (Some(_), Some(q_bare)) = (needles.ore_pick.as_ref(), q_field_order_bare.as_deref())
        {
            out.push(
                capture(
                    &pool,
                    format!("JSON/json/field_order/bare/{}", target_rows),
                    q_bare,
                    None,
                )
                .await,
            );

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
            }
        } else {
            eprintln!(
                "json bench: skipping field_order/* scenarios — no sv element on the \
                 sampled row carries an orderable term (no ob / oc / op)."
            );
        }

        out
    });
    write_metadata_file("json", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    // --- Bench loop ---

    let mut group = c.benchmark_group("JSON");
    group.sample_size(10);

    {
        let id = format!("JSON/json/contains/functional/{}", target_rows);
        let q = q_contains_functional.clone();
        let needle = sample_value_json.clone();
        group.bench_function(format!("json/contains/functional/{}", target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let rows = bench_assert(
                    sqlx::query(&q).bind(&needle).fetch_all(&pool).await,
                    &id,
                );
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
                    let rows = bench_assert(
                        sqlx::query(&q).bind(&needle).fetch_all(&pool).await,
                        &id,
                    );
                    black_box(rows.len())
                })
            });
        }

        if let Some(q) = q_field_eq_extractor.clone() {
            let id = format!("JSON/json/field_eq/extractor/{}", target_rows);
            let needle = hmac_term.clone();
            group.bench_function(format!("json/field_eq/extractor/{}", target_rows), |b| {
                b.to_async(&rt).iter(|| async {
                    let rows = bench_assert(
                        sqlx::query(&q).bind(&needle).fetch_all(&pool).await,
                        &id,
                    );
                    black_box(rows.len())
                })
            });
        }

        if let Some(q) = q_field_eq_functional.clone() {
            let id = format!("JSON/json/field_eq/functional/{}", target_rows);
            let needle = hm_field.clone();
            group.bench_function(format!("json/field_eq/functional/{}", target_rows), |b| {
                b.to_async(&rt).iter(|| async {
                    let rows = bench_assert(
                        sqlx::query(&q).bind(&needle).fetch_all(&pool).await,
                        &id,
                    );
                    black_box(rows.len())
                })
            });
        }
    }

    if has_ore {
        if let Some(q) = q_field_order_bare.clone() {
            let id = format!("JSON/json/field_order/bare/{}", target_rows);
            group.bench_function(format!("json/field_order/bare/{}", target_rows), |b| {
                b.to_async(&rt).iter(|| async {
                    let rows = bench_assert(
                        sqlx::query(&q).fetch_all(&pool).await,
                        &id,
                    );
                    black_box(rows.len())
                })
            });
        }

        if let Some(q) = q_field_order_functional.clone() {
            let id = format!("JSON/json/field_order/functional/{}", target_rows);
            group.bench_function(
                format!("json/field_order/functional/{}", target_rows),
                |b| {
                    b.to_async(&rt).iter(|| async {
                        let rows = bench_assert(
                            sqlx::query(&q).fetch_all(&pool).await,
                            &id,
                        );
                        black_box(rows.len())
                    })
                },
            );
        }
    }

    // Silence unused-variable warning when ore_field_value_json isn't bound
    // by any scenario (currently only used for parity with hm_field; reserved
    // for future field_order scenarios that bind the orderable sample).
    let _ = ore_field_value_json;

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
