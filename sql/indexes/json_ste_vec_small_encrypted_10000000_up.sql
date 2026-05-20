-- EQL 2.3 functional GIN indexes for the json ste_vec bench.
--
-- jsonb_array  — whole-document containment (contains/functional):
--                eql_v2.jsonb_array(value) @> eql_v2.jsonb_array($1).
-- stevec_query — typed field-level containment (field_eq/extractor):
--                value @> $1::eql_v2.stevec_query inlines to a native
--                jsonb @> over eql_v2.to_stevec_query(value)::jsonb.
--                XOR-aware: one index covers hm- and oc-bearing selectors.
--
-- Replaces the pre-2.3 eql_v2.ste_vec / eql_v2.hmac_256_terms GIN indexes
-- (hmac_256_terms was removed in EQL 2.3 — see cipherstash/eql#223).

CREATE INDEX
json_ste_vec_small_encrypted_10000000_jsonb_array_index
ON json_ste_vec_small_encrypted_10000000 USING GIN (
    eql_v2.jsonb_array(value)
);

CREATE INDEX
json_ste_vec_small_encrypted_10000000_stevec_query_index
ON json_ste_vec_small_encrypted_10000000 USING GIN (
    (eql_v2.to_stevec_query(value)::jsonb) jsonb_path_ops
);
