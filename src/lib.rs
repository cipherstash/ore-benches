use anyhow::{Context, Result};
use cipherstash_client::{
    config::EnvSource,
    encryption::{Plaintext, ScopedCipher},
    eql::{encrypt_eql, EqlOperation, Identifier, PreparedPlaintext},
    schema::ColumnConfig,
    ZeroKMSConfig,
};
use fake::{Dummy, Fake};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, types::Json, QueryBuilder};
use std::borrow::Cow;
use std::env;
use std::fmt::Debug;
use std::sync::Arc;

pub struct IngestOptions {
    pub bench_name: String,
    pub num_records: i32,
    pub batch_size: usize,
    pub identifier: Identifier,
    pub column_config: ColumnConfig,
}

pub struct IngestOptionsBuilder {
    bench_name: String,
    num_records: Option<i32>,
    batch_size: Option<usize>,
    identifier: Option<Identifier>,
    column_config: Option<ColumnConfig>,
}

impl IngestOptionsBuilder {
    const DEFAULT_BATCH_SIZE: usize = 1000;
    const DEFAULT_NUM_RECORDS: i32 = 100_000;

    pub fn new(bench_name: impl Into<String>) -> Self {
        Self {
            bench_name: bench_name.into(),
            num_records: None,
            batch_size: None,
            identifier: None,
            column_config: None,
        }
    }

    pub fn num_records(mut self, num_records: i32) -> Self {
        self.num_records = Some(num_records);
        self
    }

    pub fn batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = Some(batch_size);
        self
    }

    pub fn identifier(mut self, identifier: Identifier) -> Self {
        self.identifier = Some(identifier);
        self
    }

    pub fn column_config(mut self, column_config: ColumnConfig) -> Self {
        self.column_config = Some(column_config);
        self
    }

    pub fn build(self) -> Result<IngestOptions> {
        Ok(IngestOptions {
            bench_name: self.bench_name,
            num_records: self.num_records.unwrap_or(Self::DEFAULT_NUM_RECORDS),
            batch_size: self.batch_size.unwrap_or(Self::DEFAULT_BATCH_SIZE),
            identifier: self.identifier.context("identifier is required")?,
            column_config: self.column_config.context("column_config is required")?,
        })
    }
}

impl IngestOptions {
    pub async fn ingest<T, F>(self, f: F) -> Result<()>
    where
        T: Into<Plaintext> + Dummy<F> + Send + Debug,
    {
        let database_url =
            env::var("DATABASE_URL").context("DATABASE_URL environment variable must be set")?;

        let num_records: i32 = env::var("NUM_RECORDS")
            .unwrap_or_else(|_| "10000".to_string())
            .parse()
            .expect("NUM_RECORDS must be a valid integer");

        let hf_iteration: i32 = env::var("HYPERFINE_ITERATION")
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .expect("HYPERFINE_ITERATION must be a valid integer");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        let client = ZeroKMSConfig::builder()
            .add_source(EnvSource::new())
            .build_with_client_key()
            .expect("failed to build config")
            .create_client();

        let scoped_cipher = ScopedCipher::init_default(Arc::new(client)).await?;
        let scoped_cipher = Arc::new(scoped_cipher);

        let column_config = Cow::Borrowed(&self.column_config);

        for batch_start in (0..self.num_records).step_by(self.batch_size) {
            let batch_end = (batch_start + self.batch_size as i32).min(self.num_records);
            let batch_count = batch_end - batch_start;

            let prepared = (0..batch_count)
                .map(|_| {
                    let x: T = f.fake();

                    PreparedPlaintext::new(
                        // FIXME: take a reference instead of using Cow?
                        column_config.clone(),
                        // FIXME: take a reference instead of owning the identifier
                        self.identifier.clone(),
                        Plaintext::new(x),
                        EqlOperation::Store,
                    )
                })
                .collect::<Vec<_>>();

            let out = encrypt_eql(scoped_cipher.clone(), prepared, &Default::default()).await?;

            QueryBuilder::new(format!("INSERT INTO {} (value) ", self.identifier.table()))
                .push_values(out.into_iter(), |mut b, v| {
                    b.push_bind(Json(v));
                })
                .build()
                .execute(&pool)
                .await?;

        }

        let result = json!({
            "inserted": num_records
        });
        let filename = format!("target/{}-{num_records}_{hf_iteration}.json", self.bench_name);
        std::fs::write(&filename, serde_json::to_string(&result)?)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct WrappedJson(pub serde_json::Value);

impl From<WrappedJson> for Plaintext {
    fn from(WrappedJson(value): WrappedJson) -> Self {
        Plaintext::JsonB(Some(value))
    }
}
