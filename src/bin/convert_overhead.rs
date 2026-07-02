//! Conversion-overhead ingest scenario: quantifies what
//! `eql_bindings::from_v2` adds on top of encryption.
//!
//! Runs the SAME workload in two modes selected by `CONVERT_MODE`:
//!
//!   * `encrypt_only`    — generate fake names and encrypt them as storage
//!     payloads (the shared cost floor).
//!   * `encrypt_convert` — the same, plus a v2→v3 `from_v2` conversion of
//!     every payload (target `eql_v3.text_search`, matching the
//!     encrypt_string_v3 ingest path).
//!
//! The column config matches `encrypt_string_v3` (unique + match + ore →
//! hm + bf + ob) so the conversion input is exactly what the real v3
//! string ingest converts. No database writes in either mode — the delta
//! between the two hyperfine families is pure client-side conversion cost
//! (JSON re-shaping + strict validation), free of INSERT noise.
//!
//! Reported as its own ingest family: `convert_overhead_encrypt_only` /
//! `convert_overhead_encrypt_convert` (see the mise task
//! `bench:ingest:convert-overhead`).
//!
//! Environment variables:
//! - CONVERT_MODE: `encrypt_only` | `encrypt_convert` (required)
//! - NUM_RECORDS: number of values to process (default: 10000)
//! - HYPERFINE_ITERATION: set by hyperfine; keys the validation sidecar
//! - CS_CLIENT_ID / CS_CLIENT_KEY / CS_WORKSPACE_CRN: CipherStash creds

use anyhow::{bail, Context, Result};
use cipherstash_client::{
    encryption::Plaintext,
    eql::{encrypt_eql, EqlOperation, EqlOutput, Identifier, PreparedPlaintext},
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::{
    init_scoped_cipher, init_tracing,
    v3::{ciphertext_to_v3, TargetDomain},
};
use fake::{faker::name::raw::Name, locales::EN, Fake};
use serde_json::json;
use std::borrow::Cow;
use std::env;
use std::hint::black_box;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let mode = env::var("CONVERT_MODE").context("CONVERT_MODE must be set")?;
    let convert = match mode.as_str() {
        "encrypt_only" => false,
        "encrypt_convert" => true,
        other => bail!("CONVERT_MODE must be `encrypt_only` or `encrypt_convert`, got `{other}`"),
    };

    let num_records: i32 = env::var("NUM_RECORDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("NUM_RECORDS must be a valid integer");

    let hf_iteration: i32 = env::var("HYPERFINE_ITERATION")
        .unwrap_or_else(|_| "0".to_string())
        .parse()
        .expect("HYPERFINE_ITERATION must be a valid integer");

    let batch_size: usize = 1000;

    let scoped_cipher = init_scoped_cipher().await?;

    // Match encrypt_string_v3's config exactly — the conversion input must
    // be the hm+bf+ob payload the real v3 string ingest converts.
    let column_config = ColumnConfig::build("value")
        .casts_as(ColumnType::Text)
        .add_index(Index::new_unique())
        .add_index(Index::new_match())
        .add_index(Index::new_ore());
    let identifier = Identifier::new("string_encrypted_v3", "value");
    let target = TargetDomain::parse("text_search").expect("text_search is a v3 domain");

    let mut processed: i64 = 0;
    for batch_start in (0..num_records).step_by(batch_size) {
        let batch_end = (batch_start + batch_size as i32).min(num_records);
        let batch_count = batch_end - batch_start;

        let prepared = (0..batch_count)
            .map(|_| {
                let name: String = Name(EN).fake();
                PreparedPlaintext::new(
                    Cow::Borrowed(&column_config),
                    identifier.clone(),
                    Plaintext::new(name),
                    EqlOperation::Store,
                )
            })
            .collect::<Vec<_>>();

        let out = encrypt_eql(scoped_cipher.clone(), prepared, &Default::default()).await?;

        for output in out {
            let EqlOutput::Store(ciphertext) = output else {
                unreachable!("storage batch must yield EqlOutput::Store");
            };
            if convert {
                // black_box keeps the release optimiser from eliding the
                // conversion whose cost this scenario exists to measure.
                black_box(ciphertext_to_v3(&ciphertext, target)?);
            } else {
                black_box(&ciphertext);
            }
            processed += 1;
        }
    }

    // Validation sidecar consumed by combine_benchmark — same contract as
    // IngestOptions::ingest (`inserted` here means "values processed";
    // this bench deliberately never touches the database).
    let result = json!({ "inserted": processed });
    let filename = format!("target/convert_overhead_{mode}-{num_records}_{hf_iteration}.json");
    std::fs::write(&filename, serde_json::to_string(&result)?)?;

    Ok(())
}
