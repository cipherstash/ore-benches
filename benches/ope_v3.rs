//! EQL v3 CLLW-OPE ordering bench — NEW in v3, no v2 counterpart.
//!
//! Runs the SAME scenario shapes as benches/ore_v3.rs against
//! `integer_encrypted_ope_v3_<N>` (`eql_v3.integer_ord_ope`) so the report
//! can chart ORE (custom plpgsql btree opclass) vs OPE (native bytea btree,
//! fully inlinable) per scenario — the headline v3 fast-ordering story.
//!
//! The `op` term is REAL OPE-CLLW ciphertext: cipherstash-client 0.38.1
//! emits it for `Index::new_ope()` columns (CIP-3280/CIP-3348), so both
//! ciphertext sizes and client-side term generation are representative.
//!
//! Startup includes an order-parity assertion: the ordered scenario's
//! decrypt pass must return values sorted ascending — verifying against
//! real crypto that OPE ciphertext byte order agrees with plaintext order
//! end-to-end.

use cipherstash_client::{
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{
    bench_assert, init_scoped_cipher, init_tracing,
    v3::{encrypt_stored_v3, V3EncryptedQuery},
    write_metadata_file_in, ScenarioMetadata,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::runtime::Runtime;

static QUERY_TEMPLATES: &[(&str, i32, &str)] = &[
    (
        "SELECT id, value FROM {TABLE} WHERE value > $1 LIMIT 10",
        5000,
        "range_gt_10",
    ),
    (
        "SELECT id, value FROM {TABLE} WHERE value > $1 LIMIT 100",
        5000,
        "range_gt_100",
    ),
    (
        "SELECT id, value FROM {TABLE} WHERE value < $1 LIMIT 10",
        5000,
        "range_lt_10",
    ),
    (
        "SELECT id, value FROM {TABLE} WHERE value < $1 LIMIT 100",
        5000,
        "range_lt_100",
    ),
    (
        "SELECT id, value FROM {TABLE} \
         WHERE value < $1 \
         ORDER BY eql_v3.ord_ope_term(value) LIMIT 10",
        5000,
        "range_lt_ordered_10",
    ),
];

fn criterion_benchmark(c: &mut Criterion) {
    init_tracing();
    let rt = Runtime::new().unwrap();

    let target_rows = std::env::var("TARGET_ROWS").unwrap_or_else(|_| "unknown".to_string());
    let table_suffix = match target_rows.as_str() {
        "10000" | "100000" | "1000000" | "10000000" => format!("_{}", target_rows),
        _ => String::new(),
    };
    let table_name = format!("integer_encrypted_ope_v3{}", table_suffix);

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

    // Mirror the ingest config: Index::new_ope() so the client emits the
    // real `op` term on the Store-shaped needle.
    let column_config = ColumnConfig::build("value")
        .casts_as(ColumnType::Int)
        .add_index(Index::new_ope());
    let identifier = Identifier::new(&table_name, "value");

    let queries: Vec<V3EncryptedQuery> = rt.block_on(async {
        let mut queries = Vec::with_capacity(QUERY_TEMPLATES.len());
        for (query_template, x, _) in QUERY_TEMPLATES {
            let query_str = query_template.replace("{TABLE}", &table_name);
            let param = encrypt_stored_v3(
                Arc::clone(&cipher),
                &column_config,
                &identifier,
                *x,
                "integer_ord_ope",
            )
            .await
            .expect("failed to encrypt+convert OPE threshold");
            queries.push(V3EncryptedQuery::new(param, query_str, Arc::clone(&cipher)));
        }
        queries
    });

    // Order-parity gate: the ordered scenario's results, decrypted, must be
    // ascending — verifies against real crypto that OPE ciphertext byte
    // order agrees with plaintext order.
    rt.block_on(async {
        let ordered = queries
            .iter()
            .zip(QUERY_TEMPLATES)
            .find(|(_, (_, _, s))| *s == "range_lt_ordered_10")
            .map(|(q, _)| q)
            .expect("ordered scenario present");
        let values: Vec<i32> = ordered
            .execute_and_decrypt(&pool)
            .await
            .expect("order-parity decrypt failed");
        assert!(
            values.windows(2).all(|w| w[0] <= w[1]),
            "OPE ciphertext order does not match plaintext order: {:?}",
            values
        );
        eprintln!(
            "ope_v3 bench: order-parity OK over {} decrypted rows",
            values.len()
        );
    });

    let metadata = rt.block_on(async {
        let mut out = Vec::with_capacity(queries.len());
        for (i, query) in queries.iter().enumerate() {
            let (_, _, scenario) = QUERY_TEMPLATES[i];
            let bench_id = format!("OPE/ope/{}/{}", scenario, target_rows);
            let (explain, indexes_used, rows_returned) = query
                .capture_metadata(&pool)
                .await
                .expect("metadata capture failed");
            out.push(ScenarioMetadata {
                id: bench_id,
                query: query.statement.clone(),
                parameters: vec![query.parameter_json().expect("serialise parameter")],
                explain,
                indexes_used,
                rows_returned,
            });
        }
        out
    });
    write_metadata_file_in("results/query/v3", "ope", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("OPE");
    group.sample_size(10);

    for (i, query) in queries.into_iter().enumerate() {
        let (_, _, scenario) = QUERY_TEMPLATES[i];
        let exec_id = format!("OPE/ope/{}/{}", scenario, target_rows);
        let decrypt_id = format!("OPE/ope_decrypt/{}/{}", scenario, target_rows);

        let exec_id_inner = exec_id.clone();
        group.bench_function(format!("ope/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _: Vec<_> = bench_assert(query.execute(&pool).await, &exec_id_inner);
            })
        });

        let decrypt_id_inner = decrypt_id.clone();
        group.bench_function(format!("ope_decrypt/{}/{}", scenario, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _r: Vec<i32> = black_box(bench_assert(
                    query.execute_and_decrypt(&pool).await,
                    &decrypt_id_inner,
                ));
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
