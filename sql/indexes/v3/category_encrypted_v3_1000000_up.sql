-- eql_v3 functional index for category_encrypted_v3_1000000
-- (column typed eql_v3.text_eq — hm). The GROUP_BY_V3 scenarios group on
-- eql_v3.eq_term(value); like the v2 hmac hash index, this index exists for
-- equality lookups — GROUP BY itself is served by HashAggregate.

CREATE INDEX
category_encrypted_v3_1000000_eq_term_index
ON category_encrypted_v3_1000000 USING hash (
    eql_v3.eq_term(value)
);
