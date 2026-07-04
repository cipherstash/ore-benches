//! v3 sibling of `encrypt_int`: random i32s into `integer_encrypted_v3_*`
//! as `eql_v3.integer_ord` payloads (ORE block term, custom btree opclass).
//!
//! Environment variables: DATABASE_URL, NUM_RECORDS, TABLE_SUFFIX,
//! V3_CONVERT_ONLY, CS_CLIENT_ID / CS_CLIENT_KEY / CS_WORKSPACE_CRN.

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
    dbbenches::init_tracing();

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
        .convert_to_v3(TargetDomain::parse("integer_ord").expect("integer_ord is a v3 domain"))
        .build()?
        .ingest::<i32, _>(Faker)
        .await?;

    Ok(())
}
