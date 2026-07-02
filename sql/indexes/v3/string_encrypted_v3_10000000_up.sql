-- eql_v3 functional indexes for string_encrypted_v3_10000000
-- (column typed eql_v3.text_search — hm + ob + bf).
--
-- eq_term    — hash index; engages for equality (EXACT_V3 scenarios). The
--              inlinable `=` operator and the explicit extractor form both
--              reduce to eql_v3.eq_term(value) = eql_v3.eq_term($1).
-- match_term — GIN over eql_v3.bloom_filter (smallint[]); engages for the
--              bloom containment `@>` (MATCH_V3). v3 removes LIKE/ILIKE.
--
-- No ord_term index: no v3 string scenario orders or range-scans the
-- column (the ob term is present because text_search requires it, not
-- because a scenario uses it).

CREATE INDEX
string_encrypted_v3_10000000_eq_term_index
ON string_encrypted_v3_10000000 USING hash (
    eql_v3.eq_term(value)
);

CREATE INDEX
string_encrypted_v3_10000000_match_term_index
ON string_encrypted_v3_10000000 USING GIN (
    eql_v3.match_term(value)
);
