-- EQL v3 CLLW-OPE ordering index (eql_v3.integer_ord_ope): the fast path.
-- ord_ope_term -> eql_v3_internal.ope_cllw (DOMAIN over bytea), so this is
-- a NATIVE bytea btree — no custom comparator anywhere in the chain.

CREATE INDEX integer_encrypted_ope_v3_10000_ope_index
ON integer_encrypted_ope_v3_10000 (eql_v3.ord_ope_term(value));
