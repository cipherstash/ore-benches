//! v3 sibling of `encrypt_string`: random English names into
//! `string_encrypted_v3_*` as `eql_v3.text_search` payloads.
//!
//! The config adds `Index::new_ore()` on top of v2's unique+match: v3 has no
//! hm+bf domain — `text_search` (the only equality+match domain) also
//! requires the ORE term `ob`, and `from_v2` fails closed on a missing term.
//! v3 string rows are therefore wider than their v2 siblings; the comparison
//! report flags this when reading ingest deltas.
//!
//! Environment variables: DATABASE_URL, NUM_RECORDS, TABLE_SUFFIX,
//! V3_CONVERT_ONLY (encrypt+convert without inserting — used to decompose
//! conversion overhead), CS_CLIENT_ID / CS_CLIENT_KEY / CS_WORKSPACE_CRN.

use anyhow::Result;
use cipherstash_client::{
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::IngestOptionsBuilder;
use fake::{faker::name::raw::Name, locales::EN};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dbbenches::init_tracing();

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
                .add_index(Index::new_ore()),
        )
        .build()?
        .ingest_v3::<String, Name<EN>>(Name(EN), "text_search")
        .await?;

    Ok(())
}
