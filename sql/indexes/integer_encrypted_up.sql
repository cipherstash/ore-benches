-- Functional B-tree index over the ORE term. Range queries and ORDER BY are
-- only accelerated when the predicate is written in extractor form, e.g.
--   WHERE eql_v2.ore_block_u64_8_256(value) > eql_v2.ore_block_u64_8_256($1::jsonb)
--   ORDER BY eql_v2.ore_block_u64_8_256(value)
-- The natural `WHERE value > $1` form falls through to a sequential scan
-- because the `>` operator on eql_v2_encrypted is plpgsql and not inlinable.
CREATE INDEX
integer_encrypted_ore_index
ON integer_encrypted (
    eql_v2.ore_block_u64_8_256(value)
);
