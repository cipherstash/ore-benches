//! New-family smoke bench (v3-only, no v2 counterpart, 10k rows, no tiers).
//!
//! One range scenario + one extractor-ordered scenario per ordered scalar
//! family the v2 benches never covered (date / timestamp / numeric / bigint
//! / double), plus a select-back scenario for storage-only `boolean`.
//! Purpose: catch per-family CHECK / operator-routing / inlining breakage
//! and index engagement — NOT to produce tier curves. Runs against the
//! fixed `scalar_smoke_v3` table populated by `prepare:scalar_smoke_v3`.
//!
//! Startup builds a functional `eql_v3.ord_term(col)` btree per ordered
//! family so the ordered scenario's `indexes_used` proves the family's
//! whole extractor chain inlines.

use chrono::{Duration, TimeZone, Utc};
use cipherstash_client::{
    encryption::Plaintext,
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbbenches::{
    bench_assert, init_scoped_cipher, init_tracing,
    v3::{encrypt_stored_v3, V3EncryptedQuery},
    write_metadata_file_in, ScenarioMetadata,
};
use rust_decimal::Decimal;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::runtime::Runtime;

const TABLE: &str = "scalar_smoke_v3";

fn criterion_benchmark(c: &mut Criterion) {
    init_tracing();
    let rt = Runtime::new().unwrap();

    // Fixed-size table; keep a tier-like id segment for report consistency.
    let target_rows = "10000".to_string();

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

    let epoch = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();

    // (family, column, ColumnType, mid-range needle) — needle sits near the
    // middle of the ingest distribution (see encrypt_scalar_smoke_v3) so the
    // range scenarios return rows.
    let families: Vec<(&str, &str, ColumnType, Plaintext)> = vec![
        (
            "date_ord",
            "date_val",
            ColumnType::Date,
            Plaintext::NaiveDate(Some((epoch + Duration::days(5_000)).date_naive())),
        ),
        (
            "timestamp_ord",
            "timestamp_val",
            ColumnType::Timestamp,
            Plaintext::Timestamp(Some(epoch + Duration::seconds(400_000_000))),
        ),
        (
            "numeric_ord",
            "numeric_val",
            ColumnType::Decimal,
            Plaintext::Decimal(Some(Decimal::new(5_000_000, 2))),
        ),
        (
            "bigint_ord",
            "bigint_val",
            ColumnType::BigInt,
            Plaintext::BigInt(Some(i64::MAX / 2)),
        ),
        (
            "double_ord",
            "double_val",
            ColumnType::Float,
            Plaintext::Float(Some(500_000.0)),
        ),
    ];

    // Per-family functional ord_term indexes.
    rt.block_on(async {
        for (_, col, _, _) in &families {
            sqlx::query(&format!("DROP INDEX IF EXISTS {TABLE}_{col}_ord_idx"))
                .execute(&pool)
                .await
                .expect("drop stale smoke index");
            sqlx::query(&format!(
                "CREATE INDEX {TABLE}_{col}_ord_idx ON {TABLE} (eql_v3.ord_term({col}))"
            ))
            .execute(&pool)
            .await
            .expect("create smoke ord index");
        }
        sqlx::query(&format!("ANALYZE {TABLE}"))
            .execute(&pool)
            .await
            .expect("ANALYZE smoke table");
    });

    // Build queries: (id_segment, statement, param)
    let queries: Vec<(String, V3EncryptedQuery)> = rt.block_on(async {
        let mut out = Vec::new();
        for (family, col, ty, needle) in families {
            let config = ColumnConfig::build(col)
                .casts_as(ty)
                .add_index(Index::new_ore());
            let ident = Identifier::new(TABLE, col);
            let param = encrypt_stored_v3(
                Arc::clone(&cipher),
                &config,
                &ident,
                needle,
                family,
            )
            .await
            .unwrap_or_else(|e| panic!("needle encrypt+convert failed for `{family}`: {e:?}"));

            out.push((
                format!("{family}/range_gt_10"),
                V3EncryptedQuery::new(
                    param.clone(),
                    format!("SELECT id, {col} FROM {TABLE} WHERE {col} > $1 LIMIT 10"),
                    Arc::clone(&cipher),
                ),
            ));
            out.push((
                format!("{family}/range_gt_ordered_10"),
                V3EncryptedQuery::new(
                    param,
                    format!(
                        "SELECT id, {col} FROM {TABLE} WHERE {col} > $1 \
                         ORDER BY eql_v3.ord_term({col}) LIMIT 10"
                    ),
                    Arc::clone(&cipher),
                ),
            ));
        }
        out
    });

    let metadata = rt.block_on(async {
        let mut out = Vec::with_capacity(queries.len() + 1);
        for (segment, query) in &queries {
            let bench_id = format!("SMOKE_V3/smoke/{}/{}", segment, target_rows);
            let (explain, indexes_used, rows_returned) = query
                .capture_metadata(&pool)
                .await
                .unwrap_or_else(|e| panic!("metadata capture failed for `{segment}`: {e:?}"));
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
    write_metadata_file_in("results/query/v3", "scalar_smoke", &target_rows, metadata)
        .expect("failed to write bench metadata sidecar");

    let mut group = c.benchmark_group("SMOKE_V3");
    group.sample_size(10);

    for (segment, query) in queries {
        let bench_id = format!("SMOKE_V3/smoke/{}/{}", segment, target_rows);
        let inner_id = bench_id.clone();
        group.bench_function(format!("smoke/{}/{}", segment, target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let _: Vec<_> = bench_assert(query.execute(&pool).await, &inner_id);
            })
        });
    }

    // Storage-only boolean: select-back is the entire supported surface.
    {
        let q = format!("SELECT id, boolean_val FROM {TABLE} LIMIT 10");
        let id = format!("SMOKE_V3/smoke/boolean/select_back/{}", target_rows);
        let inner_id = id.clone();
        group.bench_function(format!("smoke/boolean/select_back/{}", target_rows), |b| {
            b.to_async(&rt).iter(|| async {
                let rows = bench_assert(sqlx::query(&q).fetch_all(&pool).await, &inner_id);
                black_box(rows.len());
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
