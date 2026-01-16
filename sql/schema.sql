CREATE TABLE integer_plaintext (
    id SERIAL PRIMARY KEY,
    value INT NOT NULL
);

CREATE TABLE integer_encrypted (
    id SERIAL PRIMARY KEY,
    value cs_encrypted_v2 NOT NULL
);
