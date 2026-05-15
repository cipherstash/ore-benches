CREATE TABLE IF NOT EXISTS integer_plaintext (
    id SERIAL PRIMARY KEY,
    value INT NOT NULL
);

CREATE TABLE IF NOT EXISTS integer_encrypted (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS integer_encrypted_10000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS integer_encrypted_100000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS integer_encrypted_1000000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS integer_encrypted_10000000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS string_plaintext (
    id SERIAL PRIMARY KEY,
    value TEXT NOT NULL
);

-- Sized plaintext variants for the GROUP BY plaintext-baseline bench.
-- Populated via SQL (`md5(random()::text)`) by `prepare:string_plaintext`;
-- no encryption-client dependency so they're cheap to populate.
CREATE TABLE IF NOT EXISTS string_plaintext_10000 (
    id SERIAL PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS string_plaintext_100000 (
    id SERIAL PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS string_plaintext_1000000 (
    id SERIAL PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS string_plaintext_10000000 (
    id SERIAL PRIMARY KEY,
    value TEXT NOT NULL
);

-- Low-cardinality categorical data (~250 distinct buckets, uniform random)
-- for "realistic GROUP BY" scenarios. The encrypted variant carries an `hm`
-- HMAC term so `GROUP BY eql_v2.hmac_256(value)` and `WHERE eql_v2.hmac_256(value) = ...`
-- both engage their respective indexes. The plaintext variant is a baseline
-- for the same query shape against an unindexed TEXT column.
CREATE TABLE IF NOT EXISTS category_encrypted_10000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS category_encrypted_100000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS category_encrypted_1000000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS category_encrypted_10000000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS category_plaintext_10000 (
    id SERIAL PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS category_plaintext_100000 (
    id SERIAL PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS category_plaintext_1000000 (
    id SERIAL PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS category_plaintext_10000000 (
    id SERIAL PRIMARY KEY,
    value TEXT NOT NULL
);

-- Combo: a single row carries three encrypted columns — `name` (match + hmac
-- via the unique search index), `age` (ORE), `category` (hmac for equality
-- and GROUP BY). Used by `benches/combo.rs` to exercise composite-predicate
-- shapes the EQL query-performance guide §6 describes (bloom + ORE order +
-- limit, filtered GROUP BY, top-N filtered GROUP BY).
CREATE TABLE IF NOT EXISTS combo_encrypted_10000 (
    id SERIAL PRIMARY KEY,
    name eql_v2_encrypted NOT NULL,
    age eql_v2_encrypted NOT NULL,
    category eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS combo_encrypted_100000 (
    id SERIAL PRIMARY KEY,
    name eql_v2_encrypted NOT NULL,
    age eql_v2_encrypted NOT NULL,
    category eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS combo_encrypted_1000000 (
    id SERIAL PRIMARY KEY,
    name eql_v2_encrypted NOT NULL,
    age eql_v2_encrypted NOT NULL,
    category eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS combo_encrypted_10000000 (
    id SERIAL PRIMARY KEY,
    name eql_v2_encrypted NOT NULL,
    age eql_v2_encrypted NOT NULL,
    category eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS string_encrypted (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS string_encrypted_10000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS string_encrypted_100000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS string_encrypted_1000000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS string_encrypted_10000000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS json_small_plaintext (
    id SERIAL PRIMARY KEY,
    value JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS json_ste_vec_small_encrypted (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS json_ste_vec_small_encrypted_10000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS json_ste_vec_small_encrypted_100000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS json_ste_vec_small_encrypted_1000000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS json_ste_vec_small_encrypted_10000000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS json_small_encrypted (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS json_small_encrypted_10000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS json_small_encrypted_100000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS json_small_encrypted_1000000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS json_small_encrypted_10000000 (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);

CREATE TABLE IF NOT EXISTS json_large_plaintext (
    id SERIAL PRIMARY KEY,
    value JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS json_large_encrypted (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);