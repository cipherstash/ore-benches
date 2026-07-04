CREATE INDEX category_encrypted_v3_10000000_eq_btree_index
ON category_encrypted_v3_10000000 (eql_v3.eq_term(value));
