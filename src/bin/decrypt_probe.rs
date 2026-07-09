//! Decrypt probe: sample and decrypt one row from an encrypted bench table.
//!
//! Diagnoses credential/workspace/keyset issues without running a full
//! bench — e.g. "which workspace was this table encrypted under?" (run with
//! CS_WORKSPACE_ID=<id> per candidate) or "has the scoped-cipher TTL
//! behaviour changed?". Works against both v2 (`eql_v2_encrypted`) and v3
//! (`eql_v3.*` domain) tables — pass `--v3` for v3.
//!
//! Usage:
//!   decrypt_probe <table> [--v3]
//!
//! Prints the decrypted sample on success; exits non-zero on failure.

use anyhow::{Context, Result};
use dbbenches::{init_scoped_cipher, sample_plaintext_string, v3::sample_plaintext_string_v3};
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dbbenches::init_tracing();

    let args: Vec<String> = env::args().collect();
    let table = args
        .get(1)
        .context("usage: decrypt_probe <table> [--v3]")?
        .clone();
    let is_v3 = args.iter().any(|a| a == "--v3");

    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL environment variable must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    let cipher = init_scoped_cipher().await?;

    let sample = if is_v3 {
        sample_plaintext_string_v3(&pool, cipher, &table).await?
    } else {
        sample_plaintext_string(&pool, cipher, &table).await?
    };

    println!("decrypt OK: {} -> {:?}", table, sample);
    Ok(())
}
