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
//        on which ORE tag the chosen sv element carries:
//          ocf -> eql_v2.ore_cllw_u64_8     (CLLW fixed,   ints)
//          ocv -> eql_v2.ore_cllw_var_8     (CLLW variable, strings)
//          ob  -> eql_v2.ore_block_u64_8_256 (block ORE — post-James-unification)
//
// Needle / selector picking happens once at startup against the target
// table. The bench picks one sv element with an ORE tag (for the order
// scenarios) and falls through to sv[0] otherwise — a single chosen
// selector is reused across all scenarios so the comparison is consistent.
//
// Shape compatibility: scenarios in (2) depend on the post-2.3 ste_vec
// shape (sv elements emit `hm` rather than `b3`). The cipherstash-client
// version pinned in benches/main currently emits the pre-2.3 shape, so the
// hm-dependent scenarios are skipped at startup with a clear message and
// will start producing numbers once the upstream change lands and the
// table is re-ingested.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{bench_assert, extract_indexes_used, write_metadata_file, ScenarioMetadata};
use serde_json::Value as JsonValue;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use sqlx::Row;
use tokio::runtime::Runtime;

/// Sampled needles, picked once at startup.
#[derive(Debug)]
struct Needles {
    /// Deterministic selector hash for the chosen field (inlined into SQL
    /// strings, not bound — the planner needs it as a literal for any
    /// functional index to match).
    selector: String,
    /// Whole-row value (for the containment needle).
    sample_value: JsonValue,
    /// `value -> '<selector>'` for the chosen selector (for the field_eq
    /// needles).
    sample_field_value: JsonValue,
    /// `[{"s":"<sel>","hm":"<hash>"}]` for the field_eq/extractor scenario.
    /// None when the chosen sv element lacks `hm` (pre-2.3 shape).
    hmac_term: Option<String>,
    /// ORE tag on the chosen sv element ("ob"/"ocf"/"ocv"), or None.
    /// Drives `ore_extractor_for` to pick the matching extractor for the
    /// field_order/functional scenario.
    ore_term: Option<String>,
}

/// Map an sv-element ORE tag to the EQL extractor function that returns
/// the matching ORE typed value. The extractor accepts an
/// eql_v2_encrypted argument.
fn ore_extractor_for(tag: &str) -> Option<&'static str> {
    match tag {
        "ob" => Some("eql_v2.ore_block_u64_8_256"),
        "ocf" => Some("eql_v2.ore_cllw_u64_8"),
        "ocv" => Some("eql_v2.ore_cllw_var_8"),
        _ => None,
    }
}

async fn sample_needles(pool: &sqlx::PgPool, table: &str) -> Needles {
    // Prefer an sv element with an ORE tag — the field_order scenarios
    // depend on ORE being present. Fall back to sv[0] if no ORE-bearing
    // element exists. We sample one row first (so the SRF expansion below
    // unfolds over a single row's sv array), then unfold the sv array via
    // LATERAL jsonb_array_elements ... WITH ORDINALITY to get each
    // element and its index so we can rank them.
    let row = sqlx::query(&format!(
        "SELECT sel, hmac, sv_elem, sample_value FROM (
           SELECT elem ->> 's'  AS sel,
                  elem ->> 'hm' AS hmac,
                  elem          AS sv_elem,
                  sample_value,
                  (elem ? 'ob' OR elem ? 'ocf' OR elem ? 'ocv') AS has_ore,
                  ord
           FROM (
             SELECT value::jsonb AS sample_value,
                    (value).data -> 'sv' AS sv_array
             FROM {table}
             LIMIT 1
           ) source,
           LATERAL jsonb_array_elements(sv_array) WITH ORDINALITY AS j(elem, ord)
         ) ranked
         ORDER BY has_ore DESC, ord ASC
         LIMIT 1"
    ))
    .fetch_optional(pool)
    .await
    .expect("query for sample selector failed")
    .unwrap_or_else(|| panic!("table `{table}` is empty"));

    let selector: String = row.get("sel");
    let hmac: Option<String> = row.get("hmac");
    let sample_value: Json<JsonValue> = row.get("sample_value");
    let sv_elem: Json<JsonValue> = row.get("sv_elem");

    let ore_term = sv_elem
        .0
        .as_object()
        .and_then(|m| ["ob", "ocf", "ocv"].iter().find(|t| m.contains_key(**t)))
        .map(|t| (*t).to_string());

    // Pull `value -> 'sel'` separately — the result is an eql_v2_encrypted
    // whose top-level fields are the sv element's fields (s, hm, b3, ocf,
    // etc.), wrapped with the source row's meta (i, v, ...).
    // Explicit ::text cast on the selector: eql_v2."->" has multiple
    // overloads (text, eql_v2_encrypted, integer) and PostgreSQL's
    // assignment-cast resolution will otherwise try to coerce the literal
    // string into eql_v2_encrypted (a composite type), producing
    // "malformed record literal".
    let field_row = sqlx::query(&format!(
        "SELECT (value -> '{selector}'::text)::jsonb AS sample_field_value
         FROM {table}
         LIMIT 1"
    ))
    .fetch_one(pool)
    .await
    .expect("query for sample field value failed");
    let sample_field_value: Json<JsonValue> = field_row.get("sample_field_value");

    let hmac_term = hmac
        .as_ref()
        .map(|h| format!(r#"[{{"s":"{}","hm":"{}"}}]"#, selector, h));

    Needles {
        selector,
        sample_value: sample_value.0,
        sample_field_value: sample_field_value.0,
        hmac_term,
        ore_term,
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
            "json bench picked selector `{}` from `{}` (hm: {}, ore: {})",
            &needles.selector,
            &table_name,
            needles.hmac_term.is_some(),
            needles.ore_term.as_deref().unwrap_or("<none>"),
        );
        (pool, needles)
    });

    // Serialise sampled values to JSON strings for binding ($1 is bound as
    // text and cast to ::jsonb / ::eql_v2_encrypted in the SQL).
    let sample_value_json =
        serde_json::to_string(&needles.sample_value).expect("serialise sample value");
    let sample_field_value_json =
        serde_json::to_string(&needles.sample_field_value).expect("serialise sample field value");

    let selector = &needles.selector;

    // --- Query strings ---

    let q_contains_functional = format!(
        "SELECT id FROM {table_name} \
         WHERE eql_v2.ste_vec(value) \
             @> eql_v2.ste_vec($1::jsonb::eql_v2_encrypted) LIMIT 10"
    );

    let q_field_eq_bare = format!(
        "SELECT id FROM {table_name} \
         WHERE (value -> '{selector}'::text) = $1::jsonb::eql_v2_encrypted LIMIT 10"
    );

    let q_field_eq_extractor = format!(
        "SELECT id FROM {table_name} \
         WHERE eql_v2.hmac_256_terms(value) @> $1::jsonb LIMIT 10"
    );

    let q_field_eq_functional = format!(
        "SELECT id FROM {table_name} \
         WHERE eql_v2.hmac_256(value, '{selector}') \
             = eql_v2.hmac_256($1::jsonb::eql_v2_encrypted) LIMIT 10"
    );

    let q_field_order_bare = format!(
        "SELECT id FROM {table_name} \
         ORDER BY (value -> '{selector}'::text) LIMIT 10"
    );

    let q_field_order_functional = needles
        .ore_term
        .as_deref()
        .and_then(ore_extractor_for)
        .map(|fn_name| {
            format!(
                "SELECT id FROM {table_name} \
                 ORDER BY {fn_name}(value -> '{selector}'::text) LIMIT 10"
            )
        });

    // --- Metadata sidecar ---

    let has_hm = needles.hmac_term.is_some();
    let has_ore = needles.ore_term.is_some();

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

        if has_hm {
            out.push(
                capture(
                    &pool,
                    format!("JSON/json/field_eq/bare/{}", target_rows),
                    &q_field_eq_bare,
                    Some(&sample_field_value_json),
                )
                .await,
            );

            out.push(
                capture(
                    &pool,
                    format!("JSON/json/field_eq/extractor/{}", target_rows),
                    &q_field_eq_extractor,
                    needles.hmac_term.as_deref(),
                )
                .await,
            );

            out.push(
                capture(
                    &pool,
                    format!("JSON/json/field_eq/functional/{}", target_rows),
                    &q_field_eq_functional,
                    Some(&sample_field_value_json),
                )
                .await,
            );
        } else {
            eprintln!(
                "json bench: skipping field_eq/* scenarios — sv element on selector `{}` \
                 has no `hm` term (cipherstash-client emitting pre-2.3 shape). Bump the \
                 suite pin to a version that emits `hm` and re-ingest via \
                 `mise run prepare:json_ste_vec_small <rows>` to enable these.",
                &needles.selector
            );
        }

        if has_ore {
            out.push(
                capture(
                    &pool,
                    format!("JSON/json/field_order/bare/{}", target_rows),
                    &q_field_order_bare,
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
                "json bench: skipping field_order/* scenarios — sv element on selector `{}` \
                 carries no ORE term (no ob / ocf / ocv). Try a selector that has ORE.",
                &needles.selector
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
        {
            let id = format!("JSON/json/field_eq/bare/{}", target_rows);
            let q = q_field_eq_bare.clone();
            let needle = sample_field_value_json.clone();
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

        {
            let id = format!("JSON/json/field_eq/extractor/{}", target_rows);
            let q = q_field_eq_extractor.clone();
            let needle = needles
                .hmac_term
                .clone()
                .expect("hmac_term present when has_hm");
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

        {
            let id = format!("JSON/json/field_eq/functional/{}", target_rows);
            let q = q_field_eq_functional.clone();
            let needle = sample_field_value_json.clone();
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
        {
            let id = format!("JSON/json/field_order/bare/{}", target_rows);
            let q = q_field_order_bare.clone();
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

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
