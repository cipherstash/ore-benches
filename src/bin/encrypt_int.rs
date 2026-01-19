//! Encrypt integer data binary - encrypts generated integers using CipherStash
//!
//! This binary generates random integers using the fake crate and encrypts them
//! using the cipherstash-client library with ORE (Order-Revealing Encryption),
//! storing the encrypted values in the integer_encrypted table (or a suffixed
//! variant based on TABLE_SUFFIX).
//!
//! The encrypted integers support:
//! - Exact match queries
//! - Range queries (>, <, >=, <=)
//! - Ordered queries (ORDER BY)
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
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::IngestOptionsBuilder;
use fake::Faker;
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
    let table_name = format!("integer_encrypted{}", table_suffix);

    IngestOptionsBuilder::new("encrypt_int")
        .num_records(num_records)
        .batch_size(batch_size)
        .identifier(Identifier::new(&table_name, "value"))
        .column_config(
            ColumnConfig::build("value")
                .casts_as(ColumnType::Int)
                .add_index(Index::new_ore()),
        )
        .build()?
        .ingest::<i32, _>(Faker)
        .await?;

    Ok(())
}
