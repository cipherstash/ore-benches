-- EQL v3 functional indexes for the string bench (public.text_search).
--
-- eq_term    -> eql_v3_internal.hmac_256 (DOMAIN over text): btree, NOT hash.
--   v2 used hash here; hash index builds degrade badly at scale (see the
--   benches/json.rs commentary and the native-pg derisk experiment), and
--   btree also serves GROUP BY. The comparison report flags the index-type
--   delta; a 10k-only hash datapoint disambiguates.
-- match_term -> eql_v3_internal.bloom_filter (DOMAIN over smallint[]): v3
--   ships no GIN opclass, so this GIN relies on the base type's native
--   array_ops. Engagement is smoke-gated (verify:v3-plans) — if it does not
--   engage, that is a headline finding for the release.
-- ord_term   -> eql_v3_internal.ore_block_256 (composite): custom btree
--   opclass with a plpgsql comparator; build time is captured by
--   prepare:_table into results/ingest/index_build_times.jsonl.

CREATE INDEX string_encrypted_v3_1000000_eq_btree_index
ON string_encrypted_v3_1000000 (eql_v3.eq_term(value));

CREATE INDEX string_encrypted_v3_1000000_match_gin_index
ON string_encrypted_v3_1000000 USING GIN (eql_v3.match_term(value));

CREATE INDEX string_encrypted_v3_1000000_ord_index
ON string_encrypted_v3_1000000 (eql_v3.ord_term(value));
