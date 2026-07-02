//! EQL v3 twin of `encrypt_string`: encrypts generated names via the
//! existing cipherstash-client (v2 wire) pipeline, converts each storage
//! payload with `eql_bindings::from_v2`, and inserts into
//! `string_encrypted_v3` (or a `TABLE_SUFFIX` variant).
//!
//! Target domain: `eql_v3.text_search` (hm + ob + bf) — the only
//! single-column v3 domain that serves both the EXACT_V3 (hmac equality)
//! and MATCH_V3 (bloom containment) scenario families, mirroring how the
//! v2 `string_encrypted` table is shared by exact.rs and match.rs. To
//! satisfy `text_search`'s required `ob` term the column config adds an
//! ORE index that v2's `encrypt_string` does not have — this bin therefore
//! encrypts one extra term per value, and its ingest throughput is NOT
//! directly comparable to `encrypt_string`'s (use the dedicated
//! `convert_overhead` family to quantify pure conversion cost).
//!
//! Environment variables: DATABASE_URL, NUM_RECORDS (default 10000),
//! TABLE_SUFFIX, CS_CLIENT_ID / CS_CLIENT_KEY / CS_WORKSPACE_CRN.

use anyhow::Result;
use cipherstash_client::{
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::{v3::TargetDomain, IngestOptionsBuilder};
use fake::{faker::name::raw::Name, locales::EN};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let num_records: i32 = env::var("NUM_RECORDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("NUM_RECORDS must be a valid integer");

    let table_suffix = env::var("TABLE_SUFFIX").unwrap_or_default();
    let table_name = format!("string_encrypted_v3{}", table_suffix);

    IngestOptionsBuilder::new("encrypt_string_v3")
        .num_records(num_records)
        .batch_size(1000)
        .identifier(Identifier::new(&table_name, "value"))
        .column_config(
            ColumnConfig::build("value")
                .casts_as(ColumnType::Text)
                .add_index(Index::new_unique())
                .add_index(Index::new_match())
                // text_search requires `ob` — see the module docs above.
                .add_index(Index::new_ore()),
        )
        .convert_to_v3(TargetDomain::parse("text_search").expect("text_search is a v3 domain"))
        .build()?
        .ingest::<String, Name<EN>>(Name(EN))
        .await?;

    Ok(())
}
