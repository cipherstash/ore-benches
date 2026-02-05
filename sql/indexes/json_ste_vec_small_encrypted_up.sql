CREATE INDEX
json_ste_vec_small_encrypted_ste_vec_index
ON json_ste_vec_small_encrypted USING GIN (
    eql_v2.ste_vec(value)
);
