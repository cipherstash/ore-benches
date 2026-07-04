//! Random i32s into `integer_encrypted_ope_v3_*` as `eql_v3.integer_ord_ope`
//! payloads — the v3 CLLW-OPE fast ordering path (native bytea btree).
//!
//! cipherstash-client 0.38.1 emits the scalar OPE-CLLW `op` term for
//! columns configured with `Index::new_ope()` (CIP-3280/CIP-3348), so this
//! is REAL OPE ciphertext end-to-end — the earlier synthetic-op workaround
//! is retired, and ciphertext sizes / client-side term-generation cost are
//! now representative.
//!
//! Environment variables: DATABASE_URL, NUM_RECORDS, TABLE_SUFFIX,
//! V3_CONVERT_ONLY, CS_CLIENT_ID / CS_CLIENT_KEY / CS_WORKSPACE_CRN.

use anyhow::Result;
use cipherstash_client::{
    eql::Identifier,
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::IngestOptionsBuilder;
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
    let table_name = format!("integer_encrypted_ope_v3{}", table_suffix);

    IngestOptionsBuilder::new("encrypt_int_ope_v3")
        .num_records(num_records)
        .batch_size(1000)
        .identifier(Identifier::new(&table_name, "value"))
        .column_config(
            ColumnConfig::build("value")
                .casts_as(ColumnType::Int)
                .add_index(Index::new_ope()),
        )
        .build()?
        .ingest_v3::<i32, _>(Faker, "integer_ord_ope")
        .await?;

    Ok(())
}
