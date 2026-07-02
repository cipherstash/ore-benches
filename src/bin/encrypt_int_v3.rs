//! EQL v3 twin of `encrypt_int`: encrypts random `i32` values via the
//! existing cipherstash-client (v2 wire) pipeline, converts each storage
//! payload with `eql_bindings::from_v2`, and inserts into
//! `integer_encrypted_v3` (or a `TABLE_SUFFIX` variant).
//!
//! Target domain: `eql_v3.int4_ord_ore` (ob) — same ColumnType::Int / ORE
//! index configuration as the v2 bin, so the encryption workload is
//! identical and the ingest numbers differ only by the from_v2 conversion.
//!
//! Environment variables: DATABASE_URL, NUM_RECORDS (default 10000),
//! TABLE_SUFFIX, CS_CLIENT_ID / CS_CLIENT_KEY / CS_WORKSPACE_CRN.

use anyhow::Result;
use cipherstash_client::{
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::{v3::TargetDomain, IngestOptionsBuilder};
use fake::Faker;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let num_records: i32 = env::var("NUM_RECORDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("NUM_RECORDS must be a valid integer");

    let table_suffix = env::var("TABLE_SUFFIX").unwrap_or_default();
    let table_name = format!("integer_encrypted_v3{}", table_suffix);

    IngestOptionsBuilder::new("encrypt_int_v3")
        .num_records(num_records)
        .batch_size(1000)
        .identifier(Identifier::new(&table_name, "value"))
        .column_config(
            ColumnConfig::build("value")
                .casts_as(ColumnType::Int)
                .add_index(Index::new_ore()),
        )
        .convert_to_v3(TargetDomain::parse("int4_ord_ore").expect("int4_ord_ore is a v3 domain"))
        .build()?
        .ingest::<i32, _>(Faker)
        .await?;

    Ok(())
}
