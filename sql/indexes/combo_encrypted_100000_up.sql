CREATE INDEX
combo_encrypted_100000_name_gin_index
ON combo_encrypted_100000 USING GIN (
    eql_v2.bloom_filter(name)
);

CREATE INDEX
combo_encrypted_100000_age_ore_index
ON combo_encrypted_100000 (
    eql_v2.ore_block_u64_8_256(age)
);

CREATE INDEX
combo_encrypted_100000_category_hash_index
ON combo_encrypted_100000 USING hash (
    eql_v2.hmac_256(category)
);
