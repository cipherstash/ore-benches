-- EQL v3 SteVec containment index — the recipe documented in the v3 bundle
-- itself (src/v3/jsonb/types.sql): value @> $1::eql_v3.jsonb_query inlines
-- to a native jsonb @> over eql_v3.to_ste_vec_query(value)::jsonb, served
-- by this jsonb_path_ops GIN. Per-selector functional btrees for the
-- field_eq/field_order scenarios are created by the bench at startup
-- (benches/json_v3.rs), mirroring the v2 json bench.

CREATE INDEX json_ste_vec_small_encrypted_v3_1000000_stevec_query_index
ON json_ste_vec_small_encrypted_v3_1000000 USING GIN (
    (eql_v3.to_ste_vec_query(value)::jsonb) jsonb_path_ops
);
