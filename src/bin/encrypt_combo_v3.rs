//! EQL v3 twin of `encrypt_combo`: encrypts three-column combo rows
//! (`name`, `age`, `category`) via the existing cipherstash-client (v2
//! wire) pipeline, converts each storage payload with
//! `eql_bindings::from_v2` for its per-column target domain, and inserts
//! into the `combo_encrypted_v3_*` tables. Used by `benches/combo_v3.rs`.
//!
//! Per-column target domains (matching the scenario capabilities):
//!   * `name`     → `eql_v3.text_match`   (bf — bloom containment; v3 has
//!     no LIKE, so the v2 unique+match config's `hm` term is dropped by
//!     the conversion)
//!   * `age`      → `eql_v3.integer_ord_ore` (ob — ORE ordering)
//!   * `category` → `eql_v3.text_eq`      (hm — hmac equality / GROUP BY)
//!
//! Environment variables: DATABASE_URL, NUM_RECORDS (default 10000),
//! TABLE_SUFFIX, CS_CLIENT_ID / CS_CLIENT_KEY / CS_WORKSPACE_CRN.

use anyhow::{Context, Result};
use cipherstash_client::{
    encryption::Plaintext,
    eql::{encrypt_eql, EqlCiphertext, EqlOperation, EqlOutput, Identifier, PreparedPlaintext},
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::{
    init_scoped_cipher, init_tracing,
    v3::{ciphertext_to_v3, TargetDomain},
    FakeCategory,
};
use fake::{faker::name::raw::Name, locales::EN, Fake};
use sqlx::{postgres::PgPoolOptions, types::Json, QueryBuilder};
use std::borrow::Cow;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL environment variable must be set")?;
    let num_records: i32 = env::var("NUM_RECORDS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("NUM_RECORDS must be a valid integer");
    let table_suffix = env::var("TABLE_SUFFIX").unwrap_or_default();
    let table_name = format!("combo_encrypted_v3{}", table_suffix);
    let batch_size: usize = 1000;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Init once and reuse for the binary lifetime — see lib.rs::ingest
    // for the rationale.
    let scoped_cipher = init_scoped_cipher().await?;

    // Same column configs as the v2 encrypt_combo bin — the conversion
    // step (not the encryption step) narrows each payload to its target
    // domain's terms, keeping the encryption workload identical to v2.
    let name_config = ColumnConfig::build("name")
        .casts_as(ColumnType::Text)
        .add_index(Index::new_unique())
        .add_index(Index::new_match());
    let age_config = ColumnConfig::build("age")
        .casts_as(ColumnType::Int)
        .add_index(Index::new_ore());
    let category_config = ColumnConfig::build("category")
        .casts_as(ColumnType::Text)
        .add_index(Index::new_unique());

    let name_ident = Identifier::new(&table_name, "name");
    let age_ident = Identifier::new(&table_name, "age");
    let category_ident = Identifier::new(&table_name, "category");

    let name_target = TargetDomain::parse("text_match").expect("text_match is a v3 domain");
    let age_target =
        TargetDomain::parse("integer_ord_ore").expect("integer_ord_ore is a v3 domain");
    let category_target = TargetDomain::parse("text_eq").expect("text_eq is a v3 domain");

    for batch_start in (0..num_records).step_by(batch_size) {
        let batch_end = (batch_start + batch_size as i32).min(num_records);
        let batch_count = batch_end - batch_start;

        let mut prepared = Vec::with_capacity((batch_count * 3) as usize);
        for _ in 0..batch_count {
            let name: String = Name(EN).fake();
            // Same distribution as the v2 bin — uniform 18..=90.
            let age: i32 = (18..=90).fake();
            let category: String = FakeCategory.fake();

            prepared.push(PreparedPlaintext::new(
                Cow::Borrowed(&name_config),
                name_ident.clone(),
                Plaintext::new(name),
                EqlOperation::Store,
            ));
            prepared.push(PreparedPlaintext::new(
                Cow::Borrowed(&age_config),
                age_ident.clone(),
                Plaintext::new(age),
                EqlOperation::Store,
            ));
            prepared.push(PreparedPlaintext::new(
                Cow::Borrowed(&category_config),
                category_ident.clone(),
                Plaintext::new(category),
                EqlOperation::Store,
            ));
        }

        let out = encrypt_eql(scoped_cipher.clone(), prepared, &Default::default()).await?;

        let ciphertexts: Vec<EqlCiphertext> = out
            .into_iter()
            .map(|o| match o {
                EqlOutput::Store(ct) => ct,
                EqlOutput::Query(_) => {
                    unreachable!("storage batch must yield EqlOutput::Store")
                }
            })
            .collect();

        // encrypt_eql preserves input order; chunks of 3 reassemble per-row
        // (name, age, category) tuples, converted per-column to v3.
        let rows = ciphertexts
            .chunks_exact(3)
            .map(|c| {
                Ok((
                    ciphertext_to_v3(&c[0], name_target).context("name payload")?,
                    ciphertext_to_v3(&c[1], age_target).context("age payload")?,
                    ciphertext_to_v3(&c[2], category_target).context("category payload")?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        QueryBuilder::new(format!("INSERT INTO {} (name, age, category) ", table_name))
            .push_values(rows, |mut b, (name, age, category)| {
                b.push_bind(Json(name));
                b.push_bind(Json(age));
                b.push_bind(Json(category));
            })
            .build()
            .execute(&pool)
            .await?;
    }

    Ok(())
}
