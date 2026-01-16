CREATE TABLE IF NOT EXISTS integer_plaintext (
    id SERIAL PRIMARY KEY,
    value INT NOT NULL
);

CREATE TABLE IF NOT EXISTS integer_encrypted (
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

CREATE TABLE IF NOT EXISTS json_small_plaintext (
    id SERIAL PRIMARY KEY,
    value JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS json_small_encrypted (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);
