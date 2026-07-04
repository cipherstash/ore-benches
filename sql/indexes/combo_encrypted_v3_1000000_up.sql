-- EQL v3 combo indexes, mirroring the v2 combo set: bloom GIN on name,
-- ORE btree on age, equality btree on category (v2 used hash here).

CREATE INDEX combo_encrypted_v3_1000000_name_match_gin_index
ON combo_encrypted_v3_1000000 USING GIN (eql_v3.match_term(name));

CREATE INDEX combo_encrypted_v3_1000000_age_ord_index
ON combo_encrypted_v3_1000000 (eql_v3.ord_term(age));

CREATE INDEX combo_encrypted_v3_1000000_category_eq_btree_index
ON combo_encrypted_v3_1000000 (eql_v3.eq_term(category));
