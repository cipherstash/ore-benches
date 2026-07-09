-- EQL v3 whole-document containment index for the json ste_vec bench.
-- eql_v3.jsonb_contains(value, needle) inlines to
-- eql_v3.jsonb_array(value) @> eql_v3.jsonb_array(needle), so this GIN over
-- eql_v3.jsonb_array(value) serves both the contains/functional (whole-doc
-- needle) and field_eq/extractor (single-entry needle) scenarios. The
-- per-selector functional btrees for field_eq/field_order are built by the
-- bench at startup (benches/json_v3.rs), mirroring the v2 json bench.

CREATE INDEX json_ste_vec_small_encrypted_v3_1000000_jsonb_array_index
ON json_ste_vec_small_encrypted_v3_1000000 USING GIN (
    eql_v3.jsonb_array(value)
);
