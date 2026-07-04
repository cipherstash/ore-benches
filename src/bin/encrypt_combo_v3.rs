//! v3 sibling of `encrypt_combo`: three-column rows (`name`, `age`,
//! `category`) into `combo_encrypted_v3_*`, with per-column v3 targets:
//!
//!   * `name`     → eql_v3.text_search (unique+match+ore config; see
//!                  encrypt_string_v3 for why ORE is added)
//!   * `age`      → eql_v3.integer_ord
//!   * `category` → eql_v3.text_eq
//!
//! Environment variables: DATABASE_URL, NUM_RECORDS, TABLE_SUFFIX,
//! CS_CLIENT_ID / CS_CLIENT_KEY / CS_WORKSPACE_CRN.

use anyhow::{Context, Result};
use cipherstash_client::{
    encryption::Plaintext,
    eql::{encrypt_eql, EqlCiphertext, EqlOperation, EqlOutput, Identifier, PreparedPlaintext},
    schema::{column::Index, ColumnConfig, ColumnType},
};
use dbbenches::{init_scoped_cipher, v3::to_v3_stored, FakeCategory};
use fake::{faker::name::raw::Name, locales::EN, Fake};
use sqlx::{postgres::PgPoolOptions, types::Json, QueryBuilder};
use std::borrow::Cow;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dbbenches::init_tracing();

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

    let scoped_cipher = init_scoped_cipher().await?;

    let name_config = ColumnConfig::build("name")
        .casts_as(ColumnType::Text)
        .add_index(Index::new_unique())
        .add_index(Index::new_match())
        .add_index(Index::new_ore());
    let age_config = ColumnConfig::build("age")
        .casts_as(ColumnType::Int)
        .add_index(Index::new_ore());
    let category_config = ColumnConfig::build("category")
        .casts_as(ColumnType::Text)
        .add_index(Index::new_unique());

    let name_ident = Identifier::new(&table_name, "name");
    let age_ident = Identifier::new(&table_name, "age");
    let category_ident = Identifier::new(&table_name, "category");

    for batch_start in (0..num_records).step_by(batch_size) {
        let batch_end = (batch_start + batch_size as i32).min(num_records);
        let batch_count = batch_end - batch_start;

        let mut prepared = Vec::with_capacity((batch_count * 3) as usize);
        for _ in 0..batch_count {
            let name: String = Name(EN).fake();
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
        // (name, age, category) tuples, each converted to its column's v3
        // domain payload.
        let rows = ciphertexts
            .chunks_exact(3)
            .map(|c| {
                Ok((
                    to_v3_stored(&c[0], "text_search")?,
                    to_v3_stored(&c[1], "integer_ord")?,
                    to_v3_stored(&c[2], "text_eq")?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        QueryBuilder::new(format!(
            "INSERT INTO {} (name, age, category) ",
            table_name
        ))
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
