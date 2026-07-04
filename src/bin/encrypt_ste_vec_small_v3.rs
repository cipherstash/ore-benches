//! v3 sibling of `encrypt_ste_vec_small`: small JSON documents (4 flat
//! fields) into `json_ste_vec_small_encrypted_v3_*` as `eql_v3.json`
//! (SteVec document) payloads.
//!
//! SteVecMode::Standard is load-bearing: Standard emits `oc` (CLLW-ORE)
//! per orderable entry, which is one of the two per-entry terms
//! (`hm` XOR `oc`) the `from_v2` SteVec conversion accepts. Compat mode
//! emits `op`, which the v3 document entry shape does not carry.
//!
//! Environment variables: DATABASE_URL, NUM_RECORDS, BATCH_SIZE,
//! TABLE_SUFFIX, V3_CONVERT_ONLY, CS_* credentials.

use anyhow::Result;
use cipherstash_client::{
    eql::Identifier,
    schema::{
        column::{ArrayIndexMode, Index, IndexType, SteVecMode},
        ColumnConfig, ColumnType,
    },
};
use dbbenches::{v3::TargetDomain, FakeJsonSmall, IngestOptionsBuilder, WrappedJson};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dbbenches::init_tracing();

    let num_records: i32 = env::var("NUM_RECORDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("NUM_RECORDS must be a valid integer");

    let batch_size: usize = env::var("BATCH_SIZE")
        .unwrap_or_else(|_| "1000".to_string())
        .parse()
        .expect("BATCH_SIZE must be a valid integer");

    let table_suffix = env::var("TABLE_SUFFIX").unwrap_or_default();
    let table_name = format!("json_ste_vec_small_encrypted_v3{}", table_suffix);

    IngestOptionsBuilder::new("encrypt_ste_vec_small_v3")
        .num_records(num_records)
        .batch_size(batch_size)
        .identifier(Identifier::new(&table_name, "value"))
        .column_config(
            ColumnConfig::build("value")
                .casts_as(ColumnType::Json)
                .add_index(Index::new(IndexType::SteVec {
                    prefix: "value".to_string(),
                    term_filters: Default::default(),
                    array_index_mode: ArrayIndexMode::default(),
                    mode: SteVecMode::Standard,
                })),
        )
        .convert_to_v3(TargetDomain::parse("json").expect("json is a v3 domain"))
        .build()?
        .ingest::<WrappedJson, _>(FakeJsonSmall)
        .await?;

    Ok(())
}
