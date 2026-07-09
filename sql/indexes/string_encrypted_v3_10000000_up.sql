-- See string_encrypted_v3_10000_up.sql for the index rationale.
CREATE INDEX string_encrypted_v3_10000000_eq_btree_index
ON string_encrypted_v3_10000000 (eql_v3.eq_term(value));

CREATE INDEX string_encrypted_v3_10000000_match_gin_index
ON string_encrypted_v3_10000000 USING GIN (eql_v3.match_term(value));

CREATE INDEX string_encrypted_v3_10000000_ord_index
ON string_encrypted_v3_10000000 (eql_v3.ord_term(value));
