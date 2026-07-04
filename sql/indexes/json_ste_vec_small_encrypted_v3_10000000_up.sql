CREATE INDEX json_ste_vec_small_encrypted_v3_10000000_stevec_query_index
ON json_ste_vec_small_encrypted_v3_10000000 USING GIN (
    (eql_v3.to_ste_vec_query(value)::jsonb) jsonb_path_ops
);
