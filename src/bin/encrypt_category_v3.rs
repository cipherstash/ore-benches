//! v3 sibling of `encrypt_category`: low-cardinality categorical strings
//! (`CAT_001`..`CAT_250`) into `category_encrypted_v3_*` as
//! `public.text_eq` payloads (hm only — GROUP BY / equality scenarios).
//!
//! Environment variables: DATABASE_URL, NUM_RECORDS, TABLE_SUFFIX,
//! V3_CONVERT_ONLY, CS_CLIENT_ID / CS_CLIENT_KEY / CS_WORKSPACE_CRN.

use anyhow::Result;
use cipherstash_client::{
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::{v3::TargetDomain, FakeCategory, IngestOptionsBuilder};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dbbenches::init_tracing();

    let num_records: i32 = env::var("NUM_RECORDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("NUM_RECORDS must be a valid integer");

    let table_suffix = env::var("TABLE_SUFFIX").unwrap_or_default();
    let table_name = format!("category_encrypted_v3{}", table_suffix);

    IngestOptionsBuilder::new("encrypt_category_v3")
        .num_records(num_records)
        .batch_size(1000)
        .identifier(Identifier::new(&table_name, "value"))
        .column_config(
            ColumnConfig::build("value")
                .casts_as(ColumnType::Text)
                .add_index(Index::new_unique()),
        )
        .convert_to_v3(TargetDomain::parse("text_eq").expect("text_eq is a v3 domain"))
        .build()?
        .ingest::<String, FakeCategory>(FakeCategory)
        .await?;

    Ok(())
}
