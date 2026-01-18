CREATE INDEX
string_encrypted_hash_index
ON string_encrypted using hash (
    eql_v2.hmac_256(value)
);

CREATE INDEX
string_encrypted_gin_index
ON string_encrypted USING GIN (
    eql_v2.bloom_filter(value)
);

CREATE INDEX
string_encrypted_eql_index
ON string_encrypted (
    value eql_v2.encrypted_operator_class
);

