use anyhow::{Context, Result};
use cipherstash_client::{
    config::EnvSource,
    credentials::ServiceCredentials,
    encryption::{Plaintext, QueryOp, ScopedCipher},
    eql::{decrypt_eql, encrypt_eql, EqlCiphertext, EqlOperation, Identifier, PreparedPlaintext},
    schema::{column::IndexType, ColumnConfig},
    ZeroKMSConfig,
};
use fake::{Dummy, Fake};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, types::Json, QueryBuilder};
use std::borrow::Cow;
use std::env;
use std::fmt::Debug;
use std::sync::Arc;

pub async fn init_scoped_cipher() -> Result<Arc<ScopedCipher<ServiceCredentials>>> {
    let client = ZeroKMSConfig::builder()
        .add_source(EnvSource::new())
        .build_with_client_key()
        .context("failed to build config")?
        .create_client();

    let scoped_cipher = ScopedCipher::init_default(Arc::new(client)).await?;
    Ok(Arc::new(scoped_cipher))
}

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
        let filename = format!(
            "target/{}-{num_records}_{hf_iteration}.json",
            self.bench_name
        );
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

pub struct EncryptedQueryBuilder {
    pub column_config: ColumnConfig,
    pub identifier: Identifier,
    pub index_type: Option<IndexType>,
    pub statement: Option<String>,
}

impl EncryptedQueryBuilder {
    pub fn new(column_config: ColumnConfig, identifier: Identifier) -> Self {
        Self {
            column_config,
            identifier,
            index_type: None,
            statement: None,
        }
    }

    pub fn index_type(mut self, index_type: IndexType) -> Self {
        self.index_type = Some(index_type);
        self
    }

    pub fn statement(mut self, statement: impl Into<String>) -> Self {
        self.statement = Some(statement.into());
        self
    }

    pub async fn build_query<T>(
        self,
        plaintext: T,
        cipher: Arc<ScopedCipher<ServiceCredentials>>,
    ) -> Result<EncryptedQuery>
    where
        T: Into<Plaintext> + Send + Debug,
    {
        let index_type = self
            .index_type
            .context("index_type must be set to build query")?;

        let prepared = PreparedPlaintext::new(
            Cow::Owned(self.column_config),
            self.identifier.clone(),
            plaintext.into(),
            EqlOperation::Query(&index_type, QueryOp::Default),
        );

        let mut out = encrypt_eql(Arc::clone(&cipher), vec![prepared], &Default::default()).await?;

        Ok(EncryptedQuery {
            eql: out.remove(0),
            statement: self.statement.context("statement must be set")?,
            scoped_cipher: cipher,
        })
    }
}

pub struct EncryptedQuery {
    pub eql: EqlCiphertext,
    pub statement: String,
    scoped_cipher: Arc<ScopedCipher<ServiceCredentials>>,
}

impl EncryptedQuery {
    pub async fn execute(&self, pool: &sqlx::PgPool) -> Result<Vec<(i32, Json<EqlCiphertext>)>> {
        let results: Vec<(i32, Json<EqlCiphertext>)> = sqlx::query_as(&self.statement)
            .bind(Json(&self.eql))
            .fetch_all(pool)
            .await?;

        Ok(results)
    }

    pub async fn execute_and_decrypt<T>(&self, pool: &sqlx::PgPool) -> Result<Vec<T>>
    where
        T: TryFrom<Plaintext>,
        <T as TryFrom<Plaintext>>::Error: Debug,
    {
        let results: Vec<(i32, Json<EqlCiphertext>)> = self.execute(pool).await?;

        let decrypted = decrypt_eql(
            Arc::clone(&self.scoped_cipher),
            results.into_iter().map(|(_, value)| value.0),
            &Default::default(),
        )
        .await?
        .into_iter()
        .map(|pt| T::try_from(pt).expect("failed to convert plaintext"))
        .collect();

        Ok(decrypted)
    }
}
