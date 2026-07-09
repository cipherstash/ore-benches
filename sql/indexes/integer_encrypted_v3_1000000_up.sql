-- EQL v3 ORE ordering index (public.integer_ord): functional btree over the
-- ore_block_256 composite, compared by the custom plpgsql opclass. Build
-- time at scale is a named risk — captured by prepare:_table.

CREATE INDEX integer_encrypted_v3_1000000_ord_index
ON integer_encrypted_v3_1000000 (eql_v3.ord_term(value));
