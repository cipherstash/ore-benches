//! EQL v3 twin of `encrypt_ste_vec_small`: encrypts small JSON documents
//! (first_name / last_name / age / email) with SteVec indexing via the
//! existing cipherstash-client (v2 wire) pipeline, converts each `k: "sv"`
//! payload with `eql_bindings::from_v2`, and inserts into
//! `json_ste_vec_small_encrypted_v3` (or a `TABLE_SUFFIX` variant).
//!
//! Target domain: `eql_v3.json` — the SteVec document domain
//! (`{v: 3, k: "sv", i, sv: [...]}`, per-entry `s` + `c` + exactly one of
//! `hm` XOR `oc`). Standard SteVec mode matches the v2 bin, so the encryption
//! workload is identical and the ingest numbers differ only by the from_v2
//! conversion. `sv` entry order is preserved by the converter — `sv[0]` is
//! the decryption root.
//!
//! Environment variables: DATABASE_URL, NUM_RECORDS (default 10000),
//! BATCH_SIZE (default 1000), TABLE_SUFFIX,
//! CS_CLIENT_ID / CS_CLIENT_KEY / CS_WORKSPACE_CRN.

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
                    // Standard mode emits `oc` (ORE CLLW) for orderable
                    // terms — the term the v3 jsonb entry contract (`hm`
                    // XOR `oc`) and the field_order scenarios expect.
                    mode: SteVecMode::Standard,
                })),
        )
        .convert_to_v3(TargetDomain::parse("json").expect("json is the v3 SteVec domain"))
        .build()?
        .ingest::<WrappedJson, _>(FakeJsonSmall)
        .await?;

    Ok(())
}
