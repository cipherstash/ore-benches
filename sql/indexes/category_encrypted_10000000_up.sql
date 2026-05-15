CREATE INDEX
category_encrypted_10000000_hash_index
ON category_encrypted_10000000 using hash (
    eql_v2.hmac_256(value)
);
