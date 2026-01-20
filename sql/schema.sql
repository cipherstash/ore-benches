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