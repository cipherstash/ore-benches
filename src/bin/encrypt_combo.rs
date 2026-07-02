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
    eql::{encrypt_eql, EqlCiphertext, EqlOperation, EqlOutput, Identifier, PreparedPlaintext},
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::{init_scoped_cipher, FakeCategory};
use fake::{faker::name::raw::Name, locales::EN, Fake};
use sqlx::{postgres::PgPoolOptions, types::Json, QueryBuilder};
use std::borrow::Cow;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Honour RUST_LOG for cipherstash-client / zerokms-protocol trace!
    // emissions. The cipherstash-client traces the full ViturRequestError
    // (kind + message + underlying transport error) at the failure point
    // in vitur_client::generate_keys; without a subscriber that line is a
    // no-op. The try_init is a no-op if a subscriber was already installed
    // upstream — keeps it safe to leave in.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .try_init();

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

    // Init once and reuse for the binary lifetime — see lib.rs::ingest
    // for the rationale.
    let scoped_cipher = init_scoped_cipher().await?;

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

        // Every PreparedPlaintext above used EqlOperation::Store, so encrypt_eql
        // yields only EqlOutput::Store. cipherstash-client splits the storage /
        // query payload shapes (since 0.34.1-alpha.9) — unwrap to the storage
        // ciphertext (which, unlike EqlOutput, is Clone) before reassembling rows.
        let ciphertexts: Vec<EqlCiphertext> = out
            .into_iter()
            .map(|o| match o {
                EqlOutput::Store(ct) => ct,
                EqlOutput::Query(_) => {
                    unreachable!("storage batch must yield EqlOutput::Store")
                }
            })
            .collect();

        // encrypt_eql preserves input order; chunks of 3 reassemble per-row
        // (name, age, category) tuples for the multi-column INSERT.
        let rows: Vec<(_, _, _)> = ciphertexts
            .chunks_exact(3)
            .map(|c| (c[0].clone(), c[1].clone(), c[2].clone()))
            .collect();

        QueryBuilder::new(format!("INSERT INTO {} (name, age, category) ", table_name))
            .push_values(rows, |mut b, (name, age, category)| {
                b.push_bind(Json(name));
                b.push_bind(Json(age));
                b.push_bind(Json(category));
            })
            .build()
            .execute(&pool)
            .await?;
    }

    Ok(())
}
