CREATE INDEX
json_ste_vec_small_encrypted_1000000_ste_vec_index
ON json_ste_vec_small_encrypted_1000000 USING GIN (
    eql_v2.ste_vec(value)
);

CREATE INDEX
json_ste_vec_small_encrypted_1000000_eql_index
ON json_ste_vec_small_encrypted_1000000 (
    value eql_v2.encrypted_operator_class
);
