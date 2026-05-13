CREATE INDEX
integer_encrypted_10000000_ore_index
ON integer_encrypted_10000000 (
    eql_v2.ore_block_u64_8_256(value)
);
