//! Encrypt three-column combo rows (`name`, `age`, `category`) into the
//! `combo_encrypted_*` tables. Used by `benches/combo.rs` for composite
//! predicate scenarios — bloom + ORE order + LIMIT, filtered GROUP BY,
//! top-N filtered GROUP BY.
//!
//! Per row we generate:
//!   * `name`     — `fake::Name<EN>` (random English name, ~unique)
//!   * `age`      — uniform 18..=90 (low-cardinality numeric, ORE-indexable)
//!   * `category` — `dbbenches::FakeCategory` (CAT_001..CAT_250, 250 buckets)
//!
//! Each row produces three `PreparedPlaintext` entries which we encrypt in
//! one `encrypt_eql` call per batch, then split back into 3-tuples for the
//! `INSERT INTO ... (name, age, category)` statement.
//!
//! Environment variables:
//! - DATABASE_URL: PostgreSQL connection string
//! - NUM_RECORDS: Number of records to generate (default: 10000)
//! - TABLE_SUFFIX: Optional suffix for table name (e.g., _10000)
//! - CS_CLIENT_ID / CS_CLIENT_KEY / CS_WORKSPACE_CRN: CipherStash credentials

use anyhow::{Context, Result};
use cipherstash_client::{
    encryption::Plaintext,
    eql::{encrypt_eql, EqlOperation, Identifier, PreparedPlaintext},
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::{init_scoped_cipher, FakeCategory};
use fake::{faker::name::raw::Name, locales::EN, Fake};
use sqlx::{postgres::PgPoolOptions, types::Json, QueryBuilder};
use std::borrow::Cow;
use std::env;

// Re-init scoped cipher every 200_000 rows (same threshold as lib.rs::ingest)
// to stay under the ZeroKMS TTL window for 10M-row preparations.
const REINIT_EVERY_ROWS: i32 = 200_000;

#[tokio::main]
async fn main() -> Result<()> {
    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL environment variable must be set")?;
    let num_records: i32 = env::var("NUM_RECORDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("NUM_RECORDS must be a valid integer");
    let table_suffix = env::var("TABLE_SUFFIX").unwrap_or_default();
    let table_name = format!("combo_encrypted{}", table_suffix);
    let batch_size: usize = 1000;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let mut scoped_cipher = init_scoped_cipher().await?;
    let mut rows_since_reinit: i32 = 0;

    let name_config = ColumnConfig::build("name")
        .casts_as(ColumnType::Text)
        .add_index(Index::new_unique())
        .add_index(Index::new_match());
    let age_config = ColumnConfig::build("age")
        .casts_as(ColumnType::Int)
        .add_index(Index::new_ore());
    let category_config = ColumnConfig::build("category")
        .casts_as(ColumnType::Text)
        .add_index(Index::new_unique());

    let name_ident = Identifier::new(&table_name, "name");
    let age_ident = Identifier::new(&table_name, "age");
    let category_ident = Identifier::new(&table_name, "category");

    for batch_start in (0..num_records).step_by(batch_size) {
        if rows_since_reinit >= REINIT_EVERY_ROWS {
            eprintln!(
                "encrypt_combo: refreshing scoped cipher after {} rows ({}/{} total)",
                rows_since_reinit, batch_start, num_records
            );
            scoped_cipher = init_scoped_cipher().await?;
            rows_since_reinit = 0;
        }

        let batch_end = (batch_start + batch_size as i32).min(num_records);
        let batch_count = batch_end - batch_start;

        let mut prepared = Vec::with_capacity((batch_count * 3) as usize);
        for _ in 0..batch_count {
            let name: String = Name(EN).fake();
            // Realistic age range — uniform 18..=90 gives ~72 distinct
            // buckets, low enough cardinality that a HashAggregate over the
            // ORE-block extractor would group meaningfully but high enough
            // that range predicates like `age > 30` are selective.
            let age: i32 = (18..=90).fake();
            let category: String = FakeCategory.fake();

            prepared.push(PreparedPlaintext::new(
                Cow::Borrowed(&name_config),
                name_ident.clone(),
                Plaintext::new(name),
                EqlOperation::Store,
            ));
            prepared.push(PreparedPlaintext::new(
                Cow::Borrowed(&age_config),
                age_ident.clone(),
                Plaintext::new(age),
                EqlOperation::Store,
            ));
            prepared.push(PreparedPlaintext::new(
                Cow::Borrowed(&category_config),
                category_ident.clone(),
                Plaintext::new(category),
                EqlOperation::Store,
            ));
        }

        let out = encrypt_eql(scoped_cipher.clone(), prepared, &Default::default()).await?;

        // encrypt_eql preserves input order; chunks of 3 reassemble per-row
        // (name, age, category) tuples for the multi-column INSERT.
        let rows: Vec<(_, _, _)> = out
            .chunks_exact(3)
            .map(|c| (c[0].clone(), c[1].clone(), c[2].clone()))
            .collect();

        QueryBuilder::new(format!(
            "INSERT INTO {} (name, age, category) ",
            table_name
        ))
        .push_values(rows, |mut b, (name, age, category)| {
            b.push_bind(Json(name));
            b.push_bind(Json(age));
            b.push_bind(Json(category));
        })
        .build()
        .execute(&pool)
        .await?;

        rows_since_reinit += batch_count;
    }

    Ok(())
}
