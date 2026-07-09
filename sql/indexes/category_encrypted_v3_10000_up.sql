-- EQL v3 equality index (public.text_eq): functional btree over eq_term
-- (hmac_256, DOMAIN over text). btree instead of v2's hash — see the
-- string index file for the rationale.

CREATE INDEX category_encrypted_v3_10000_eq_btree_index
ON category_encrypted_v3_10000 (eql_v3.eq_term(value));
