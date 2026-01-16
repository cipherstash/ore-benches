use anyhow::{Context, Result};
use cipherstash_client::{
    config::EnvSource,
    encryption::{Plaintext, ScopedCipher},
    eql::{encrypt_eql, EqlOperation, Identifier, PreparedPlaintext},
    schema::ColumnConfig,
    ZeroKMSConfig,
};
use fake::{Dummy, Fake};
use sqlx::{postgres::PgPoolOptions, types::Json, QueryBuilder};
use std::borrow::Cow;
use std::env;
use std::sync::Arc;

pub struct IngestOptions {
    pub num_records: i32,
    pub batch_size: usize,
    pub identifier: Identifier,
    pub column_config: ColumnConfig,
}

pub struct IngestOptionsBuilder {
    num_records: Option<i32>,
    batch_size: Option<usize>,
    identifier: Option<Identifier>,
    column_config: Option<ColumnConfig>,
}

impl IngestOptionsBuilder {
    const DEFAULT_BATCH_SIZE: usize = 1000;
    const DEFAULT_NUM_RECORDS: i32 = 100_000;

    pub fn new() -> Self {
        Self {
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
            num_records: self.num_records.unwrap_or(
                Self::DEFAULT_NUM_RECORDS,
            ),
            batch_size: self.batch_size.unwrap_or(
                Self::DEFAULT_BATCH_SIZE,
            ),
            identifier: self.identifier.context("identifier is required")?,
            column_config: self.column_config.context("column_config is required")?,
        })
    }
}

impl IngestOptions {
    pub async fn ingest<T, F>(self, f: F) -> Result<()> where T: Into<Plaintext> + Dummy<F> + Send {
        let database_url =
            env::var("DATABASE_URL").context("DATABASE_URL environment variable must be set")?;

        let num_records: i32 = env::var("NUM_RECORDS")
            .unwrap_or_else(|_| "10000".to_string())
            .parse()
            .expect("NUM_RECORDS must be a valid integer");

        println!("Connecting to database...");
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

        println!("Encrypting and inserting {} values...", self.num_records);
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

            println!("Inserted {} / {} records", batch_end, num_records);
        }

        Ok(())
    }
}