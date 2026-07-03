-- eql_v3 functional index for integer_ope_encrypted_v3_10000
-- (column typed eql_v3.integer_ord_ope — op).
--
-- ord_ope_term returns eql_v3_internal.ope_cllw (a bytea domain), so the
-- btree uses bytea's native DEFAULT operator class — no per-row plpgsql
-- compare. Bare-form range predicates (`value > $1`) inline to
-- eql_v3.ord_ope_term(value) > eql_v3.ord_ope_term($1) and match this
-- index; extractor-form ORDER BY streams rows out of it already sorted.

CREATE INDEX
integer_ope_encrypted_v3_10000_ord_ope_term_index
ON integer_ope_encrypted_v3_10000 USING btree (
    eql_v3.ord_ope_term(value)
);
