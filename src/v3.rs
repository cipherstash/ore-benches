//! EQL v3 harness plumbing.
//!
//! The pinned cipherstash-client (0.38.1) emits EQL **v2.3** wire
//! payloads. EQL v3 replaced the single `eql_v2_encrypted` composite with
//! per-capability jsonb DOMAIN types (`public.text_search`,
//! `public.integer_ord`, …), and ships a supported conversion path in the
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

use crate::{extract_indexes_used, EqlV2Encrypted};
use anyhow::{Context, Result};
use cipherstash_client::{
    encryption::{Plaintext, ScopedCipher},
    eql::{decrypt_eql, encrypt_eql, EqlCiphertext, EqlOperation, EqlOutput, PreparedPlaintext},
    AutoStrategy,
};
use eql_bindings::from_v2::{from_v2, from_v2_query};
// Re-exported so ingest binaries can name their conversion target as a
// typed TargetDomain at the builder call site (IngestOptionsBuilder::
// convert_to_v3) without depending on eql-bindings directly.
pub use eql_bindings::from_v2::TargetDomain;
use serde_json::{json, Value};
use sqlx::postgres::{PgTypeInfo, PgTypeKind, PgValueRef};
use sqlx::types::Json;
use std::borrow::Cow;
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
    to_v3_stored_target(v2, parse_target(target)?)
}

/// [`to_v3_stored`] with an already-resolved [`TargetDomain`] — the hot-loop
/// form used by the ingest pipeline (`IngestOptionsBuilder::convert_to_v3`),
/// where the target is parsed once at the builder call site.
pub fn to_v3_stored_target(v2: &EqlCiphertext, target: TargetDomain) -> Result<Value> {
    let v2_value = serde_json::to_value(v2).context("serialize v2 ciphertext")?;
    from_v2(&v2_value, target)
        .with_context(|| format!("from_v2 conversion to `{:?}` failed", target))
}

/// Convert a v2 SteVec QUERY payload into the v3 SteVec containment-needle
/// JSON (the `{"sv":[…]}` shape fed to `eql_v3.jsonb_contains(value, $1)` —
/// alpha.3 dropped the dedicated `jsonb_query` type; the needle is a plain
/// jsonb value).
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


/// Convert a stored v2 row (as decoded by [`EqlV2Encrypted`]) to the v3
/// payload — used by cross-version correctness checks that read a v2 table
/// and probe the v3 table with the same logical value.
pub fn v2_row_to_v3_stored(row: EqlV2Encrypted, target: &str) -> Result<Value> {
    to_v3_stored(&row.into_ciphertext(), target)
}
