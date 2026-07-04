# EQL v3 Regression Analysis — Scenario Deep-Dives

Companion to the auto-generated [`report/V3_COMPARISON.md`](report/V3_COMPARISON.md).
That report flags *what* moved; this document works through flagged scenarios one
at a time — the exact queries both versions ran, what actually differs between
them, and the most likely causes of the delta, with supporting evidence from the
recorded plans and related scenarios.

Methodology reminders (details in `report/V3_COMPARISON.md`):

- v2 numbers are the committed EQL 2.3 baseline results; v3 numbers were produced
  on the same machine (M1 Max, native PG 17, port 5400) on 2026-07-03/04.
- Every query and bound parameter quoted here comes from the metadata sidecars
  the benches write at startup (`results/query/**/*_metadata_*.json`), which
  also capture `EXPLAIN (FORMAT JSON)` and the indexes the planner used.
- The v2 tables were truncated after the workspace re-login, so v2 queries can
  no longer be re-executed interactively; v2 evidence is the recorded sidecars.

---

## COMBO/combo/bloom_ore_order_limit

*"Find the 10 youngest customers whose first name matches a pattern"* — bloom
filter + ORE ORDER BY + LIMIT, the composite-predicate shape from §6 of the EQL
query-performance guide.

### Timings

| Tier | v2 median | v3 median | Δ |
|---|---|---|---|
| 10k | 0.287 ms | 0.525 ms | +82.8% |
| 100k | 2.083 ms | 2.780 ms | +33.5% |
| 1M | 16.643 ms | 22.703 ms | +36.4% |

### The queries

**v2** (`combo_encrypted_<N>`, columns `name`/`age`/`category` all `eql_v2_encrypted`):

```sql
SELECT id FROM combo_encrypted_1000000
WHERE name LIKE $1
ORDER BY eql_v2.ore_block_u64_8_256(age) LIMIT 10
```

Bound parameter: a v2 **query** payload, bloom-only — keys `{v, k, i, bf}`, no
ciphertext. Produced by `EqlOperation::Query` against the match index.

**v3** (`combo_encrypted_v3_<N>`, `name eql_v3.text_search`, `age eql_v3.integer_ord`):

```sql
SELECT id FROM combo_encrypted_v3_1000000
WHERE name @> $1
ORDER BY eql_v3.ord_term(age) LIMIT 10
```

Bound parameter: a full v3 **stored** payload — keys `{v, i, c, hm, ob, bf}`.
No v3 scalar query wire shape exists (every domain CHECK requires the
ciphertext `c`), so the needle is "Bob" encrypted with `EqlOperation::Store`
and converted via `eql-bindings::from_v2`.

Both versions engaged their bloom GIN (`..._name_gin_index` /
`..._name_match_gin_index` in `indexes_used`) at every tier, and both returned
10 rows.

### What changed between the scenarios

1. **Operator: `LIKE` → `@>`.** EQL v3 removed the `~~` operator; bloom
   matching is exposed only as containment. Semantically identical (needle
   bloom bits ⊆ value bloom bits), but a different operator entry inlining to
   `eql_v3.match_term(name) @> eql_v3.match_term($1::eql_v3.text_search)`,
   giving the planner a different expression tree.
2. **Parameter shape: bloom-only query payload → full stored envelope.** The
   v3 needle carries a ciphertext and two extra index terms, and the
   `::eql_v3.text_search` cast runs the domain CHECK on it in-plan. More bytes
   per execution, and the RHS `match_term()` extracts `bf` out of a larger
   jsonb.
3. **ORDER BY extractor: `eql_v2.ore_block_u64_8_256(age)` →
   `eql_v3.ord_term(age)`.** The v3 term is the `eql_v3_internal.ore_block_256`
   composite whose comparisons run through the custom btree operator class with
   a **plpgsql comparator** (`compare_ore_block_256_terms`). Every comparison
   in the top-10 sort over the bloom-matched rows pays a plpgsql call; the v2
   ORE type's comparison path was cheaper.
4. **Wider `name` rows.** `text_search` requires the ORE term (`ob`), which
   v2's unique+match config didn't carry — more heap bytes per row scanned.

### Likely cause, ranked

> **Update (validated):** the primary cause turned out to be the LANGUAGE
> sql extraction helpers in the ORE opclass path, not the comparator — see
> the attribution + patch experiments under Issue 3. With a logic-identical
> plpgsql swap of two helper functions, this scenario went from 22.70 ms to
> **13.97 ms at 1M — 16% faster than v2**. The analysis below is preserved
> as the pre-experiment reasoning.

**Primary suspect: the v3 ORE comparator in the sort (change 3).** The
standalone ordered-scan scenario isolates exactly this code path
(`ORE/ore/range_lt_ordered_10`: index scan whose descent/order comparisons go
through the same opclass), and it shows a stable, size-independent penalty of
the same magnitude:

| Tier | v2 ORE ordered | v3 ORE ordered | v3 OPE ordered (native bytea btree) |
|---|---|---|---|
| 10k | 0.481 ms | 0.760 ms | 0.118 ms |
| 100k | 0.502 ms | 0.735 ms | 0.119 ms |
| 1M | 0.513 ms | 0.736 ms | 0.118 ms |

The +33–36% at 100k/1M on this combo scenario tracks the +43–46% on the pure
ordered scan. In the combo shape the sort input is the bloom-matched row set
(a Top-N sort over ~thousands of rows at 1M), so ~N·log(10) plpgsql comparator
calls land directly on the critical path.

**Secondary: parameter size + domain-cast CHECK (change 2)** — a fixed
per-execution cost, which is why the *relative* delta is largest at 10k
(+82.8%), where the query is fastest and fixed costs weigh most.

**Minor: wider rows (change 4)** — more heap I/O per bloom-matched row; small
at these tiers.

The `@>`-vs-`LIKE` operator change itself (change 1) is likely noise: both
inline to the same bloom-containment expression shape and both engaged the GIN.
The standalone match scenarios (`MATCH/match/eql_cast_*`) sit at −6% to +7% at
100k/1M, so the operator swap alone does not explain a +36%.

### How to confirm / follow up

- **Isolate the comparator:** add a combo variant ordering by an OPE age column
  (`eql_v3.ord_ope_term`). The pure-scan evidence (0.118 ms flat) predicts the
  variant would land at or below the v2 number — if so, the regression is
  attributable to the ORE opclass, not the combo shape.
- **Upstream fix candidates:** a C-level or SQL-inlinable comparator for
  `ore_block_256` in EQL v3, or steering ordered workloads to `_ord_ope`
  domains — cipherstash-client 0.38.1 emits the `op` term via `Index::new_ope()` (CIP-3348, adopted 2026-07-04).
- Related finding, same root: the ORE index **build** at 1M took 44 s vs 1 s
  for OPE (`results/ingest/index_build_times.jsonl`).

---

## JSON/json/field_eq/functional

Per-selector functional equality on an encrypted JSON field, via the
extractor recipe. Flagged at +18.9…+20.6% across tiers (e.g. 100k:
0.11 → 0.13 ms).

### The queries

**v2** (`json_ste_vec_small_encrypted_<N>`):

```sql
SELECT id FROM json_ste_vec_small_encrypted_100000
WHERE eql_v2.eq_term(value -> '<sel>'::text)
    = eql_v2.eq_term($1::jsonb::eql_v2.ste_vec_entry) LIMIT 10
```

**v3** (`json_ste_vec_small_encrypted_v3_<N>`):

```sql
SELECT id FROM json_ste_vec_small_encrypted_v3_100000
WHERE eql_v3.eq_term(value -> '<sel>'::text)
    = eql_v3.eq_term($1::jsonb::eql_v3.jsonb_entry) LIMIT 10
```

Same shape, same needle construction, and — important — **the same index
type in both versions**: a per-selector btree on `eq_term(value -> sel)`,
built at bench startup (the v2 json bench had already moved off hash after
the hash-build derisk work; only `EXACT/eql_hash` changed index type between
versions). Both versions' sidecars show the btree engaged. Index type is
ruled out for this scenario.

### Attribution (same protocol as the ORE experiments; 100k, 200 warmed runs)

| | v2 (0.0109 ms/query in-DB) | v3 (0.0286 ms/query in-DB) |
|---|---|---|
| `eq_term` | 0.0055 ms | 0.0058 ms |
| `is_valid_ste_vec_entry_payload` | — | **0.0180 ms (1 call/query)** |

`eq_term` costs are identical. The entire in-DB delta is one new call per
query: the **`eql_v3.jsonb_entry` domain CHECK** evaluated when the needle
is cast. The absolute delta (+0.018 ms) matches the criterion-level
regression (+0.02 ms) almost exactly.

### Root cause — third instance of the non-inlinable LANGUAGE sql pattern

- v2's `ste_vec_entry` domain CHECK is an **inline expression**
  (`jsonb_typeof(VALUE) = 'object' AND VALUE ? 's' AND …`) — evaluated
  directly by the executor, no function call.
- v3's `jsonb_entry` domain CHECK is
  `CHECK (eql_v3_internal.is_valid_ste_vec_entry_payload(VALUE))` — a
  LANGUAGE sql function, and domain constraints are another context where
  SQL functions **cannot be inlined**, so every cast pays the per-call
  SQL-function executor (~18 µs).

Fix: inline the validation expression directly into the CHECK constraint
(as v2 did, and as the generated scalar domains already do). See issue 4.

### Related: btree vs hash for `EXACT/eql_hash` (where the index type DID change)

Direct A/B on `string_encrypted_v3_100000` (create hash on
`eq_term(value)`, drop btree, time, restore): btree 0.0115 ms → hash
0.0076 ms → btree restored 0.0113 ms per lookup. Hash is genuinely ~35%
faster in-DB for the point lookup, but the absolute difference (0.004 ms)
covers only a fraction of that scenario's criterion delta (~0.015–0.02 ms);
the rest is the stored-envelope needle (issue 1). The btree choice remains
correct for build-time reasons (hash builds degrade badly at scale).

---

*Further scenarios to be added as triage continues.*

---

## Issues

Candidate EQL v3 issues surfaced by this analysis, to be filed in Linear.

### 1. Add a `k = "q"` query payload variant (no `c` field) for scalar domains

**Problem:** v3 has no scalar query wire shape. Every scalar domain CHECK
requires the ciphertext key `c`, and the `RIGHTARG = jsonb` operator overloads
cast the bound parameter to the domain — so a query needle must be a full
stored envelope even though `c` plays no part in any comparison. Consequences:

- Clients must run full encryption (record ciphertext + every required term)
  just to build a WHERE-clause parameter; v2 generated only the one term the
  predicate needed.
- Every scalar query ships a larger parameter and evaluates the domain CHECK
  on it in-plan (e.g. `{v, i, c, hm, ob, bf}` for `text_search` vs v2's
  bloom-only `{v, k, i, bf}`).
- `eql-bindings::from_v2_query` deliberately refuses scalar targets
  (`UnsupportedQueryTarget`) rather than invent a shape ahead of this design.

**Proposal (from team discussion 2026-07-03):** add a `k = "q"` payload
variant with no `c` field — either accepted by the existing domains' CHECKs
(`c` required only when `k != "q"`) or as parallel `*_query` domain types the
comparison wrappers accept on the RHS (the pattern `eql_v3.jsonb_query`
already establishes for SteVec needles). `from_v2_query` can then support
scalar targets, and the bench harness's Store-shaped needle workaround
(`src/v3.rs` module docs) goes away.

**Evidence:** parameter shapes recorded in
`results/query/v3/*_metadata_*.json`; fixed per-execution overhead visible as
the inflated small-tier deltas (e.g. COMBO bloom_ore_order_limit +82.8% at 10k
vs +36.4% at 1M).

### 2. No hm+bf-only text domain — `text_search` forces the ORE term

**Problem:** the only v3 text domain with both equality and match capability
(`text_search`) also requires `ob`. A v2 unique+match column migrating to v3
must start generating ORE terms, capping string ingest at ORE-generation speed:
**9,649 → 1,278 rec/s (−87%)** at the 10k batch size. Rows are also wider.

**Proposal:** add an hm+bf domain (e.g. `text_eq_match`) to the catalog, or
document the cost as an accepted trade-off of the v3 type system.

**Evidence:** `report/V3_COMPARISON.md` ingest table;
`results/ingest/v3/encrypt_string_v3_combined.json`.

### 3. `ore_block_256` opclass path overhead — SQL-language helpers regressed vs v2; whole chain trails OPE

**Problem (two distinct layers, per the attribution experiment below):**

- **v2→v3 ordered-scan regression (+43–46%,
  `ORE/ore/range_lt_ordered_10`: 0.51 → 0.74 ms):** NOT the comparator —
  per-query comparator time is equal across versions. The measured driver is
  the term-extraction helpers (`jsonb_array_to_bytea_array`,
  `jsonb_array_to_ore_block_256`), rewritten from v2's plpgsql to
  LANGUAGE sql in v3: in the opclass call path they cannot inline and pay
  the per-call SQL-function executor — 3.5× v2's per-call cost for
  identical logic. The stored-envelope needle's domain-cast CHECK (issue 1)
  contributes fixed per-query overhead on top.
- **ORE-vs-OPE gap (0.74 vs 0.12 ms; index build 44 s vs 1 s at 1M):** the
  comparator+extraction machinery as a whole — ~0.4–0.5 ms of tracked
  function self-time per query on either ORE version, vs zero function
  calls on the fully-inlined OPE path
  (`results/ingest/index_build_times.jsonl`).

**Proposal:** (a) quick win for the regression — revert the opclass-path
helpers to plpgsql (LANGUAGE sql buys nothing where inlining is impossible)
or cache the decoded bytea representation. **Validated locally**: a
logic-identical language-only swap of the two helpers eliminated the entire
regression (0.32 → 0.19 ms/query, marginally faster than v2 on the same
harness; see the A→B→A experiment in the COMBO section). (b) structural — a
C-level decode+compare for `ore_block_256`, and/or position `_ord_ope` as
the recommended ordering path now that cipherstash-client 0.38.1 emits the `op` term (CIP-3348)
— the OPE benches show 0.118 ms flat at every tier, ~1.2× plaintext.

**Evidence:** `report/v3/ore_vs_ope_*.png`; ordered-scan table in the COMBO
section above; function-level attribution experiment below.

### 4. `jsonb_entry` domain CHECK calls a non-inlinable SQL function — and the pattern is systemic

**Problem:** `eql_v3.jsonb_entry`'s CHECK is
`eql_v3_internal.is_valid_ste_vec_entry_payload(VALUE)` — a LANGUAGE sql
function. Domain constraints cannot inline SQL functions, so every cast to
`jsonb_entry` (e.g. the needle in every `field_eq` query) pays ~18 µs of
SQL-function-executor overhead. Measured: this one call is the ENTIRE
v2→v3 regression on `JSON/json/field_eq/functional` (+19%); v2's
`ste_vec_entry` CHECK was an inline expression and cost nothing
measurable.

**Proposal:** inline the validation expression directly into the CHECK
constraint (v2's approach; the generated scalar domains already do this).
More broadly: **audit every LANGUAGE sql function reachable from a
non-inlinable context** — domain CHECK constraints, btree operator-class
support paths, and any function invoked from plpgsql — this is the third
confirmed instance of the pattern (with the two `ore_block_256` extraction
helpers of issue 3), and "LANGUAGE sql for inlineability" is v3's global
strategy, so more instances likely exist.

**Evidence:** attribution tables in the field_eq section of this report;
`pg_get_constraintdef` comparison of `ste_vec_entry` (v2) vs `jsonb_entry`
(v3).

#### Experiment: function-level attribution (`track_functions`), v2 vs v3 vs OPE

Protocol: `SET track_functions = 'all'`, warm the query in one session,
`pg_stat_reset()`, run the bench-shaped query 10× in a fresh session, read
`pg_stat_user_functions` (per-query = totals / 10). Query:
`WHERE value < <needle> ORDER BY <extractor>(value) LIMIT 10` against the
1M tables, needle inlined as a literal. The v2 table was repopulated for
this (the v2 SQL surface was still installed; only the data had been
truncated).

Per-query self-time, grouped:

| | v2 ORE (median 0.513 ms) | v3 ORE (median 0.736 ms) | v3 OPE (median 0.118 ms) |
|---|---|---|---|
| comparator chain (`compare_*_term*`) | 0.200 ms (15.6 calls) | 0.192 ms (15.7 calls) | — |
| extraction chain (`*_to_bytea_array`, `*ore_block*`, `has_*`) | 0.179 ms (10.1 calls) | 0.264 ms (11.1 calls) | — |
| — of which `jsonb_array_to_bytea_array` | 0.046 ms | **0.160 ms** | — |
| **total tracked** | **≈ 0.40 ms** | **≈ 0.47 ms** | **0 (no calls at all)** |

Three separate conclusions, one per comparison:

1. **The comparator does NOT explain the v2→v3 regression.** Per-query
   comparator time is equal across versions (0.200 vs 0.192 ms, same call
   counts) — consistent with the near-identical plpgsql bodies (v3 only adds
   deriving the block count N from the ciphertext length plus a
   well-formedness check).
2. **The v3 extraction chain is measurably slower (+0.085 ms/query), led by
   `jsonb_array_to_bytea_array` at 3.5× the per-call cost.** The two
   implementations have identical logic; the difference is the language:
   v2's is **plpgsql**, v3's is **LANGUAGE sql**. In the opclass call path a
   SQL function cannot be inlined and goes through the per-call SQL-function
   executor, which is substantially slower than a cached plpgsql plan — v3's
   inline-everything strategy backfires in the one context where inlining is
   impossible. Tracked deltas cover ~0.09 ms of the ~0.22 ms bench delta;
   the remainder is untracked per-call fmgr overhead and the needle's
   domain-cast CHECK (v3 binds a stored envelope; v2 bound a terms-only
   query payload — issue 1).
3. **The comparator+extraction machinery IS why both ORE paths trail OPE.**
   The OPE run tracks zero function calls — `ord_ope_term` → `ope_cllw` →
   native bytea comparison inlines completely — and it runs at 0.118 ms
   flat vs ~0.4–0.5 ms of tracked function time on either ORE path.

Upstream implications: (a) for the v2→v3 regression specifically, revert the
opclass-path helpers (`jsonb_array_to_bytea_array`,
`jsonb_array_to_ore_block_256`) to plpgsql — LANGUAGE sql buys nothing where
inlining can't happen — or cache the decoded representation; (b) for the
larger ORE-vs-OPE gap, a C-level decode+compare or steering ordered
workloads to `_ord_ope` (client 0.38.1 emits `op`; adopted).

#### Validation: language-only swap of the helpers (A→B→A)

To test the hypothesis directly, both helpers were `CREATE OR REPLACE`d
locally with **logic-identical plpgsql** versions (no other change), timed
with a fixed harness (100 warmed executions of the bench-shaped query via a
plpgsql loop, `clock_timestamp()` wall time), then restored:

| variant | ms/query (harness) |
|---|---|
| v2 ORE (reference) | 0.199–0.209 |
| v3 ORE, shipped LANGUAGE sql helpers | 0.318–0.322 |
| **v3 ORE, plpgsql helpers** | **0.188–0.191** |
| v3 ORE, originals restored | 0.322 |

The language-only swap **eliminates the entire v2→v3 ordered-scan
regression** — patched v3 is marginally *faster* than v2 on the same
harness. Attribution after the swap: `jsonb_array_to_bytea_array` self-time
0.160 → 0.052 ms/query (v2's plpgsql equivalent: 0.046), comparator
unchanged (0.155 vs 0.152), call counts identical. Restoring the shipped
definitions returned timing to 0.322 ms.

Conclusion: **confirmed** — the v2→v3 ORE ordered-scan regression is caused
by the opclass-path helper functions being LANGUAGE sql, which cannot inline
when called from the btree support machinery and pay per-call SQL-function
executor overhead. The fix is a two-function language change (or better,
caching/C-level decode). Note the harness numbers exclude client/protocol
overhead, so absolute values sit below the criterion medians (0.513/0.736
ms); the *gap* is what transfers.

#### End-to-end validation: real criterion benches with the patch (1M rows)

Same patch applied, then the actual `bench:v3:query:ore` and
`bench:v3:query:combo` criterion runs (full sqlx/bound-parameter path);
functions and committed result files restored afterwards.

| Scenario | v2 | v3 shipped | v3 patched | patch effect |
|---|---|---|---|---|
| ORE/range_lt_ordered_10 | 0.513 ms | 0.736 ms | **0.553 ms** | **−24.9%** |
| COMBO/bloom_ore_order_limit | 16.643 ms | 22.703 ms | **13.969 ms** | **−38.5%** |
| ORE/range_{gt,lt}_{10,100} | — | — | — | −4.6…−8.3% |
| COMBO/filtered_group_by (control, no ORE ordering) | 6.396 ms | 5.303 ms | 5.438 ms | +2.6% (noise) |
| COMBO/top_n_filtered_group_by (control) | 5.287 ms | 5.339 ms | 5.342 ms | +0.1% (noise) |

- The ordered scan lands within 8% of v2 (residual consistent with the
  stored-envelope needle, issue 1, and wider payloads).
- **COMBO/bloom_ore_order_limit doesn't just recover — it beats v2 by 16%**
  (13.97 vs 16.64 ms): the Top-N sort over the bloom-matched set pays the
  extraction chain per compared row, so the helper fix has multiplied
  effect there.
- The no-ordering COMBO controls are unmoved (±2.6%), confirming the effect
  is specific to the ORE opclass path.

This also resolves the COMBO scenario's cause ranking (above): the
"comparator in the sort" framing was wrong — the validated driver is the
LANGUAGE sql extraction helpers invoked throughout the ORE ordering path.
