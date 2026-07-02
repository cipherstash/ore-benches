//! EQL v3 twin of `encrypt_category`: encrypts low-cardinality categorical
//! values (`CAT_001`..`CAT_250`) via the existing cipherstash-client (v2
//! wire) pipeline, converts each storage payload with
//! `eql_bindings::from_v2`, and inserts into `category_encrypted_v3_*`.
//!
//! Target domain: `eql_v3.text_eq` (hm) — same unique-index configuration
//! as the v2 bin, so the encryption workload is identical and the ingest
//! numbers differ only by the from_v2 conversion. Drives the GROUP_BY_V3
//! scenarios in `benches/group_by_v3.rs`.
//!
//! Environment variables: DATABASE_URL, NUM_RECORDS (default 10000),
//! TABLE_SUFFIX, CS_CLIENT_ID / CS_CLIENT_KEY / CS_WORKSPACE_CRN.

use anyhow::Result;
use cipherstash_client::{
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::{v3::TargetDomain, FakeCategory, IngestOptionsBuilder};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
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
