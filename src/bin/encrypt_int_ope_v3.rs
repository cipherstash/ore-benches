//! Random i32s into `integer_encrypted_ope_v3_*` as `eql_v3.integer_ord_ope`
//! payloads — the v3 CLLW-OPE fast ordering path (native bytea btree).
//!
//! The pinned client does not emit the `op` term (CIP-3280/CIP-3348), so the
//! term is SYNTHESIZED: a fixed-width big-endian hex encoding of the
//! sign-flipped plaintext (see `dbbenches::v3::to_v3_stored_with_synth_ope`).
//! Server-side behaviour (index build, comparisons, plan shapes) is
//! measured faithfully; two caveats for the report:
//!
//!   * ciphertext SIZE is approximate until a client emits real CLLW-OPE
//!     (width tunable via SYNTH_OPE_HEX_WIDTH, default 32 hex chars);
//!   * client-side ingest cost EXCLUDES real OPE term generation — the
//!     column config carries no v2 index, so this binary's throughput is
//!     "encrypt + synthesize + convert + insert", not a like-for-like
//!     client comparison against `encrypt_int_v3`.
//!
//! Environment variables: DATABASE_URL, NUM_RECORDS, TABLE_SUFFIX,
//! SYNTH_OPE_HEX_WIDTH, V3_CONVERT_ONLY, CS_* credentials.

use anyhow::Result;
use cipherstash_client::{
    eql::Identifier,
    schema::{ColumnConfig, ColumnType},
};
use dbbenches::{v3::i32_order_key, IngestOptionsBuilder};
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
        // No v2 index config: integer_ord_ope requires only the `op` term,
        // which is injected synthetically per record below.
        .column_config(ColumnConfig::build("value").casts_as(ColumnType::Int))
        .build()?
        .ingest_v3_synth_ope::<i32, _, _>(Faker, "integer_ord_ope", |v: &i32| i32_order_key(*v))
        .await?;

    Ok(())
}
