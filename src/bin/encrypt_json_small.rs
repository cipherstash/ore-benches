//! Encrypt small JSON data binary - encrypts generated JSON objects using CipherStash
//!
//! This binary generates small JSON objects (first_name, last_name, age, email) using
//! the fake crate and encrypts them using the cipherstash-client library WITHOUT any
//! searchable indexes. It is the baseline that pairs with `encrypt_ste_vec_small` to
//! quantify the ingest cost of SteVec indexing.
//!
//! Environment variables:
//! - DATABASE_URL: PostgreSQL connection string
//! - NUM_RECORDS: Number of records to generate (default: 10000)
//! - BATCH_SIZE: Number of records per batch insert (default: 1000)
//! - TABLE_SUFFIX: Optional suffix for table name (e.g., _10000)
//! - CS_CLIENT_ID: CipherStash client ID
//! - CS_CLIENT_KEY: CipherStash client key
//! - CS_WORKSPACE_CRN: CipherStash workspace CRN

use anyhow::Result;
use cipherstash_client::{
    eql::Identifier,
    schema::{ColumnConfig, ColumnType},
};
use dbbenches::{FakeJsonSmall, IngestOptionsBuilder, WrappedJson};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let num_records: i32 = env::var("NUM_RECORDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("NUM_RECORDS must be a valid integer");

    let batch_size: usize = env::var("BATCH_SIZE")
        .unwrap_or_else(|_| "1000".to_string())
        .parse()
        .expect("BATCH_SIZE must be a valid integer");

    let table_suffix = env::var("TABLE_SUFFIX").unwrap_or_default();
    let table_name = format!("json_small_encrypted{}", table_suffix);

    IngestOptionsBuilder::new("encrypt_json_small")
        .num_records(num_records)
        .batch_size(batch_size)
        .identifier(Identifier::new(&table_name, "value"))
        .column_config(
            // No searchable indexes — pure encryption-and-ingest baseline.
            ColumnConfig::build("value").casts_as(ColumnType::JsonB),
        )
        .build()?
        .ingest::<WrappedJson, _>(FakeJsonSmall)
        .await?;

    Ok(())
}
