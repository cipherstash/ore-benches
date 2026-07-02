-- eql_v3 GIN index for json_ste_vec_small_encrypted_v3_1000000
-- (column typed eql_v3.json).
--
-- The canonical v3 containment recipe: the typed
-- `@>(eql_v3.json, eql_v3.jsonb_query)` operator inlines to a native
-- `jsonb @>` over eql_v3.to_ste_vec_query(value)::jsonb, so a functional
-- GIN on the same expression engages. One index serves whole-document
-- containment AND field-level needle containment (the needle is
-- `{"sv":[{s, hm|oc}]}`), replacing the two v2 GINs (jsonb_array +
-- to_stevec_query).
--
-- The per-selector field_eq / field_order functional indexes are built by
-- benches/json_v3.rs at startup — the selector hash is only known once the
-- bench has sampled a row, so they cannot live in this static file.

CREATE INDEX
json_ste_vec_small_encrypted_v3_1000000_ste_vec_query_index
ON json_ste_vec_small_encrypted_v3_1000000 USING GIN (
    (eql_v3.to_ste_vec_query(value)::jsonb) jsonb_path_ops
);
