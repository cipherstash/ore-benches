-- eql_v3 functional index for integer_encrypted_v3_100000
-- (column typed eql_v3.integer_ord_ore — ob).
--
-- ord_term returns eql_v3.ore_block_256, whose btree operator class is the
-- DEFAULT for the type — bare-form range predicates (`value > $1`) inline
-- to eql_v3.ord_term(value) > eql_v3.ord_term($1) and match this index;
-- extractor-form ORDER BY streams rows out of it already sorted.

CREATE INDEX
integer_encrypted_v3_100000_ord_term_index
ON integer_encrypted_v3_100000 USING btree (
    eql_v3.ord_term(value)
);
