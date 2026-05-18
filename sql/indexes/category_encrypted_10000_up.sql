CREATE INDEX
category_encrypted_10000_hash_index
ON category_encrypted_10000 using hash (
    eql_v2.hmac_256(value)
);
