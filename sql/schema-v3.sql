-- EQL v3 twins of the benchmarked tables. Applied by `mise run setup-db-v3`
-- AFTER the eql_v3 SQL installer (built from the EQL repo via `mise run
-- build`; there is no released v3 artifact yet).
--
-- v3 has no generic `eql_v2_encrypted`-style envelope type: each column is
-- typed as the per-scalar-per-capability jsonb domain that carries exactly
-- the index terms its bench scenarios need. Mapping from the v2 tables:
--
--   string_encrypted    → eql_v3.text_search   (hm + ob + bf)
--       The v2 string table serves BOTH the EXACT (hmac equality) and MATCH
--       (bloom containment) scenario families from one column. No single v3
--       domain carries hm + bf without ob, so `text_search` — the "all text
--       capabilities" domain — is the only single-column twin that supports
--       both families. Cost of that choice: the v3 string ingest additionally
--       encrypts an ORE term (`ob`) that v2's encrypt_string does not, which
--       is visible in the encrypt_string_v3 ingest numbers (documented in
--       the bin and README).
--   integer_encrypted   → eql_v3.integer_ord_ore  (ob)
--       The v2 int scenarios encrypt `i32` via ColumnType::Int (int4, not
--       int8) with an ORE index only.
--   category_encrypted  → eql_v3.text_eq       (hm)
--       Equality/GROUP BY only.
--   combo_encrypted     → name eql_v3.text_match / age eql_v3.integer_ord_ore /
--                         category eql_v3.text_eq
--       Per-column capability match for the composite-predicate scenarios
--       (bloom containment + ORE order + hmac GROUP BY). v3 removes LIKE, so
--       `name` needs only the bloom term.
--   json_ste_vec_*      → eql_v3.json          (ste_vec document domain)
--
-- Plaintext baseline tables (string_plaintext_*, category_plaintext_*, ...)
-- are version-independent and shared with the v2 schema — not duplicated
-- here.
--
-- TODO(CIP-3280): `_ord_ope` scenario scaffold. eql_v3 ships `_ord_ope`
-- domains (wire key `op`, extractor `eql_v3.ord_ope_term`, OPE-CLLW ordering
-- via native bytea comparison — the Supabase-friendly ordered path), but
-- cipherstash-client 0.38.0 does NOT emit the `op` term (CIP-3280 is
-- unreleased). Once a client release emits `op`:
--   1. add `integer_ope_encrypted_v3*` tables typed eql_v3.int4_ord_ope,
--   2. add btree indexes on eql_v3.ord_ope_term(value) under sql/indexes/v3/,
--   3. enable the ope scenario stub in benches/ore_v3.rs.

-- Base (un-tiered) tables used by the ingest throughput benches.

CREATE TABLE IF NOT EXISTS string_encrypted_v3 (
    id SERIAL PRIMARY KEY,
    value eql_v3.text_search NOT NULL
);

CREATE TABLE IF NOT EXISTS integer_encrypted_v3 (
    id SERIAL PRIMARY KEY,
    value eql_v3.integer_ord_ore NOT NULL
);

CREATE TABLE IF NOT EXISTS json_ste_vec_small_encrypted_v3 (
    id SERIAL PRIMARY KEY,
    value eql_v3.json NOT NULL
);

-- Row-count-tier tables used by the query benches
-- (`<base>_v3_<tier>`, populated by the prepare:v3:* tasks).

CREATE TABLE IF NOT EXISTS string_encrypted_v3_10000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.text_search NOT NULL
);

CREATE TABLE IF NOT EXISTS string_encrypted_v3_100000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.text_search NOT NULL
);

CREATE TABLE IF NOT EXISTS string_encrypted_v3_1000000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.text_search NOT NULL
);

CREATE TABLE IF NOT EXISTS string_encrypted_v3_10000000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.text_search NOT NULL
);

CREATE TABLE IF NOT EXISTS integer_encrypted_v3_10000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.integer_ord_ore NOT NULL
);

CREATE TABLE IF NOT EXISTS integer_encrypted_v3_100000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.integer_ord_ore NOT NULL
);

CREATE TABLE IF NOT EXISTS integer_encrypted_v3_1000000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.integer_ord_ore NOT NULL
);

CREATE TABLE IF NOT EXISTS integer_encrypted_v3_10000000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.integer_ord_ore NOT NULL
);

CREATE TABLE IF NOT EXISTS category_encrypted_v3_10000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.text_eq NOT NULL
);

CREATE TABLE IF NOT EXISTS category_encrypted_v3_100000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.text_eq NOT NULL
);

CREATE TABLE IF NOT EXISTS category_encrypted_v3_1000000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.text_eq NOT NULL
);

CREATE TABLE IF NOT EXISTS category_encrypted_v3_10000000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.text_eq NOT NULL
);

CREATE TABLE IF NOT EXISTS combo_encrypted_v3_10000 (
    id SERIAL PRIMARY KEY,
    name eql_v3.text_match NOT NULL,
    age eql_v3.integer_ord_ore NOT NULL,
    category eql_v3.text_eq NOT NULL
);

CREATE TABLE IF NOT EXISTS combo_encrypted_v3_100000 (
    id SERIAL PRIMARY KEY,
    name eql_v3.text_match NOT NULL,
    age eql_v3.integer_ord_ore NOT NULL,
    category eql_v3.text_eq NOT NULL
);

CREATE TABLE IF NOT EXISTS combo_encrypted_v3_1000000 (
    id SERIAL PRIMARY KEY,
    name eql_v3.text_match NOT NULL,
    age eql_v3.integer_ord_ore NOT NULL,
    category eql_v3.text_eq NOT NULL
);

CREATE TABLE IF NOT EXISTS combo_encrypted_v3_10000000 (
    id SERIAL PRIMARY KEY,
    name eql_v3.text_match NOT NULL,
    age eql_v3.integer_ord_ore NOT NULL,
    category eql_v3.text_eq NOT NULL
);

CREATE TABLE IF NOT EXISTS json_ste_vec_small_encrypted_v3_10000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.json NOT NULL
);

CREATE TABLE IF NOT EXISTS json_ste_vec_small_encrypted_v3_100000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.json NOT NULL
);

CREATE TABLE IF NOT EXISTS json_ste_vec_small_encrypted_v3_1000000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.json NOT NULL
);

CREATE TABLE IF NOT EXISTS json_ste_vec_small_encrypted_v3_10000000 (
    id SERIAL PRIMARY KEY,
    value eql_v3.json NOT NULL
);
