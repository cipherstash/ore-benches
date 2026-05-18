CREATE INDEX
category_encrypted_100000_hash_index
ON category_encrypted_100000 using hash (
    eql_v2.hmac_256(value)
);
