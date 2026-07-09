//! Plaintext baselines for the v3 encrypted-vs-plaintext comparison — the
//! docs/marketing "overhead vs native Postgres" story. Runs the SAME query
//! shapes as the headline encrypted scenarios against plaintext tables with
//! equivalent indexes:
//!
//!   plaintext/exact_eq            string_plaintext_<N>   WHERE value = $1 LIMIT 1     (btree)
//!   plaintext/range_gt_10         integer_plaintext_<N>  WHERE value > $1 LIMIT 10    (btree)
//!   plaintext/range_lt_ordered_10 integer_plaintext_<N>  WHERE value < $1 ORDER BY value LIMIT 10
//!   plaintext/json_contains       json_small_plaintext_<N>  WHERE value @> $1 LIMIT 10 (jsonb_path_ops GIN)
//!   plaintext/json_field_eq       json_small_plaintext_<N>  WHERE value -> 'age' = $1 LIMIT 10
//!
//! (GROUP BY plaintext baselines already live inside benches/group_by_v3.rs,
//! carried over from the v2 bench.)
//!
//! The comparison report maps these onto their encrypted counterparts by
//! scenario name. No encryption client is involved anywhere in this bench.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{
    bench_assert, extract_indexes_used, init_tracing, write_metadata_file_in, ScenarioMetadata,
};
use serde_json::Value as JsonValue;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use tokio::runtime::Runtime;

fn criterion_benchmark(c: &mut Criterion) {
    init_tracing();
    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS").unwrap_or_else(|_| "unknown".to_string());
    let table_suffix = match target_rows.as_str() {
        "10000" | "100000" | "1000000" | "10000000" => format!("_{}", target_rows),
        _ => String::new(),
    };
    let string_table = format!("string_plaintext{}", table_suffix);
    let integer_table = format!("integer_plaintext{}", table_suffix);
    let json_table = format!("json_small_plaintext{}", table_suffix);

    let pool = rt.block_on(async {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database")
    });

    // Sample needles from real rows (same guarantee as the encrypted
    // benches: equality scenarios hit at least one row).
    let (string_needle, json_age_needle): (String, i64) = rt.block_on(async {
        let s: (String,) =
            sqlx::query_as(&format!("SELECT value FROM {string_table} LIMIT 1"))
                .fetch_one(&pool)
                .await
                .expect("sample from string_plaintext failed — is it prepared?");
        let a: (JsonValue,) = sqlx::query_as(&format!(
            "SELECT (value -> 'age')::jsonb FROM {json_table} LIMIT 1"
        ))
        .fetch_one(&pool)
        .await
        .expect("sample from json_small_plaintext failed — is it prepared?");
        (s.0, a.0.as_i64().expect("age is an integer"))
    });
    let json_contains_needle = format!(r#"{{"age": {}}}"#, json_age_needle);
    let json_field_eq_needle = json_age_needle.to_string();

    // (scenario, statement, bind)
    enum Bind {
        Text(String),
        Int(i32),
        Jsonb(String),
    }
    let scenarios: Vec<(&str, String, Option<Bind>)> = vec![
        (
            "exact_eq",
            format!("SELECT id, value FROM {string_table} WHERE value = $1 LIMIT 1"),
            Some(Bind::Text(string_needle)),
        ),
        (
            "range_gt_10",
            format!("SELECT id, value FROM {integer_table} WHERE value > $1 LIMIT 10"),
            Some(Bind::Int(5000)),
        ),
        (
            "range_lt_ordered_10",
            format!(
                "SELECT id, value FROM {integer_table} WHERE value < $1 ORDER BY value LIMIT 10"
            ),
            Some(Bind::Int(5000)),
        ),
        (
            "json_contains",
            format!("SELECT id FROM {json_table} WHERE value @> $1::jsonb LIMIT 10"),
            Some(Bind::Jsonb(json_contains_needle)),
        ),
        (
            "json_field_eq",
            format!("SELECT id FROM {json_table} WHERE value -> 'age' = $1::jsonb LIMIT 10"),
            Some(Bind::Jsonb(json_field_eq_needle)),
        ),
    ];

    fn bind_query<'q>(
        q: &'q str,
        bind: &'q Option<Bind>,
    ) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        let query = sqlx::query(q);
        match bind {
            Some(Bind::Text(s)) => query.bind(s),
            Some(Bind::Int(i)) => query.bind(i),
            Some(Bind::Jsonb(j)) => query.bind(j),
            None => query,
        }
    }

    let metadata = rt.block_on(async {
        let mut out = Vec::with_capacity(scenarios.len());
        for (scenario, statement, bind) in &scenarios {
            let bench_id = format!("PLAINTEXT/plaintext/{}/{}", scenario, target_rows);
            let explain_sql = format!("EXPLAIN (FORMAT JSON) {}", statement);
            let (Json(explain),): (Json<JsonValue>,) = {
                let q = sqlx::query_as(&explain_sql);
                match bind {
                    Some(Bind::Text(s)) => q.bind(s),
                    Some(Bind::Int(i)) => q.bind(i),
                    Some(Bind::Jsonb(j)) => q.bind(j),
                    None => q,
                }
                .fetch_one(&pool)
                .await
                .expect("EXPLAIN failed")
            };
            let indexes_used = extract_indexes_used(&explain);
            let rows = bind_query(statement, bind)
                .fetch_all(&pool)
                .await
                .expect("execute for row-count failed");
            out.push(ScenarioMetadata {
                id: bench_id,
                query: statement.clone(),
                parameters: Vec::new(),
                explain,
                indexes_used,
                rows_returned: rows.len() as u64,
            });
        }
        out
    });
    write_metadata_file_in("results/query/v3", "plaintext", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("PLAINTEXT");
    group.sample_size(10);

    for (scenario, statement, bind) in scenarios {
        let bench_id = format!("PLAINTEXT/plaintext/{}/{}", scenario, target_rows);
        let inner_id = bench_id.clone();
        group.bench_function(format!("plaintext/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let rows = bench_assert(
                    bind_query(&statement, &bind).fetch_all(&pool).await,
                    &inner_id,
                );
                black_box(rows.len());
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
