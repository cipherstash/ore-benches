//! EQL v3 sibling of `benches/json.rs` — encrypted-JSONB (SteVec) queries
//! against `json_ste_vec_small_encrypted_v3_<N>` (`eql_v3.json`).
//!
//! Scenario ids match the v2 bench. The SQL surface maps as:
//!
//!   contains/functional
//!     v2: eql_v2.jsonb_array(value) @> eql_v2.jsonb_array($1::eql_v2_encrypted)
//!     v3: value @> $1::jsonb::eql_v3.jsonb_query
//!     One canonical containment recipe in v3: the typed @>(json, jsonb_query)
//!     overload inlines to `eql_v3.to_ste_vec_query(value)::jsonb @> needle`
//!     and engages the jsonb_path_ops GIN from the static index DDL. The
//!     needle is the sampled row's sv entries stripped to `s` + term —
//!     exactly what `eql_v3.to_ste_vec_query` produces.
//!
//!   field_eq/bare       (value -> '<sel>'::text) = $1::jsonb::eql_v3.jsonb_entry
//!   field_eq/extractor  value @> $1::jsonb::eql_v3.jsonb_query   (single-entry needle)
//!   field_eq/functional eql_v3.eq_term(value -> '<sel>'::text) = eql_v3.eq_term($1::jsonb::eql_v3.jsonb_entry)
//!   field_order/functional  ORDER BY eql_v3.ore_cllw(value -> '<sel>'::text) LIMIT 10
//!
//! One v3-specific expectation: `eql_v3."->"` is LANGUAGE sql (v2's was
//! plpgsql), so the bare `->`+`=` form should now inline all the way down
//! and match the per-selector functional index — check `indexes_used` for
//! field_eq/bare, which could not engage in v2.
//!
//! Per-selector functional indexes (eq_term / ore_cllw over `value -> sel`)
//! are built at startup once the selector is sampled, mirroring the v2
//! bench's create_field_indexes.

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
    /// normalized to the jsonb_query shape (drives contains/functional).
    document_query: String,
    hm_pick: Option<HmPick>,
    ore_pick: Option<OrePick>,
}

#[derive(Debug)]
struct HmPick {
    selector: String,
    /// `(value -> '<selector>')::jsonb` of the sampled row — an entry
    /// merged with root meta, castable to eql_v3.jsonb_entry.
    sample_field_value: JsonValue,
    /// Single-entry jsonb_query needle for field_eq/extractor.
    hmac_term: String,
}

#[derive(Debug)]
struct OrePick {
    selector: String,
}

async fn sample_needles(pool: &sqlx::PgPool, table: &str) -> Needles {
    // v3 payloads are jsonb domains — `value::jsonb -> 'sv'` walks the
    // document directly (no composite `.data` hop like v2).
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
    // containment needle (what eql_v3.to_ste_vec_query does in SQL).
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
                    hmac_term: format!(r#"{{"sv":[{{"s":"{}","hm":"{}"}}]}}"#, sel, h),
                });
            }
        }

        if ore_pick.is_none() && obj.contains_key("oc") {
            ore_pick = Some(OrePick {
                selector: sel.clone(),
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
/// build-cost rationale.
async fn create_field_indexes(pool: &sqlx::PgPool, table: &str, needles: &Needles) {
    eprintln!("json_v3 bench: building per-selector functional indexes...");

    if let Some(p) = needles.hm_pick.as_ref() {
        sqlx::query(&format!("DROP INDEX IF EXISTS {table}_field_eq_idx"))
            .execute(pool)
            .await
            .expect("drop stale field_eq index");
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
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
