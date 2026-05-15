use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use tokio::runtime::Runtime;

// The canonical `GROUP BY` recipe on an encrypted column is the extractor
// form: `GROUP BY eql_v2.hmac_256(value)`. The body of `eql_v2.hmac_256` is
// inlinable single-statement SQL (`(val).data ->> 'hm'`), so the planner
// folds it into the aggregation and the group key is a 32-byte HMAC that
// fits comfortably in `work_mem`. `HashAggregate` engages on every
// deployment without `work_mem` tuning.
//
// The bench wraps the GROUP BY in a `count(*)` subquery:
//
//   SELECT count(*) FROM (SELECT 1 FROM tbl GROUP BY eql_v2.hmac_256(value)) g
//
// rather than running the bare `SELECT count(*) FROM tbl GROUP BY ...` form.
// The bench tables are populated by `encrypt_string` with `fake` random
// English names — effectively unique per row at high cardinality — so the
// bare-GROUP BY shape emitted ~as many rows as the table size. Wall-clock
// time was then dominated by result emission (server-side row construction,
// network round-trip, sqlx deserialisation, the bench's iter-and-sum loop),
// not by the per-row hash extraction or HashAggregate insert that the
// recipe is actually about. Wrapping in `count(*)` keeps the inner
// HashAggregate work identical (scan + hash + group) but emits a single row,
// so the bench measures aggregation cost rather than emission cost.
//
// The natural form (`GROUP BY value` directly on `eql_v2_encrypted`) was
// dropped from this bench in an earlier pass. The planner estimates the
// hash table against the full ~1-2 KB encrypted payload, decides it won't
// fit in the default `work_mem = 4 MB`, and falls back to `GroupAggregate`
// + sort. At 100k rows that's ~29 s vs the extractor's tens of ms. The
// natural form measured the planner's cost model, not EQL — and recommended
// practice is the extractor form anyway. See §5 of
// `docs/reference/query-performance.md` in the EQL repo.
static QUERY_TEMPLATES: &[(&str, &str)] = &[
    (
        "SELECT count(*) FROM \
         (SELECT 1 FROM {TABLE} GROUP BY eql_v2.hmac_256(value)) g",
        "count_groups",
    ),
];

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS")
        .unwrap_or_else(|_| "unknown".to_string());

    let table_suffix = match target_rows.as_str() {
        "10000" | "100000" | "1000000" | "10000000" => format!("_{}", target_rows),
        _ => String::new(),
    };
    let table_name = format!("string_encrypted{}", table_suffix);

    let pool = rt.block_on(async {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database")
    });

    let mut group = c.benchmark_group("GROUP_BY");
    group.sample_size(10);

    for (query_template, scenario) in QUERY_TEMPLATES {
        let query_str = query_template.replace("{TABLE}", &table_name);

        group.bench_function(format!("group_by/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let rows = sqlx::query(&query_str)
                    .fetch_all(&pool)
                    .await
                    .expect("group_by query failed");
                // Drain the result to force the aggregation to materialise.
                black_box(rows.iter().map(|r| r.get::<i64, _>(0)).sum::<i64>())
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
