CREATE INDEX
json_ste_vec_small_encrypted_100000_ste_vec_index
ON json_ste_vec_small_encrypted_100000 USING GIN (
    eql_v2.ste_vec(value)
);

CREATE INDEX
json_ste_vec_small_encrypted_100000_eql_index
ON json_ste_vec_small_encrypted_100000 (
    value eql_v2.encrypted_operator_class
);
