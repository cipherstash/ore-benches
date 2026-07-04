//! EQL v3 harness plumbing.
//!
//! The pinned cipherstash-client (0.34.1-alpha.9) emits EQL **v2.3** wire
//! payloads. EQL v3 replaced the single `eql_v2_encrypted` composite with
//! per-capability jsonb DOMAIN types (`eql_v3.text_search`,
//! `eql_v3.integer_ord`, …), and ships a supported conversion path in the
//! `eql-bindings` crate (`from_v2`) that names these benches as a consumer.
//! Everything v3-specific in the bench harness funnels through this module:
//!
//!   * stored payloads: client v2 encrypt → [`to_v3_stored`] → bind as jsonb
//!     (every v3 domain CHECK accepts its own converted payload — the
//!     converter strict-parses through the domain's binding struct before
//!     returning).
//!   * query parameters: **also stored-shape**. `from_v2_query` deliberately
//!     supports only the SteVec containment needle ([`to_v3_query_json`]);
//!     no v3 scalar *query* wire shape exists because every scalar domain
//!     CHECK requires the ciphertext `c` that v2 query payloads omit. So
//!     scalar needles are encrypted with `EqlOperation::Store` and converted
//!     whole ([`encrypt_stored_v3`]). The v3 operator surface is built for
//!     this: each domain wires `= / < / …` overloads with `RIGHTARG = jsonb`
//!     that cast the bound parameter to the domain.
//!   * decryption: v3 rows carry the same record ciphertext `c` the v2
//!     client wrote — [`v3_row_to_v2_ciphertext`] rebuilds a minimal v2
//!     envelope so the pinned client's `decrypt_eql` can decrypt v3 rows
//!     (needle sampling, `_decrypt` scenarios).

use crate::{extract_indexes_used, EqlV2Encrypted, IngestOptions};
use anyhow::{Context, Result};
use cipherstash_client::{
    encryption::{Plaintext, ScopedCipher},
    eql::{decrypt_eql, encrypt_eql, EqlCiphertext, EqlOperation, EqlOutput, PreparedPlaintext},
    AutoStrategy,
};
use eql_bindings::from_v2::{from_v2, from_v2_query, TargetDomain};
use fake::{Dummy, Fake};
use serde_json::{json, Value};
use sqlx::postgres::{PgTypeInfo, PgTypeKind, PgValueRef};
use sqlx::{postgres::PgPoolOptions, types::Json, QueryBuilder};
use std::borrow::Cow;
use std::env;
use std::fmt::Debug;
use std::sync::Arc;

/// Parse an unqualified v3 domain name (`"text_search"`, `"integer_ord"`,
/// `"json"`, …) with an anyhow-friendly error.
fn parse_target(target: &str) -> Result<TargetDomain> {
    TargetDomain::parse(target)
        .with_context(|| format!("unknown v3 target domain `{}`", target))
}

/// Convert a STORED v2.3 ciphertext into the v3 payload for `target`.
///
/// The returned `Value` has already passed `eql-bindings`' strict parse
/// through the target domain's binding struct, so the corresponding domain
/// CHECK is guaranteed to accept it at INSERT/cast time.
pub fn to_v3_stored(v2: &EqlCiphertext, target: &str) -> Result<Value> {
    let v2_value = serde_json::to_value(v2).context("serialize v2 ciphertext")?;
    let parsed = parse_target(target)?;
    from_v2(&v2_value, parsed)
        .with_context(|| format!("from_v2 conversion to `{}` failed", target))
}

/// Convert a v2 SteVec QUERY payload (containment needle) into the v3
/// `eql_v3.jsonb_query` wire shape.
pub fn to_v3_query_json(v2_query: &Value) -> Result<Value> {
    from_v2_query(v2_query, TargetDomain::Json)
        .context("from_v2_query conversion to `json` failed")
}

/// Rebuild a minimal v2 envelope from a v3 stored payload so the pinned
/// client's `decrypt_eql` can decrypt it.
///
/// Scalars carry the record ciphertext at `c`; SteVec documents carry it on
/// the FIRST `sv` entry (`sv[0].c` — the root-selector entry, mirroring
/// upstream `SteVec::into_root_ciphertext`; conversion preserves entry order
/// verbatim so this invariant survives the v2→v3 round trip).
pub fn v3_row_to_v2_ciphertext(v3: &Value) -> Result<EqlCiphertext> {
    let obj = v3
        .as_object()
        .context("v3 payload is not a JSON object")?;
    let c = match obj.get("c") {
        Some(c) => c.clone(),
        None => obj
            .get("sv")
            .and_then(Value::as_array)
            .and_then(|sv| sv.first())
            .and_then(|entry| entry.get("c"))
            .cloned()
            .context("v3 payload has neither `c` nor `sv[0].c`")?,
    };
    let i = obj.get("i").cloned().unwrap_or_else(|| json!({}));
    let v2 = json!({ "v": 2, "k": "ct", "i": i, "c": c });
    serde_json::from_value(v2).context("rebuilt v2 envelope did not parse as EqlCiphertext")
}

/// Encrypt one plaintext with `EqlOperation::Store` and convert to the v3
/// stored payload for `target`. This is how ALL v3 scalar query needles are
/// produced (see module docs — no scalar query wire shape exists).
pub async fn encrypt_stored_v3(
    cipher: Arc<ScopedCipher<AutoStrategy>>,
    column_config: &cipherstash_client::schema::ColumnConfig,
    identifier: &cipherstash_client::eql::Identifier,
    plaintext: impl Into<Plaintext>,
    target: &str,
) -> Result<Value> {
    let prepared = PreparedPlaintext::new(
        Cow::Owned(column_config.clone()),
        identifier.clone(),
        plaintext.into(),
        EqlOperation::Store,
    );
    let mut out = encrypt_eql(cipher, vec![prepared], &Default::default()).await?;
    let EqlOutput::Store(ciphertext) = out.remove(0) else {
        unreachable!("EqlOperation::Store yields EqlOutput::Store");
    };
    to_v3_stored(&ciphertext, target)
}

/// Decoded `SELECT value` cell from an `eql_v3.*` domain column.
///
/// v3 domains are DOMAINs over jsonb, so the wire representation is plain
/// jsonb — only the reported type OID differs (Postgres reports the domain's
/// OID in RowDescription). `compatible` therefore accepts jsonb itself plus
/// any domain whose base type is jsonb-compatible.
///
/// The v2 harness needed `EqlV2Encrypted` to avoid a `value::jsonb` cast
/// being folded into ORDER BY sort keys; v3 has no bare `ORDER BY value`
/// scenario (ordering always goes through an extractor function), but
/// decoding the domain directly keeps the SELECT list identical to v2's.
#[derive(Debug)]
pub struct EqlV3Encrypted(pub Json<Value>);

impl EqlV3Encrypted {
    pub fn into_value(self) -> Value {
        self.0 .0
    }
}

impl sqlx::Type<sqlx::Postgres> for EqlV3Encrypted {
    fn type_info() -> PgTypeInfo {
        <Json<Value> as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        if <Json<Value> as sqlx::Type<sqlx::Postgres>>::compatible(ty) {
            return true;
        }
        matches!(
            ty.kind(),
            PgTypeKind::Domain(base)
                if <Json<Value> as sqlx::Type<sqlx::Postgres>>::compatible(base)
        )
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for EqlV3Encrypted {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let json = <Json<Value> as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
        Ok(EqlV3Encrypted(json))
    }
}

/// Sample a plaintext string from a v3 encrypted table by decrypting the
/// first row — the v3 sibling of [`crate::sample_plaintext_string`].
pub async fn sample_plaintext_string_v3(
    pool: &sqlx::PgPool,
    cipher: Arc<ScopedCipher<AutoStrategy>>,
    table_name: &str,
) -> Result<String> {
    let row: (EqlV3Encrypted,) =
        sqlx::query_as(&format!("SELECT value FROM {} LIMIT 1", table_name))
            .fetch_one(pool)
            .await
            .with_context(|| format!("sample query failed against {}", table_name))?;

    let ciphertext = v3_row_to_v2_ciphertext(&row.0.into_value())?;
    let decrypted = decrypt_eql(cipher, vec![ciphertext], &Default::default())
        .await
        .context("sample decrypt failed")?;

    let pt = decrypted
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("decrypt_eql returned empty Vec for {}", table_name))?;
    match &pt {
        Plaintext::Text(Some(s)) => Ok(s.clone()),
        other => anyhow::bail!("expected Text sample from {}, got {:?}", table_name, other),
    }
}

/// A bench query against an `eql_v3.*` column: stored-shape jsonb parameter
/// bound at `$1`, mirroring [`crate::EncryptedQuery`]'s surface so the v3
/// bench files read like their v2 siblings.
pub struct V3EncryptedQuery {
    pub param: Value,
    pub statement: String,
    scoped_cipher: Arc<ScopedCipher<AutoStrategy>>,
}

impl V3EncryptedQuery {
    pub fn new(
        param: Value,
        statement: impl Into<String>,
        scoped_cipher: Arc<ScopedCipher<AutoStrategy>>,
    ) -> Self {
        Self {
            param,
            statement: statement.into(),
            scoped_cipher,
        }
    }

    pub async fn execute(&self, pool: &sqlx::PgPool) -> Result<Vec<(i32, EqlV3Encrypted)>> {
        let results: Vec<(i32, EqlV3Encrypted)> = sqlx::query_as(&self.statement)
            .bind(Json(&self.param))
            .fetch_all(pool)
            .await?;
        Ok(results)
    }

    pub async fn execute_and_decrypt<T>(&self, pool: &sqlx::PgPool) -> Result<Vec<T>>
    where
        T: TryFrom<Plaintext>,
        <T as TryFrom<Plaintext>>::Error: Debug,
    {
        let results = self.execute(pool).await?;
        let ciphertexts = results
            .into_iter()
            .map(|(_, value)| v3_row_to_v2_ciphertext(&value.into_value()))
            .collect::<Result<Vec<_>>>()?;

        let decrypted = decrypt_eql(
            Arc::clone(&self.scoped_cipher),
            ciphertexts,
            &Default::default(),
        )
        .await?
        .into_iter()
        .map(|pt| T::try_from(pt).expect("failed to convert plaintext"))
        .collect();

        Ok(decrypted)
    }

    /// `EXPLAIN (FORMAT JSON)` with the parameter bound — same startup
    /// metadata capture as the v2 harness.
    pub async fn explain(&self, pool: &sqlx::PgPool) -> Result<Value> {
        let explain_sql = format!("EXPLAIN (FORMAT JSON) {}", self.statement);
        let row: (Json<Value>,) = sqlx::query_as(&explain_sql)
            .bind(Json(&self.param))
            .fetch_one(pool)
            .await?;
        Ok(row.0 .0)
    }

    pub fn parameter_json(&self) -> Result<Value> {
        Ok(self.param.clone())
    }

    /// Convenience for the startup metadata pass: EXPLAIN + one real
    /// execution, returning (explain, indexes_used, rows_returned).
    pub async fn capture_metadata(
        &self,
        pool: &sqlx::PgPool,
    ) -> Result<(Value, Vec<String>, u64)> {
        let explain = self.explain(pool).await?;
        let indexes_used = extract_indexes_used(&explain);
        let rows = self.execute(pool).await?.len() as u64;
        Ok((explain, indexes_used, rows))
    }
}

impl IngestOptions {
    /// The v3 sibling of [`IngestOptions::ingest`]: identical batch loop,
    /// with each stored v2 ciphertext converted to the `target` v3 domain
    /// payload before binding. The conversion is deliberately inside the
    /// measured path — v3 ingest cost for the intended consumer IS
    /// "encrypt + convert + insert"; the report decomposes the client-side
    /// share via `V3_CONVERT_ONLY`.
    ///
    /// `V3_CONVERT_ONLY=1` skips the INSERT (encrypt + convert only) so the
    /// conversion overhead is attributable when comparing against the v2
    /// ingest numbers.
    pub async fn ingest_v3<T, F>(self, f: F, target: &str) -> Result<()>
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

        let convert_only = env::var("V3_CONVERT_ONLY").is_ok_and(|v| v == "1");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        let scoped_cipher = crate::init_scoped_cipher().await?;
        let column_config = Cow::Borrowed(&self.column_config);

        for batch_start in (0..self.num_records).step_by(self.batch_size) {
            let batch_end = (batch_start + self.batch_size as i32).min(self.num_records);
            let batch_count = batch_end - batch_start;

            let prepared = (0..batch_count)
                .map(|_| {
                    let x: T = f.fake();

                    PreparedPlaintext::new(
                        column_config.clone(),
                        self.identifier.clone(),
                        Plaintext::new(x),
                        EqlOperation::Store,
                    )
                })
                .collect::<Vec<_>>();

            let out = encrypt_eql(scoped_cipher.clone(), prepared, &Default::default()).await?;

            let converted = out
                .into_iter()
                .map(|v| {
                    let EqlOutput::Store(ciphertext) = v else {
                        unreachable!("storage batch must yield EqlOutput::Store");
                    };
                    to_v3_stored(&ciphertext, target)
                })
                .collect::<Result<Vec<_>>>()?;

            if convert_only {
                continue;
            }

            QueryBuilder::new(format!("INSERT INTO {} (value) ", self.identifier.table()))
                .push_values(converted.into_iter(), |mut b, v| {
                    b.push_bind(Json(v));
                })
                .build()
                .execute(&pool)
                .await?;
        }

        let result = json!({
            "inserted": num_records,
            "convert_only": convert_only,
        });
        let filename = format!(
            "target/{}-{num_records}_{hf_iteration}.json",
            self.bench_name
        );
        std::fs::write(&filename, serde_json::to_string(&result)?)?;

        Ok(())
    }
}

/// Convert a stored v2 row (as decoded by [`EqlV2Encrypted`]) to the v3
/// payload — used by cross-version correctness checks that read a v2 table
/// and probe the v3 table with the same logical value.
pub fn v2_row_to_v3_stored(row: EqlV2Encrypted, target: &str) -> Result<Value> {
    to_v3_stored(&row.into_ciphertext(), target)
}
