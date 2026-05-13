CREATE INDEX
json_ste_vec_small_encrypted_10000000_ste_vec_index
ON json_ste_vec_small_encrypted_10000000 USING GIN (
    eql_v2.ste_vec(value)
);
