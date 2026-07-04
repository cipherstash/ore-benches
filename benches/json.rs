// JSON / ste_vec query benches.
//
// Three query pattern families that map to the patterns a JS / ORM caller
// would write against an encrypted JSON column, plus the recipe forms a
// human author would write knowing the EQL extractor functions.
//
//   1. JSON containment (whole-document)
//      contains/functional
//        WHERE eql_v2.jsonb_array(value) @> eql_v2.jsonb_array($1::jsonb::eql_v2_encrypted)
//        Engages the documented `GIN (eql_v2.jsonb_array(value))` index —
//        both sides of @> resolve to native jsonb[], which matches the GIN
//        opclass directly. The needle is a sampled row's value, so the
//        query matches at least that source row.
//
//        This is the whole-document recipe: both operands are extracted to
//        jsonb[] via eql_v2.jsonb_array. For a field-targeted needle the
//        typed stevec_query path (scenario 2, field_eq/extractor) inlines
//        to a native jsonb @> and engages its own GIN — prefer it when the
//        needle addresses specific selectors rather than the whole row.
//
//   2. JSON selector + equality (`->` + `=`)
//      field_eq/bare
//        WHERE (value -> '<sel>'::text) = $1::jsonb::eql_v2.ste_vec_entry LIMIT 10
//        The "natural" JS/ORM form. `->` returns `eql_v2.ste_vec_entry` and
//        `=` on that domain is inlinable SQL that folds to
//        `eql_v2.eq_term(a) = eql_v2.eq_term(b)` — structurally identical to
//        field_eq/functional, so it engages the same per-selector
//        `hash (eql_v2.eq_term(value -> '<sel>'))` index the bench builds at
//        startup (see `create_field_indexes`).
//      field_eq/extractor
//        WHERE value @> $1::jsonb::eql_v2.stevec_query LIMIT 10
//        Uses the typed `@>(eql_v2_encrypted, eql_v2.stevec_query)` overload,
//        which inlines to `eql_v2.to_stevec_query(value)::jsonb @> needle`
//        and engages the column-wide
//        `GIN (eql_v2.to_stevec_query(value)::jsonb jsonb_path_ops)` index —
//        one index covers every selector, XOR-aware (both hm- and oc-bearing).
//        Needle is `{"sv":[{"s":"<sel>","hm":"<hash>"}]}`.
//      field_eq/functional
//        WHERE eql_v2.eq_term(value -> '<sel>') = eql_v2.eq_term($1::eql_v2.ste_vec_entry) LIMIT 10
//        Per-selector functional form. Engages a
//        `hash (eql_v2.eq_term(value -> '<sel>'))` index. ste_vec equality is
//        per-selector, so the index can't live in the static
//        sql/indexes/*_up.sql — the bench builds it at startup once
//        sample_needles has picked the selector (see `create_field_indexes`).
//
//   3. JSON selector + ORDER BY (`->` then ORDER BY)
//      field_order/functional
//        SELECT id FROM tbl ORDER BY <ore_extractor>(value -> '<sel>'::text) LIMIT 10
//        Direct ORE extractor. <ore_extractor> is selected at startup based
//        on which orderable tag the chosen sv element carries:
//          oc -> eql_v2.ore_cllw            (ORE CLLW — sv elements)
//          ob -> eql_v2.ore_block_u64_8_256 (Block ORE — root scalars only)
//        `->` returns `eql_v2.ste_vec_entry`, and `eql_v2.ore_cllw` has an
//        overload on that domain — no `.data` cast needed. Engages a
//        `btree (<ore_extractor>(value -> '<sel>'))` index — the
//        `eql_v2.ore_cllw_ops` opclass (DEFAULT FOR TYPE, EQL #221) makes
//        `ORDER BY ... LIMIT n` an index scan. Per-selector, so the bench
//        builds it at startup too (see `create_field_indexes`).
//
//      No `field_order/bare` scenario. The bare form
//      `ORDER BY value -> '<sel>'` does not syntactically match the
//      functional ORE index expression, so the plan is always Seq Scan +
//      Top-N sort — linear in table size. The extractor form is the
//      canonical recipe; see §4 of the EQL query performance guide.
//
// Needle / selector picking happens once at startup against the target
// table. The bench picks one sv element with an orderable tag (for the order
// scenarios) and falls through to sv[0] otherwise — a single chosen
// selector is reused across all scenarios so the comparison is consistent.
//
// Shape compatibility: EQL 2.3 ste_vec elements carry `hm` (equality) and
// `oc` (ORE CLLW ordering) — exactly one of the two per element, under the
// XOR contract. Pre-2.3 columns carrying `b3` / `ocf` / `ocv` (or the
// removed Compat `op` / `opf` / `opv`) no longer satisfy the new extractor
// functions, so the hm- or order-dependent scenarios skip at startup.
// Re-ingest under the EQL 2.3 format to engage the bench.

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
/// disjoint (`hm` on the array-prefix selector lookup element, `oc` on
/// value elements).
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
    /// Orderable tag on the chosen sv element ("ob" / "oc").
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
        _ => None,
    }
}

async fn sample_needles(pool: &sqlx::PgPool, table: &str) -> Needles {
    // Sample one row's sv array, then pick (independently) the first
    // hm-bearing element for the field_eq/* scenarios and the first
    // orderable-bearing element for field_order/*. Post-#1955 these are
    // typically disjoint: the array-prefix selector lookup element carries
    // `hm`, the value elements carry `oc`.
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
            .and_then(|m| ["ob", "oc"].iter().find(|t| m.contains_key(**t)))
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
                let sample_field_value: Json<JsonValue> = field_row.get("sample_field_value");
                hm_pick = Some(HmPick {
                    selector: sel.clone(),
                    sample_field_value: sample_field_value.0,
                    // sv-shaped needle for the typed stevec_query @> recipe
                    // (post PR cipherstash/eql#223 — hmac_256_terms removed).
                    hmac_term: format!(r#"{{"sv":[{{"s":"{}","hm":"{}"}}]}}"#, sel, h),
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
                let sample_field_value: Json<JsonValue> = field_row.get("sample_field_value");
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

/// Build the per-selector functional indexes the `field_eq/functional` and
/// `field_order/functional` scenarios measure.
///
/// ste_vec equality and ordering are per-selector — the index expression
/// embeds the selector hash — so these indexes cannot be declared in the
/// static `sql/indexes/*_up.sql`: the selector is only known once
/// `sample_needles` has picked it. Building them here, before the criterion
/// loop, is one-time setup and is not part of what criterion measures.
async fn create_field_indexes(pool: &sqlx::PgPool, table: &str, needles: &Needles) {
    eprintln!("json bench: building per-selector functional indexes...");

    if let Some(p) = needles.hm_pick.as_ref() {
        sqlx::query(&format!("DROP INDEX IF EXISTS {table}_field_eq_idx"))
            .execute(pool)
            .await
            .expect("drop stale field_eq index");
        // btree, not hash: `eq_term` returns bytea and a btree serves `=`
        // equally well — but hash index *builds* degrade badly at scale
        // (random bucket I/O, and they can't use parallel workers). btree
        // builds sort-then-bulk-load and parallelise, so the 10M build goes
        // from pathological to routine. Same query cost either way.
        sqlx::query(&format!(
            "CREATE INDEX {table}_field_eq_idx ON {table} \
             USING btree (eql_v2.eq_term(value -> '{}'::text))",
            p.selector
        ))
        .execute(pool)
        .await
        .expect("create field_eq functional index");
    }

    if let Some(p) = needles.ore_pick.as_ref() {
        if let Some(fn_name) = ore_extractor_for(&p.ore_term) {
            sqlx::query(&format!("DROP INDEX IF EXISTS {table}_field_order_idx"))
                .execute(pool)
                .await
                .expect("drop stale field_order index");
            sqlx::query(&format!(
                "CREATE INDEX {table}_field_order_idx ON {table} \
                 USING btree ({fn_name}(value -> '{}'::text))",
                p.selector
            ))
            .execute(pool)
            .await
            .expect("create field_order functional index");
        }
    }

    // Refresh planner statistics so the new indexes are costed correctly.
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

        create_field_indexes(&pool, &table_name, &needles).await;

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

    // Post-EQL-2.3 typed-StEVec path (PR cipherstash/eql#223):
    //   - `->` returns `eql_v2.ste_vec_entry` (was `eql_v2_encrypted`),
    //     so RHS literals cast to `::eql_v2.ste_vec_entry` not
    //     `::eql_v2_encrypted`. Bare field equality uses the
    //     `ste_vec_entry × ste_vec_entry` `=` operator, which inlines
    //     to `eq_term(a) = eq_term(b)`.
    //   - The fused `eql_v2.hmac_256(eql_v2_encrypted, text)` was
    //     removed; the functional recipe shifts to
    //     `eql_v2.eq_term(value -> 'sel'::text)`.
    //   - For containment, `eql_v2.ste_vec(col) @> ...` no longer
    //     builds a GIN index against the strict-compare contract
    //     (#211) because GIN-on-array uses the default btree opclass
    //     on the element type, which raises on missing `ob`. Switch
    //     to the canonical `eql_v2.jsonb_array(col) @>` recipe used
    //     by the EQL test suite — same containment semantics, jsonb[]
    //     element type bypasses the broken compare.

    let q_contains_functional = format!(
        "SELECT id FROM {table_name} \
         WHERE eql_v2.jsonb_array(value) \
             @> eql_v2.jsonb_array($1::jsonb::eql_v2_encrypted) LIMIT 10"
    );

    let q_field_eq_bare = needles.hm_pick.as_ref().map(|p| {
        let selector = &p.selector;
        format!(
            "SELECT id FROM {table_name} \
             WHERE (value -> '{selector}'::text) = $1::jsonb::eql_v2.ste_vec_entry LIMIT 10"
        )
    });

    let q_field_eq_extractor = if needles.hm_pick.is_some() {
        // Post PR cipherstash/eql#223: `hmac_256_terms` was removed
        // (structurally wrong under the XOR contract — silently dropped
        // oc-bearing sv elements). Canonical replacement: the typed
        // `@>(eql_v2_encrypted, eql_v2.stevec_query)` overload, which
        // inlines to `eql_v2.to_stevec_query(col)::jsonb @> needle::jsonb`
        // and engages a functional GIN on the same expression. The new
        // recipe is XOR-aware (covers both hm- and oc-bearing selectors
        // with one index).
        Some(format!(
            "SELECT id FROM {table_name} \
             WHERE value @> $1::jsonb::eql_v2.stevec_query LIMIT 10"
        ))
    } else {
        None
    };

    let q_field_eq_functional = needles.hm_pick.as_ref().map(|p| {
        let selector = &p.selector;
        format!(
            "SELECT id FROM {table_name} \
             WHERE eql_v2.eq_term(value -> '{selector}'::text) \
                 = eql_v2.eq_term($1::jsonb::eql_v2.ste_vec_entry) LIMIT 10"
        )
    });

    // Note: a `field_order/bare` scenario (`ORDER BY value -> '<sel>' LIMIT n`)
    // is deliberately omitted. The bare form can't engage the functional ORE
    // index — `eql_v2."->"` is plpgsql, so the planner has no way to match
    // the sort key against the indexed expression — and the resulting Seq
    // Scan + Top-N sort scales linearly with table size. The extractor form
    // below is the canonical recipe, documented in §4 of the EQL query
    // performance guide (`docs/reference/query-performance.md`). Measuring
    // both would just inflate the run cost without surfacing new behaviour.

    let q_field_order_functional = needles.ore_pick.as_ref().and_then(|p| {
        let selector = &p.selector;
        ore_extractor_for(&p.ore_term).map(|fn_name| {
            // Post-PR cipherstash/eql#223: `->` returns
            // `eql_v2.ste_vec_entry` (a domain over jsonb), and
            // `eql_v2.ore_cllw` has an overload accepting that domain
            // directly — no `.data` cast needed. The `eql_v2.ore_cllw_ops`
            // btree opclass (DEFAULT FOR TYPE — #221) wires
            // functional-index match on the expression.
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

        if needles.ore_pick.is_some() {
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
                "json bench: skipping field_order/functional — no sv element on the \
                 sampled row carries an orderable term (no ob / oc)."
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
                let rows = bench_assert(sqlx::query(&q).bind(&needle).fetch_all(&pool).await, &id);
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
    }

    // Silence unused-variable warning when ore_field_value_json isn't bound
    // by any scenario (currently only used for parity with hm_field; reserved
    // for future field_order scenarios that bind the orderable sample).
    let _ = ore_field_value_json;

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
