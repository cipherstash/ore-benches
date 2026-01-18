CREATE INDEX
integer_encrypted_eql_index
ON integer_encrypted (
    value eql_v2.encrypted_operator_class
);
