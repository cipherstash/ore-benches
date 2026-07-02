-- eql_v3 functional indexes for combo_encrypted_v3_1000000 — per-column
-- capability twins of the v2 combo indexes:
--   name     eql_v3.text_match   → GIN on match_term (bloom containment;
--                                   replaces the v2 bloom_filter GIN + LIKE)
--   age      eql_v3.int4_ord_ore → btree on ord_term (ORE ordering)
--   category eql_v3.text_eq      → hash on eq_term (hmac equality)

CREATE INDEX
combo_encrypted_v3_1000000_name_match_term_index
ON combo_encrypted_v3_1000000 USING GIN (
    eql_v3.match_term(name)
);

CREATE INDEX
combo_encrypted_v3_1000000_age_ord_term_index
ON combo_encrypted_v3_1000000 USING btree (
    eql_v3.ord_term(age)
);

CREATE INDEX
combo_encrypted_v3_1000000_category_eq_term_index
ON combo_encrypted_v3_1000000 USING hash (
    eql_v3.eq_term(category)
);
