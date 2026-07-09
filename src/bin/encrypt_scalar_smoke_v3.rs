//! Populate `scalar_smoke_v3` — one column per v3 scalar family the v2
//! benches never covered (date, timestamp, numeric, bigint, double,
//! boolean). Smoke coverage only: catches per-family CHECK/conversion
//! breakage, not tier curves.
//!
//! Encrypts each family through the pinned v2 client (ORE index for the
//! ordered families, no index for boolean) and converts to the family's
//! `_ord` domain (`boolean` is storage-only). A conversion failure aborts
//! with the family name — e.g. if the v2 client cannot emit `ob` for a type,
//! that is a release-relevant finding, not something to paper over.
//!
//! Environment variables: DATABASE_URL, NUM_RECORDS (default 10000),
//! CS_CLIENT_ID / CS_CLIENT_KEY / CS_WORKSPACE_CRN.

use anyhow::{Context, Result};
use chrono::{Duration, TimeZone, Utc};
use cipherstash_client::{
    encryption::Plaintext,
    eql::{encrypt_eql, EqlCiphertext, EqlOperation, EqlOutput, Identifier, PreparedPlaintext},
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::{init_scoped_cipher, v3::to_v3_stored};
use fake::Fake;
use rust_decimal::Decimal;
use sqlx::{postgres::PgPoolOptions, types::Json, QueryBuilder};
use std::borrow::Cow;
use std::env;

const TABLE: &str = "scalar_smoke_v3";
const FAMILIES: [&str; 6] = [
    "date_ord",
    "timestamp_ord",
    "numeric_ord",
    "bigint_ord",
    "double_ord",
    "boolean",
];

#[tokio::main]
async fn main() -> Result<()> {
    dbbenches::init_tracing();

    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL environment variable must be set")?;
    let num_records: i32 = env::var("NUM_RECORDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("NUM_RECORDS must be a valid integer");
    let batch_size: usize = 1000;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let scoped_cipher = init_scoped_cipher().await?;

    let ord = |name: &str, ty: ColumnType| {
        ColumnConfig::build(name)
            .casts_as(ty)
            .add_index(Index::new_ore())
    };
    let configs = [
        ord("date_val", ColumnType::Date),
        ord("timestamp_val", ColumnType::Timestamp),
        ord("numeric_val", ColumnType::Decimal),
        ord("bigint_val", ColumnType::BigInt),
        ord("double_val", ColumnType::Float),
        // boolean is storage-only by design (two-value cardinality — any
        // searchable index leaks the plaintext).
        ColumnConfig::build("boolean_val").casts_as(ColumnType::Boolean),
    ];
    let idents: Vec<Identifier> = configs
        .iter()
        .map(|c| Identifier::new(TABLE, &c.name))
        .collect();

    let epoch = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();

    for batch_start in (0..num_records).step_by(batch_size) {
        let batch_end = (batch_start + batch_size as i32).min(num_records);
        let batch_count = batch_end - batch_start;

        let mut prepared = Vec::with_capacity((batch_count * 6) as usize);
        for _ in 0..batch_count {
            let days: i64 = (0..10_000).fake();
            let secs: i64 = (0..800_000_000).fake();
            let values: [Plaintext; 6] = [
                Plaintext::NaiveDate(Some((epoch + Duration::days(days)).date_naive())),
                Plaintext::Timestamp(Some(epoch + Duration::seconds(secs))),
                Plaintext::Decimal(Some(Decimal::new((0..10_000_000i64).fake(), 2))),
                Plaintext::BigInt(Some((0..i64::MAX).fake())),
                Plaintext::Float(Some((0.0..1_000_000.0f64).fake())),
                Plaintext::Boolean(Some((0..2).fake::<u8>() == 1)),
            ];
            for ((config, ident), value) in configs.iter().zip(&idents).zip(values) {
                prepared.push(PreparedPlaintext::new(
                    Cow::Borrowed(config),
                    ident.clone(),
                    value,
                    EqlOperation::Store,
                ));
            }
        }

        let out = encrypt_eql(scoped_cipher.clone(), prepared, &Default::default()).await?;

        let ciphertexts: Vec<EqlCiphertext> = out
            .into_iter()
            .map(|o| match o {
                EqlOutput::Store(ct) => ct,
                EqlOutput::Query(_) => {
                    unreachable!("storage batch must yield EqlOutput::Store")
                }
            })
            .collect();

        let rows = ciphertexts
            .chunks_exact(6)
            .map(|chunk| {
                chunk
                    .iter()
                    .zip(FAMILIES)
                    .map(|(ct, family)| {
                        to_v3_stored(ct, family).with_context(|| {
                            format!("v3 conversion failed for family `{}`", family)
                        })
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()?;

        QueryBuilder::new(format!(
            "INSERT INTO {} (date_val, timestamp_val, numeric_val, bigint_val, double_val, boolean_val) ",
            TABLE
        ))
        .push_values(rows, |mut b, row| {
            for v in row {
                b.push_bind(Json(v));
            }
        })
        .build()
        .execute(&pool)
        .await?;
    }

    println!("scalar_smoke_v3: inserted {} rows across {:?}", num_records, FAMILIES);
    Ok(())
}
