//! Encrypt low-cardinality categorical data (`CAT_001`..`CAT_250`) into the
//! `category_encrypted_*` tables. Used by the realistic-GROUP-BY scenarios in
//! `benches/group_by.rs` — see `dbbenches::FakeCategory` for the generator.
//!
//! Environment variables:
//! - DATABASE_URL: PostgreSQL connection string
//! - NUM_RECORDS: Number of records to generate (default: 10000)
//! - TABLE_SUFFIX: Optional suffix for table name (e.g., _10000)
//! - CS_CLIENT_ID / CS_CLIENT_KEY / CS_WORKSPACE_CRN: CipherStash credentials

use anyhow::Result;
use cipherstash_client::{
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::{FakeCategory, IngestOptionsBuilder};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let num_records: i32 = env::var("NUM_RECORDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("NUM_RECORDS must be a valid integer");

    let table_suffix = env::var("TABLE_SUFFIX").unwrap_or_default();
    let table_name = format!("category_encrypted{}", table_suffix);

    IngestOptionsBuilder::new("encrypt_category")
        .num_records(num_records)
        .batch_size(1000)
        .identifier(Identifier::new(&table_name, "value"))
        .column_config(
            ColumnConfig::build("value")
                .casts_as(ColumnType::Text)
                .add_index(Index::new_unique()),
        )
        .build()?
        .ingest::<String, FakeCategory>(FakeCategory)
        .await?;

    Ok(())
}
