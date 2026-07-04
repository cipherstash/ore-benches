-- Plaintext baseline tables for the v3 encrypted-vs-plaintext comparison
-- (the docs/marketing "overhead vs native Postgres" story). Same tiers as
-- the encrypted tables; populated by pure SQL (`prepare:integer_plaintext`,
-- `prepare:json_small_plaintext`) — no encryption client involved.
--
-- string_plaintext_<N> and category_plaintext_<N> already exist in
-- sql/schema.sql (shared with the v2 benches).

CREATE TABLE IF NOT EXISTS integer_plaintext_10000 (
    id SERIAL PRIMARY KEY,
    value INT NOT NULL
);

CREATE TABLE IF NOT EXISTS integer_plaintext_100000 (
    id SERIAL PRIMARY KEY,
    value INT NOT NULL
);

CREATE TABLE IF NOT EXISTS integer_plaintext_1000000 (
    id SERIAL PRIMARY KEY,
    value INT NOT NULL
);

-- Small JSON documents shaped like FakeJsonSmall (first_name, last_name,
-- age, email) so the containment / field-eq scenarios compare like shapes.
CREATE TABLE IF NOT EXISTS json_small_plaintext_10000 (
    id SERIAL PRIMARY KEY,
    value JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS json_small_plaintext_100000 (
    id SERIAL PRIMARY KEY,
    value JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS json_small_plaintext_1000000 (
    id SERIAL PRIMARY KEY,
    value JSONB NOT NULL
);
