CREATE INDEX
string_encrypted_1000000_hash_index
ON string_encrypted_1000000 using hash (
    eql_v2.hmac_256(value)
);

CREATE INDEX
string_encrypted_1000000_gin_index
ON string_encrypted_1000000 USING GIN (
    eql_v2.bloom_filter(value)
);

CREATE INDEX
string_encrypted_1000000_eql_index
ON string_encrypted_1000000 (
    value eql_v2.encrypted_operator_class
);
