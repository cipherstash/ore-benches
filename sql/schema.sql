CREATE TABLE IF NOT EXISTS integer_plaintext (
    id SERIAL PRIMARY KEY,
    value INT NOT NULL
);

CREATE TABLE IF NOT EXISTS integer_encrypted (
    id SERIAL PRIMARY KEY,
    value eql_v2_encrypted NOT NULL
);
