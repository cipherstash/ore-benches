//! EQL v3 twin of `benches/combo.rs` — composite-predicate scenarios over
//! `combo_encrypted_v3_<N>` (name `eql_v3.text_match`, age
//! `eql_v3.int4_ord_ore`, category `eql_v3.text_eq`).
//!
//! The v2 scenarios filter with `name LIKE $1`; v3 removes LIKE, so every
//! scenario filters with the bloom containment recipe instead:
//! `eql_v3.match_term(name) @> eql_v3.match_term($1::eql_v3.text_match)`,
//! which engages the `GIN (eql_v3.match_term(name))` index. The rest of
//! each shape mirrors v2 with the v3 extractors (`eql_v3.ord_term(age)`
//! for the ORE ORDER BY, `eql_v3.eq_term(category)` for the GROUP BY key).
//!
//! One bound encrypted parameter per scenario (the name probe), same as
//! v2. Probe flow: storage-payload conversion (target `text_match`) — see
//! benches/exact_v3.rs for why query-payload conversion is not possible.

use cipherstash_client::{
    encryption::ScopedCipher,
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
    AutoStrategy,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{
    bench_assert, extract_indexes_used, init_scoped_cipher,
    v3::{EncryptedQueryBuilderV3, EncryptedQueryV3, TargetDomain},
    write_metadata_file, ScenarioMetadata,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use std::sync::Arc;
use tokio::runtime::Runtime;

static QUERY_TEMPLATES: &[(&str, &str)] = &[
    (
        "SELECT id FROM {TABLE} \
         WHERE eql_v3.match_term(name) @> eql_v3.match_term($1::eql_v3.text_match) \
         ORDER BY eql_v3.ord_term(age) LIMIT 10",
        "bloom_ore_order_limit",
    ),
    (
        "SELECT eql_v3.eq_term(category), count(*) FROM {TABLE} \
         WHERE eql_v3.match_term(name) @> eql_v3.match_term($1::eql_v3.text_match) \
         GROUP BY 1",
        "filtered_group_by",
    ),
    (
        "SELECT eql_v3.eq_term(category), count(*) FROM {TABLE} \
         WHERE eql_v3.match_term(name) @> eql_v3.match_term($1::eql_v3.text_match) \
         GROUP BY 1 ORDER BY count(*) DESC LIMIT 10",
        "top_n_filtered_group_by",
    ),
];

async fn build_query(
    cipher: Arc<ScopedCipher<AutoStrategy>>,
    query: &str,
    pattern: &str,
    table_name: &str,
) -> EncryptedQueryV3 {
    // Same `name` column config as encrypt_combo_v3 (unique + match); the
    // text_match conversion keeps only the bloom term the scenarios need.
    let column_config = ColumnConfig::build("name")
        .casts_as(ColumnType::Text)
        .add_index(Index::new_unique())
        .add_index(Index::new_match());

    let identifier = Identifier::new(table_name, "name");
    let target = TargetDomain::parse("text_match").expect("text_match is a v3 domain");

    EncryptedQueryBuilderV3::new(column_config, identifier, target)
        .statement(query)
        .build_query(pattern.to_string(), cipher)
        .await
        .expect("Failed to build encrypted v3 query")
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS").unwrap_or_else(|_| "unknown".to_string());

    let table_suffix = match target_rows.as_str() {
        "10000" | "100000" | "1000000" | "10000000" => format!("_{}", target_rows),
        _ => String::new(),
    };
    let table_name = format!("combo_encrypted_v3{}", table_suffix);

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

    // "Bob" matches the v2 combo bench's probe. Note the semantics shift
    // slightly with the LIKE removal: v2's `LIKE 'Bob'` requires the bloom
    // ngrams of the pattern; the v3 containment does the same bloom-side
    // work, so the filtered set is comparable.
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

    // Combo scenarios return shapes incompatible with
    // EncryptedQueryV3::execute (typed for `(i32, Json<Value>)` rows), so
    // metadata capture and the iter loop below use raw sqlx — same
    // structure as the v2 combo bench.
    let metadata = rt.block_on(async {
        let mut out = Vec::with_capacity(queries.len());
        for (i, query) in queries.iter().enumerate() {
            let (_, scenario) = QUERY_TEMPLATES[i];
            let bench_id = format!("COMBO_V3/combo/{}/{}", scenario, target_rows);
            let explain = query.explain(&pool).await.expect("EXPLAIN failed");
            let indexes_used = extract_indexes_used(&explain);
            let parameters = vec![query.parameter_json()];
            let rows = sqlx::query(&query.statement)
                .bind(Json(&query.param))
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
                version: 3,
            });
        }
        out
    });
    write_metadata_file("combo_v3", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("COMBO_V3");
    group.sample_size(10);

    for (i, query) in queries.into_iter().enumerate() {
        let (_, scenario) = QUERY_TEMPLATES[i];
        let exec_id = format!("COMBO_V3/combo/{}/{}", scenario, target_rows);

        let exec_id_inner = exec_id.clone();
        group.bench_function(format!("combo/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let rows = bench_assert(
                    sqlx::query(&query.statement)
                        .bind(Json(&query.param))
                        .fetch_all(&pool)
                        .await,
                    &exec_id_inner,
                );
                black_box(rows.len());
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
