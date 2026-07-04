use cipherstash_client::{
    encryption::ScopedCipher,
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
    AutoStrategy,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{
    bench_assert, extract_indexes_used, init_scoped_cipher, write_metadata_file, EncryptedQuery,
    EncryptedQueryBuilder, ScenarioMetadata,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use std::sync::Arc;
use tokio::runtime::Runtime;

// Composite-predicate scenarios over `combo_encrypted_<N>` — a three-column
// table carrying `name` (match + hmac), `age` (ORE), `category` (hmac). Each
// scenario binds ONE encrypted parameter (the name pattern for LIKE) and
// references the other encrypted columns via extractor expressions in
// WHERE / ORDER BY / GROUP BY. That keeps the build_query path simple
// (single-column ColumnConfig) while still exercising composite-predicate
// shapes the EQL query-performance guide §6 describes.
//
// Scenarios:
//
//   * bloom_ore_order_limit — bloom (LIKE) + ORE ORDER BY + LIMIT
//     "Find the 10 youngest customers whose first name matches a pattern."
//     The GIN bloom filter on `name` engages for the LIKE predicate; the
//     btree on `eql_v2.ore_block_u64_8_256(age)` engages for the ORDER BY
//     because the sort key matches the index expression syntactically
//     (hybrid form — same rule as §4 of the perf guide).
//
//   * filtered_group_by — bloom (LIKE) + HashAggregate on hmac category
//     "Group customers by category, filtered by first-name pattern." Bloom
//     filters the input set first; HashAggregate then groups the filtered
//     rows by the 32-byte HMAC `category` extractor — small post-filter
//     set, small group key, in-memory aggregate.
//
//   * top_n_filtered_group_by — same as filtered_group_by with an outer
//     `ORDER BY count(*) DESC LIMIT 10`. The dashboard analytic shape:
//     "top 10 categories for customers matching X". Always emits 10 rows.

static QUERY_TEMPLATES: &[(&str, &str)] = &[
    (
        "SELECT id FROM {TABLE} \
         WHERE name LIKE $1 \
         ORDER BY eql_v2.ore_block_u64_8_256(age) LIMIT 10",
        "bloom_ore_order_limit",
    ),
    (
        "SELECT eql_v2.hmac_256(category), count(*) FROM {TABLE} \
         WHERE name LIKE $1 \
         GROUP BY 1",
        "filtered_group_by",
    ),
    (
        "SELECT eql_v2.hmac_256(category), count(*) FROM {TABLE} \
         WHERE name LIKE $1 \
         GROUP BY 1 ORDER BY count(*) DESC LIMIT 10",
        "top_n_filtered_group_by",
    ),
];

async fn build_query(
    cipher: Arc<ScopedCipher<AutoStrategy>>,
    query: &str,
    pattern: &str,
    table_name: &str,
) -> EncryptedQuery {
    // ColumnConfig describes `name`'s indexes — match for LIKE / bloom,
    // unique for equality. We pick the match index type explicitly below so
    // build_query produces a bloom-filter-shaped ciphertext for the
    // parameter.
    let column_config = ColumnConfig::build("name")
        .casts_as(ColumnType::Text)
        .add_index(Index::new_unique())
        .add_index(Index::new_match());

    let identifier = Identifier::new(table_name, "name");

    EncryptedQueryBuilder::new(column_config, identifier)
        .index_type(Index::new_match().index_type)
        .statement(query)
        .build_query(pattern, cipher)
        .await
        .expect("Failed to build encrypted query")
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS").unwrap_or_else(|_| "unknown".to_string());

    let table_suffix = match target_rows.as_str() {
        "10000" | "100000" | "1000000" | "10000000" => format!("_{}", target_rows),
        _ => String::new(),
    };
    let table_name = format!("combo_encrypted{}", table_suffix);

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

    // "Bob" is the standard first-name pattern used by match.rs; same here
    // for consistency. The bloom index trigrams should match a non-trivial
    // fraction of names produced by `Name(EN)`.
    let pattern = "Bob";

    let queries = rt.block_on(async {
        let mut queries = Vec::with_capacity(QUERY_TEMPLATES.len());
        for (query_template, _) in QUERY_TEMPLATES {
            let query_str = query_template.replace("{TABLE}", &table_name);
            let query = build_query(Arc::clone(&cipher), &query_str, pattern, &table_name).await;
            queries.push(query);
        }
        queries
    });

    // Capture per-scenario metadata before the criterion loop. Combo
    // scenarios return shapes incompatible with EncryptedQuery::execute
    // (which is typed for `Vec<(i32, Json<EqlCiphertext>)>`), so we use
    // sqlx directly here and in the iter loop below.
    let metadata = rt.block_on(async {
        let mut out = Vec::with_capacity(queries.len());
        for (i, query) in queries.iter().enumerate() {
            let (_, scenario) = QUERY_TEMPLATES[i];
            let bench_id = format!("COMBO/combo/{}/{}", scenario, target_rows);
            let explain = query.explain(&pool).await.expect("EXPLAIN failed");
            let indexes_used = extract_indexes_used(&explain);
            let parameters = vec![query.parameter_json().expect("serialise parameter")];
            let rows = sqlx::query(&query.statement)
                .bind(Json(&query.eql))
                .fetch_all(&pool)
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
            });
        }
        out
    });
    write_metadata_file("combo", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("COMBO");
    group.sample_size(10);

    for (i, query) in queries.into_iter().enumerate() {
        let (_, scenario) = QUERY_TEMPLATES[i];
        let exec_id = format!("COMBO/combo/{}/{}", scenario, target_rows);

        let exec_id_inner = exec_id.clone();
        group.bench_function(format!("combo/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let rows = bench_assert(
                    sqlx::query(&query.statement)
                        .bind(Json(&query.eql))
                        .fetch_all(&pool)
                        .await,
                    &exec_id_inner,
                );
                // Drain the result set; we don't care about the exact
                // payload shape, only that the query executed and returned
                // rows.
                black_box(rows.len());
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
