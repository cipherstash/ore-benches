//! Encrypt large JSON data binary - encrypts generated complex JSON objects using CipherStash
//!
//! This binary generates large, complex JSON objects containing user information, company
//! details, addresses, and order history using the fake crate. The objects are encrypted
//! using the cipherstash-client library WITHOUT any searchable indexes. It is the baseline
//! that pairs with `encrypt_ste_vec_large` to quantify the ingest cost of SteVec indexing
//! on large payloads.
//!
//! Environment variables:
//! - DATABASE_URL: PostgreSQL connection string
//! - NUM_RECORDS: Number of records to generate (default: 10000)
//! - BATCH_SIZE: Number of records per batch insert (default: 1000)
//! - CS_CLIENT_ID: CipherStash client ID
//! - CS_CLIENT_KEY: CipherStash client key
//! - CS_WORKSPACE_CRN: CipherStash workspace CRN

use anyhow::Result;
use cipherstash_client::{
    eql::Identifier,
    schema::{ColumnConfig, ColumnType},
};
use dbbenches::{FakeJsonLarge, IngestOptionsBuilder, WrappedJson};
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

    IngestOptionsBuilder::new("encrypt_json_large")
        .num_records(num_records)
        .batch_size(batch_size)
        .identifier(Identifier::new("json_large_encrypted", "value"))
        .column_config(
            // No searchable indexes — pure encryption-and-ingest baseline.
            ColumnConfig::build("value").casts_as(ColumnType::JsonB),
        )
        .build()?
        .ingest::<WrappedJson, _>(FakeJsonLarge)
        .await?;

    Ok(())
}
