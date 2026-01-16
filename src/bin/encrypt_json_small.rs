//! Encrypt data binary - encrypts integers from integer_plaintext table using CipherStash
//!
//! This binary reads plaintext integers from the integer_plaintext table and encrypts
//! them using the cipherstash-client library's `encrypt_eql` function, storing the
//! encrypted values in the integer_encrypted table.
//!
//! Environment variables required:
//! - DATABASE_URL: PostgreSQL connection string
//! - CS_CLIENT_ID: CipherStash client ID
//! - CS_CLIENT_KEY: CipherStash client key  
//! - CS_WORKSPACE_CRN: CipherStash workspace CRN
//!
//! TODO: The CipherStash client API needs to be properly configured based on the
//! actual cipherstash-client crate API. This is a placeholder implementation that
//! outlines the structure. Refer to cipherstash-client documentation at:
//! https://docs.rs/cipherstash-client

use anyhow::Result;
use cipherstash_client::{
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::IngestOptionsBuilder;
use fake::{Dummy, Fake, Rng, faker::{name, internet}};
use std::env;
use serde_json::json;

struct FakeJsonSmall;

// FIXME: cipherstash-client doesn't have a From<serde_json::Value> for Plaintext impl yet, so we use String here
impl Dummy<FakeJsonSmall> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &FakeJsonSmall, _: &mut R) -> Self {
        json!({
            "first_name": name::en::FirstName().fake::<String>(),
            "last_name": name::en::LastName().fake::<String>(),
            "age": (18..=99).fake::<i32>(),
            "email": internet::en::FreeEmail().fake::<String>(),
        }).to_string()
    }
}


#[tokio::main]
async fn main() -> Result<()> {
    let num_records: i32 = env::var("NUM_RECORDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("NUM_RECORDS must be a valid integer");

    IngestOptionsBuilder::new()
        .num_records(num_records)
        .batch_size(1000)
        .identifier(Identifier::new("json_small_encrypted", "value"))
        .column_config(
            ColumnConfig::build("value")
                .casts_as(ColumnType::Utf8Str)
                .add_index(Index::new_unique())
                .add_index(Index::new_match()),
        )
        .build()?
        .ingest::<String, _>(FakeJsonSmall)
        .await?;

    Ok(())
}
