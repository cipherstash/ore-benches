//! Encrypt small JSON data binary - encrypts generated JSON objects using CipherStash
//!
//! This binary generates small JSON objects (first_name, last_name, age, email) using
//! the fake crate and encrypts them using the cipherstash-client library with SteVec
//! indexing, storing the encrypted values in the json_small_encrypted table.
//!
//! The encrypted JSON objects support:
//! - Searchable encrypted vectors (SteVec) for term-based searches
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
    schema::{
        column::{Index, IndexType},
        ColumnConfig, ColumnType,
    },
};
use dbbenches::{IngestOptionsBuilder, WrappedJson};
use fake::{
    faker::{internet, name},
    Dummy, Fake, Rng,
};
use serde_json::json;
use std::env;

struct FakeJsonSmall;

// FIXME: cipherstash-client doesn't have a From<serde_json::Value> for Plaintext impl yet, so we use String here
impl Dummy<FakeJsonSmall> for WrappedJson {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &FakeJsonSmall, _: &mut R) -> Self {
        let value = json!({
            "first_name": name::en::FirstName().fake::<String>(),
            "last_name": name::en::LastName().fake::<String>(),
            "age": (18..=99).fake::<i32>(),
            "email": internet::en::FreeEmail().fake::<String>(),
        });
        WrappedJson(value)
    }
}

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
    let table_name = format!("json_ste_vec_small_encrypted{}", table_suffix);

    IngestOptionsBuilder::new("encrypt_ste_vec_small")
        .num_records(num_records)
        .batch_size(batch_size)
        .identifier(Identifier::new(&table_name, "value"))
        .column_config(
            ColumnConfig::build("value")
                .casts_as(ColumnType::JsonB)
                // FIXME: There is no convenience method for SteVec yet on Index
                .add_index(Index::new(IndexType::SteVec {
                    prefix: "value".to_string(),
                    term_filters: Default::default(),
                })),
        )
        .build()?
        .ingest::<WrappedJson, _>(FakeJsonSmall)
        .await?;

    Ok(())
}
