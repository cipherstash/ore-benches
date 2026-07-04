# JSON Queries

[← Back to overview](./BENCHMARK_REPORT.md)

Per-tier query performance. Each scenario lists its SQL, the indexes available on the target table, the indexes the planner actually picked per tier, the timing table, and the full EXPLAIN plan in a collapsed block.

## contains/functional

**Description:** Whole-document JSON containment via `ste_vec(...) @> ste_vec(...)`

**SQL Query:**
```sql
SELECT id FROM {TABLE} WHERE eql_v2.ste_vec(value) @> eql_v2.ste_vec($1::jsonb::eql_v2_encrypted) LIMIT 10
```

**Parameter:** `<sampled-row-value-as-jsonb>`

**Table: `json_ste_vec_small_encrypted_{rows}` with encrypted JSON documents (small four-field shape — first_name / last_name / age / email). Index: functional GIN on `eql_v2.ste_vec(value)`. Both sides of `@>` resolve to `eql_v2_encrypted[]`, which matches the GIN opclass directly. The needle is a sampled row's value, so the query matches at least that source row.

Note: the bare form `WHERE value @> $1::eql_v2_encrypted` does NOT engage the GIN today. `eql_v2."@>"` is marked inlinable SQL but wraps `ste_vec_contains()` which is PL/pgSQL — inlining stops at the wrapper, leaving the planner with a black-box function call and no path to the indexed expression. The bench omits the bare form because it would not complete at the 1M / 10M tiers.**

**Indexes available on the table:**
```sql
-- EQL 2.3 functional GIN indexes for the json ste_vec bench.
--
-- jsonb_array  — whole-document containment (contains/functional):
--                eql_v2.jsonb_array(value) @> eql_v2.jsonb_array($1).
-- stevec_query — typed field-level containment (field_eq/extractor):
--                value @> $1::eql_v2.stevec_query inlines to a native
--                jsonb @> over eql_v2.to_stevec_query(value)::jsonb.
--                XOR-aware: one index covers hm- and oc-bearing selectors.
--
-- Replaces the pre-2.3 eql_v2.ste_vec / eql_v2.hmac_256_terms GIN indexes
-- (hmac_256_terms was removed in EQL 2.3 — see cipherstash/eql#223).

CREATE INDEX
json_ste_vec_small_encrypted_10000_jsonb_array_index
ON json_ste_vec_small_encrypted_10000 USING GIN (
    eql_v2.jsonb_array(value)
);

CREATE INDEX
json_ste_vec_small_encrypted_10000_stevec_query_index
ON json_ste_vec_small_encrypted_10000 USING GIN (
    (eql_v2.to_stevec_query(value)::jsonb) jsonb_path_ops
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `json_ste_vec_small_encrypted_v3_10000_stevec_query_index`
- 100,000: `json_ste_vec_small_encrypted_v3_100000_stevec_query_index`
- 1,000,000: `json_ste_vec_small_encrypted_v3_1000000_stevec_query_index`
- 10,000,000: `json_ste_vec_small_encrypted_v3_10000000_stevec_query_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 1 | 282.15μs | N/A |
| 100,000 | 1 | 349.10μs | N/A |
| 1,000,000 | 1 | 403.46μs | N/A |
| 10,000,000 | 1 | 3.36ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Bitmap Heap Scan on json_ste_vec_small_encrypted_v3_10000
    Bitmap Index Scan using json_ste_vec_small_encrypted_v3_10000_stevec_query_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 1,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_10000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 4,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v3.to_ste_vec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"75146c96a91ff82146e29e87b5accb95\", \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}, {\"s\": \"1c16090f62b45fc9a1404b2135beab94\", \"oc\": \"e108bb37379dfda0f0143886d759d3bd5c9a8987cb0e9011ce1f488a1f48a4f641be2e9370d97c316286536306cdf58b38d302d2c04ba18e62c6d0b1f370d48c6d\"}, {\"s\": \"25519d3ecf26f1d3833f11d9757135a7\", \"oc\": \"e2847ed5ad759eaaa77577c5c0fa51b44f31e10c15d77798a69fe3825352bd6762a261aad8b193a47ba71a1d603e9fcbab55dfa596625eb241a11c1393bd5d4250c9097324ecbf58f3f3f7867bf21b94855ea943b2e4013f957ad9daf21a6d96dd7b0c6f29d78a86ef39d073231279dcdf6a6b920a7ab5d0f5af704be57893974cae061f80d95ca6d5856b33528aa56b5a5ba1cdabe018f9271d34c314d1b51c4ac786e1af517e65edafac3413365b4b7cf0efd7fb6c528f1e8114e52eb45db3b2aaf7b1708281d8803bd9cef694fb1cc1ed210e19688fa40c3266156b3b348c1501c0422d939b65ced981ef0bcb4886b45588dd8190803c9c\"}, {\"s\": \"6b769ff71409e303be5ca17b08063375\", \"oc\": \"e2847ed445baee8293481c3a9a1c9c5f34a93a128a1e2aaf4544ca7d305698fbe02120b9d1ff474f3b16960df97da9e501\"}, {\"s\": \"b7cb88c99e692f82aa89ac83e6d52a83\", \"oc\": \"e2847ed446d572477e0e74ae817e5ce839b4e51c56bc0b63b3859e28be3ca296e831b8c63ba3527f53ca14823237644cf3eca46ba5a79b96d6679365690a738579019c42f78c24737e\"}]}'::jsonb)::eql_v3.jsonb_query)::jsonb)",
              "Index Name": "json_ste_vec_small_encrypted_v3_10000_stevec_query_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 91.11
            }
          ],
          "Recheck Cond": "((eql_v3.to_ste_vec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"75146c96a91ff82146e29e87b5accb95\", \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}, {\"s\": \"1c16090f62b45fc9a1404b2135beab94\", \"oc\": \"e108bb37379dfda0f0143886d759d3bd5c9a8987cb0e9011ce1f488a1f48a4f641be2e9370d97c316286536306cdf58b38d302d2c04ba18e62c6d0b1f370d48c6d\"}, {\"s\": \"25519d3ecf26f1d3833f11d9757135a7\", \"oc\": \"e2847ed5ad759eaaa77577c5c0fa51b44f31e10c15d77798a69fe3825352bd6762a261aad8b193a47ba71a1d603e9fcbab55dfa596625eb241a11c1393bd5d4250c9097324ecbf58f3f3f7867bf21b94855ea943b2e4013f957ad9daf21a6d96dd7b0c6f29d78a86ef39d073231279dcdf6a6b920a7ab5d0f5af704be57893974cae061f80d95ca6d5856b33528aa56b5a5ba1cdabe018f9271d34c314d1b51c4ac786e1af517e65edafac3413365b4b7cf0efd7fb6c528f1e8114e52eb45db3b2aaf7b1708281d8803bd9cef694fb1cc1ed210e19688fa40c3266156b3b348c1501c0422d939b65ced981ef0bcb4886b45588dd8190803c9c\"}, {\"s\": \"6b769ff71409e303be5ca17b08063375\", \"oc\": \"e2847ed445baee8293481c3a9a1c9c5f34a93a128a1e2aaf4544ca7d305698fbe02120b9d1ff474f3b16960df97da9e501\"}, {\"s\": \"b7cb88c99e692f82aa89ac83e6d52a83\", \"oc\": \"e2847ed446d572477e0e74ae817e5ce839b4e51c56bc0b63b3859e28be3ca296e831b8c63ba3527f53ca14823237644cf3eca46ba5a79b96d6679365690a738579019c42f78c24737e\"}]}'::jsonb)::eql_v3.jsonb_query)::jsonb)",
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000",
          "Startup Cost": 91.11,
          "Total Cost": 95.37
        }
      ],
      "Startup Cost": 91.11,
      "Total Cost": 95.37
    }
  }
]
```

**100,000 rows**

```
Limit
  Bitmap Heap Scan on json_ste_vec_small_encrypted_v3_100000
    Bitmap Index Scan using json_ste_vec_small_encrypted_v3_100000_stevec_query_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_100000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10,
          "Plan Width": 4,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v3.to_ste_vec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"75146c96a91ff82146e29e87b5accb95\", \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}, {\"s\": \"1c16090f62b45fc9a1404b2135beab94\", \"oc\": \"e108bb37379dfda0f0143886d8b88e74805615869eb52cc484d6ef162041d4365d87c84e72059b9b53c4c2629ef66302b65d5f85e28308250871262372c7ba7aa5\"}, {\"s\": \"25519d3ecf26f1d3833f11d9757135a7\", \"oc\": \"e2847ed5ad759eaaa77577c5c0fa52b77382067c7c1183503727aa23d6ffa890bb92e49d7d3adf618c5a583890ef4fae2e0b7cce737e038a48de21980035434b2a275b9bc3f570758ace9bf131a5e56af8c778990d8cb35d5d14d956a31b41ee78400ec677324d7bcacc26566d607b36e9005eff77426e28844c51316d15096e27b1a7c0aa4dbecfc25b4e12920eddd44892d9d44ece7557c58b014486fcf212c03eddaad4a6a0edc899d412271649bb3c52968262891299f02a36f98131aea7ba\"}, {\"s\": \"6b769ff71409e303be5ca17b08063375\", \"oc\": \"e2847ed446d573e24a6d6ea9d78c3684fe59bdab26cbbdc4761b5dbaa30bf2d9901ef91d341ce854d83deeb574f94d7839\"}, {\"s\": \"b7cb88c99e692f82aa89ac83e6d52a83\", \"oc\": \"e2847ed445baeddc1bf42210ea2f8eafed295040680b11e9c80553a0b15f3d53ab5fed93648dbd08ec19585b0a001c8b9c99fdc3cd5744ac8f\"}]}'::jsonb)::eql_v3.jsonb_query)::jsonb)",
              "Index Name": "json_ste_vec_small_encrypted_v3_100000_stevec_query_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 10,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 133.8
            }
          ],
          "Recheck Cond": "((eql_v3.to_ste_vec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"75146c96a91ff82146e29e87b5accb95\", \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}, {\"s\": \"1c16090f62b45fc9a1404b2135beab94\", \"oc\": \"e108bb37379dfda0f0143886d8b88e74805615869eb52cc484d6ef162041d4365d87c84e72059b9b53c4c2629ef66302b65d5f85e28308250871262372c7ba7aa5\"}, {\"s\": \"25519d3ecf26f1d3833f11d9757135a7\", \"oc\": \"e2847ed5ad759eaaa77577c5c0fa52b77382067c7c1183503727aa23d6ffa890bb92e49d7d3adf618c5a583890ef4fae2e0b7cce737e038a48de21980035434b2a275b9bc3f570758ace9bf131a5e56af8c778990d8cb35d5d14d956a31b41ee78400ec677324d7bcacc26566d607b36e9005eff77426e28844c51316d15096e27b1a7c0aa4dbecfc25b4e12920eddd44892d9d44ece7557c58b014486fcf212c03eddaad4a6a0edc899d412271649bb3c52968262891299f02a36f98131aea7ba\"}, {\"s\": \"6b769ff71409e303be5ca17b08063375\", \"oc\": \"e2847ed446d573e24a6d6ea9d78c3684fe59bdab26cbbdc4761b5dbaa30bf2d9901ef91d341ce854d83deeb574f94d7839\"}, {\"s\": \"b7cb88c99e692f82aa89ac83e6d52a83\", \"oc\": \"e2847ed445baeddc1bf42210ea2f8eafed295040680b11e9c80553a0b15f3d53ab5fed93648dbd08ec19585b0a001c8b9c99fdc3cd5744ac8f\"}]}'::jsonb)::eql_v3.jsonb_query)::jsonb)",
          "Relation Name": "json_ste_vec_small_encrypted_v3_100000",
          "Startup Cost": 133.81,
          "Total Cost": 175.79
        }
      ],
      "Startup Cost": 133.81,
      "Total Cost": 175.79
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Bitmap Heap Scan on json_ste_vec_small_encrypted_v3_1000000
    Bitmap Index Scan using json_ste_vec_small_encrypted_v3_1000000_stevec_query_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_1000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 100,
          "Plan Width": 4,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v3.to_ste_vec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"75146c96a91ff82146e29e87b5accb95\", \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}, {\"s\": \"1c16090f62b45fc9a1404b2135beab94\", \"oc\": \"e108bb37379dfda0f01437ae7ad5de46348e80c5fec7c22251d9e7e0d2f631b25954ca13569e9672f8fd88f1925d88fa1960bb20753765c5f8ece12e838977199b\"}, {\"s\": \"25519d3ecf26f1d3833f11d9757135a7\", \"oc\": \"e2847ed5ad746be56f3e4ff371f8f110ef5c06aff4613f38c8fd199cd91a358bc9d94e5528d939e550bf1bf509691053cff925b3af3acffcbf9b9166844f77554377de88902334af1923de75e21a719a46879cd185ebd6be206fe12d1e07910fe174acd4b5d31bca97a3e6143d3586e616ab1426a81f02451e6449a2fc81297284a7d3c992b5d4b335cdc978bcec670b61c8351f6864a962468e500e3934d6784125be8831df06c9539a1b5376d75156dc28170a1367136d2096791e4cf4ee3d949248e32f2949284d\"}, {\"s\": \"6b769ff71409e303be5ca17b08063375\", \"oc\": \"e2847ed446d572477e0e74ae828304897cec715195762d277519f49005eaf3dd8cfde5577edc959cd857a4a1c89d736645129ed3356babb654e6445edf9cbd46aa\"}, {\"s\": \"b7cb88c99e692f82aa89ac83e6d52a83\", \"oc\": \"e2847ed445b99a8145a992feffd27dbccc23403644ed2feffa7aa72ca2e80f44edb4e8923b1009b39a9864e18210175a386b760b80ea51b635\"}]}'::jsonb)::eql_v3.jsonb_query)::jsonb)",
              "Index Name": "json_ste_vec_small_encrypted_v3_1000000_stevec_query_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 100,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 176.9
            }
          ],
          "Recheck Cond": "((eql_v3.to_ste_vec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"75146c96a91ff82146e29e87b5accb95\", \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}, {\"s\": \"1c16090f62b45fc9a1404b2135beab94\", \"oc\": \"e108bb37379dfda0f01437ae7ad5de46348e80c5fec7c22251d9e7e0d2f631b25954ca13569e9672f8fd88f1925d88fa1960bb20753765c5f8ece12e838977199b\"}, {\"s\": \"25519d3ecf26f1d3833f11d9757135a7\", \"oc\": \"e2847ed5ad746be56f3e4ff371f8f110ef5c06aff4613f38c8fd199cd91a358bc9d94e5528d939e550bf1bf509691053cff925b3af3acffcbf9b9166844f77554377de88902334af1923de75e21a719a46879cd185ebd6be206fe12d1e07910fe174acd4b5d31bca97a3e6143d3586e616ab1426a81f02451e6449a2fc81297284a7d3c992b5d4b335cdc978bcec670b61c8351f6864a962468e500e3934d6784125be8831df06c9539a1b5376d75156dc28170a1367136d2096791e4cf4ee3d949248e32f2949284d\"}, {\"s\": \"6b769ff71409e303be5ca17b08063375\", \"oc\": \"e2847ed446d572477e0e74ae828304897cec715195762d277519f49005eaf3dd8cfde5577edc959cd857a4a1c89d736645129ed3356babb654e6445edf9cbd46aa\"}, {\"s\": \"b7cb88c99e692f82aa89ac83e6d52a83\", \"oc\": \"e2847ed445b99a8145a992feffd27dbccc23403644ed2feffa7aa72ca2e80f44edb4e8923b1009b39a9864e18210175a386b760b80ea51b635\"}]}'::jsonb)::eql_v3.jsonb_query)::jsonb)",
          "Relation Name": "json_ste_vec_small_encrypted_v3_1000000",
          "Startup Cost": 176.93,
          "Total Cost": 596.75
        }
      ],
      "Startup Cost": 176.93,
      "Total Cost": 218.91
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Bitmap Heap Scan on json_ste_vec_small_encrypted_v3_10000000
    Bitmap Index Scan using json_ste_vec_small_encrypted_v3_10000000_stevec_query_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_10000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1001,
          "Plan Width": 4,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v3.to_ste_vec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"75146c96a91ff82146e29e87b5accb95\", \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}, {\"s\": \"1c16090f62b45fc9a1404b2135beab94\", \"oc\": \"e108bb37379dfda0f01437ae7ad5de47c4dda61612bc0a735b93a3dc54bb05ddfebc4962d4c984833bbae7114dce857343c47f70b4ea5eef763e42ad805e093100\"}, {\"s\": \"25519d3ecf26f1d3833f11d9757135a7\", \"oc\": \"e2847ed5ad746be56e47d625e878b99907019ac971f28e701d66bac220149b4e32acd1409cfd840f134e8b932942a28f75e256abe09139f15e8db9819244b866f1ecb897484dc6d614df009fbec0420fc3bb000ecfe2aaccded2bb38b532affa27ec0b06605c7e6f1e78d7db46d5930bac2f82046dd25f92bbe733f35df5fa157ad6798c4864e465919c3d514a1d1947caa38eed0a237056ba1ffd623bc679cba5b273433fac3762f4336ffe1e647c7af57d3890f7c2851f9c241125d55d5d1ddaa4209c4ee1e4f3479b46fcda5618dcc07d5d1ae1151c4d8d8b04954320066a8068d8bc4d60404c2d16ae2630909ccba1\"}, {\"s\": \"6b769ff71409e303be5ca17b08063375\", \"oc\": \"e2847ed445baee82925a7a5a663f9f9a3df582bd3359bc1b1701c8ceca2b781514316b6e0b3671d0c92d60cdc55ec016844e6ef9d3a2163ea5\"}, {\"s\": \"b7cb88c99e692f82aa89ac83e6d52a83\", \"oc\": \"e2847ed446d573e338ef0e5f508beb1aa6874da1e366adf7c62290e02e4d86a35fe932591e8e3a5e2b198e5ee74f1617c6f5178e7c6113335597e6c6665b4335fe74ba61dfb68b2524b80e337333506c16\"}]}'::jsonb)::eql_v3.jsonb_query)::jsonb)",
              "Index Name": "json_ste_vec_small_encrypted_v3_10000000_stevec_query_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1001,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 181.46
            }
          ],
          "Recheck Cond": "((eql_v3.to_ste_vec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"75146c96a91ff82146e29e87b5accb95\", \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}, {\"s\": \"1c16090f62b45fc9a1404b2135beab94\", \"oc\": \"e108bb37379dfda0f01437ae7ad5de47c4dda61612bc0a735b93a3dc54bb05ddfebc4962d4c984833bbae7114dce857343c47f70b4ea5eef763e42ad805e093100\"}, {\"s\": \"25519d3ecf26f1d3833f11d9757135a7\", \"oc\": \"e2847ed5ad746be56e47d625e878b99907019ac971f28e701d66bac220149b4e32acd1409cfd840f134e8b932942a28f75e256abe09139f15e8db9819244b866f1ecb897484dc6d614df009fbec0420fc3bb000ecfe2aaccded2bb38b532affa27ec0b06605c7e6f1e78d7db46d5930bac2f82046dd25f92bbe733f35df5fa157ad6798c4864e465919c3d514a1d1947caa38eed0a237056ba1ffd623bc679cba5b273433fac3762f4336ffe1e647c7af57d3890f7c2851f9c241125d55d5d1ddaa4209c4ee1e4f3479b46fcda5618dcc07d5d1ae1151c4d8d8b04954320066a8068d8bc4d60404c2d16ae2630909ccba1\"}, {\"s\": \"6b769ff71409e303be5ca17b08063375\", \"oc\": \"e2847ed445baee82925a7a5a663f9f9a3df582bd3359bc1b1701c8ceca2b781514316b6e0b3671d0c92d60cdc55ec016844e6ef9d3a2163ea5\"}, {\"s\": \"b7cb88c99e692f82aa89ac83e6d52a83\", \"oc\": \"e2847ed446d573e338ef0e5f508beb1aa6874da1e366adf7c62290e02e4d86a35fe932591e8e3a5e2b198e5ee74f1617c6f5178e7c6113335597e6c6665b4335fe74ba61dfb68b2524b80e337333506c16\"}]}'::jsonb)::eql_v3.jsonb_query)::jsonb)",
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000000",
          "Startup Cost": 181.71,
          "Total Cost": 4382.1
        }
      ],
      "Startup Cost": 181.71,
      "Total Cost": 223.67
    }
  }
]
```

</details>

![Query Performance - JSON/contains/functional](query_json_contains_functional_chart.png)

## field_eq/bare

**Description:** Field-level equality via `value -> 'sel' = $1::eql_v2_encrypted` (no index)

**SQL Query:**
```sql
SELECT id FROM {TABLE} WHERE (value -> '<selector-hash>'::text) = $1::jsonb::eql_v2_encrypted LIMIT 10
```

**Parameter:** `<sampled-sv-element-as-jsonb>`

**Table: `json_ste_vec_small_encrypted_{rows}`. `eql_v2."->"` is plpgsql (not inlinable), so the planner cannot match any functional index against the LHS — forces Seq Scan + per-row sv walk. This is the natural form a JS/ORM caller would write; the bench includes it to show the cost of *not* having an inlinable extractor on `->`.**

**Indexes available on the table:**
```sql
-- EQL 2.3 functional GIN indexes for the json ste_vec bench.
--
-- jsonb_array  — whole-document containment (contains/functional):
--                eql_v2.jsonb_array(value) @> eql_v2.jsonb_array($1).
-- stevec_query — typed field-level containment (field_eq/extractor):
--                value @> $1::eql_v2.stevec_query inlines to a native
--                jsonb @> over eql_v2.to_stevec_query(value)::jsonb.
--                XOR-aware: one index covers hm- and oc-bearing selectors.
--
-- Replaces the pre-2.3 eql_v2.ste_vec / eql_v2.hmac_256_terms GIN indexes
-- (hmac_256_terms was removed in EQL 2.3 — see cipherstash/eql#223).

CREATE INDEX
json_ste_vec_small_encrypted_10000_jsonb_array_index
ON json_ste_vec_small_encrypted_10000 USING GIN (
    eql_v2.jsonb_array(value)
);

CREATE INDEX
json_ste_vec_small_encrypted_10000_stevec_query_index
ON json_ste_vec_small_encrypted_10000 USING GIN (
    (eql_v2.to_stevec_query(value)::jsonb) jsonb_path_ops
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `json_ste_vec_small_encrypted_v3_10000_field_eq_idx`
- 100,000: `json_ste_vec_small_encrypted_v3_100000_field_eq_idx`
- 1,000,000: `json_ste_vec_small_encrypted_v3_1000000_field_eq_idx`
- 10,000,000: `json_ste_vec_small_encrypted_v3_10000000_field_eq_idx`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 114.67μs | N/A |
| 100,000 | 10 | 114.37μs | N/A |
| 1,000,000 | 10 | 117.68μs | N/A |
| 10,000,000 | 10 | 110.36μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_v3_10000_field_eq_idx on json_ste_vec_small_encrypted_v3_10000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_10000",
          "Async Capable": false,
          "Index Cond": "(eql_v3.eq_term((value -> '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbKvscXsWK)o!=#~jnHht4j<aWKyNLI?+QW)C7G?<XlfQHRah(ogZIFQBpQ-f<V=y^Mwgunr<BSa{jn$K~2YEYvp!g!a*yo2CAfn(>8%o`tXorx$mQNTenJF@JOviE;0q!uRzj7**vItTAx5h0C0fHflj2R7kCxN)GLcDv-n=J6x@J*=uU+qXW8o?(?MHlzkWzBxNsZ02dTwHiA~Fpuh\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_10000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::eql_v3.jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_10000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 2250.54
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 2.79
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_v3_100000_field_eq_idx on json_ste_vec_small_encrypted_v3_100000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_100000",
          "Async Capable": false,
          "Index Cond": "(eql_v3.eq_term((value -> '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbKF|4gse#%Y{Z@!BO^s55=UXrxZPWh{wh@ZYQEF}!cgT1z&>SwO|13o==OSB$RM4QzL_iN$h^QNJmX4ej52pk$E2=2F5D2RpAZ{@+r*oSJ50$0(wP^^x(nBxu$KY8l9%z=IzYxRmc<R8FQgC4UF|H>kT+7{nkk)<zjP*{o&8avr$Etr$xYds3alIA+EZH^{yqLGu)#zy\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_100000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::eql_v3.jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_100000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 100000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 22289.54
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 2.77
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_v3_1000000_field_eq_idx on json_ste_vec_small_encrypted_v3_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_1000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3.eq_term((value -> '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbLHfh4R!57VeyiH{p`_nNK5Yq^hBi+_D1CnxcMnz@trQ9FE8>CMem^_9Ed%F?|^KHy;*5#8m3?N91KFeJ0@NFj-!SnYv&REzD@qyegdh{CB_WB3FT7qNeHZPXVH{Bq1s+bqz==51VdJcWyr&6rSb5>kekQ>$vT#2_#THV@b)2s4;}q+4w_f{&NE>Ltt>e8`j_#KNRsGH;;3\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_1000000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::eql_v3.jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_1000000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 998723,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.68,
          "Total Cost": 222802.33
        }
      ],
      "Startup Cost": 0.68,
      "Total Cost": 2.91
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_v3_10000000_field_eq_idx on json_ste_vec_small_encrypted_v3_10000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_10000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3.eq_term((value -> '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbK#JWHCb!&3R+nlpj;bE=2La>dMcnX0kPYPDTv1ZWJdz~a6RaqmB81SgpV6=@s=9W3#>*V@2A$adw|{cXA<nmpH0moIvTV?Q9(jCFWrum1%e1~K1NkVIV^5XJqSTc(p1O{sZ^CIzc)+)s3JtB{9O=PmsKi2B^M^GdA+blYFVAg%D<$%2`mZ#TP}h1}V}^PLAoJ4{qktEWKY7@`<3vY@~\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_10000000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::eql_v3.jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_10000000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10008218,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.69,
          "Total Cost": 2113149.5
        }
      ],
      "Startup Cost": 0.69,
      "Total Cost": 2.8
    }
  }
]
```

</details>

![Query Performance - JSON/field_eq/bare](query_json_field_eq_bare_chart.png)

## field_eq/extractor

**Description:** Field-level equality via `hmac_256_terms @> [{s,hm}]` (functional GIN)

**SQL Query:**
```sql
SELECT id FROM {TABLE} WHERE eql_v2.hmac_256_terms(value) @> $1::jsonb LIMIT 10
```

**Parameter:** `[{"s":"<selector-hash>","hm":"<hmac>"}]`

**Table: `json_ste_vec_small_encrypted_{rows}`. Index: functional GIN on `eql_v2.hmac_256_terms(value)`. One index covers field-level equality across every selector that carries `hm`, vs the per-selector recipe below. The bench picks a (selector, hmac) pair from `sv[0]` of a sample row at startup; needle is `[{"s":"<sel>","hm":"<hash>"}]`.**

**Indexes available on the table:**
```sql
-- EQL 2.3 functional GIN indexes for the json ste_vec bench.
--
-- jsonb_array  — whole-document containment (contains/functional):
--                eql_v2.jsonb_array(value) @> eql_v2.jsonb_array($1).
-- stevec_query — typed field-level containment (field_eq/extractor):
--                value @> $1::eql_v2.stevec_query inlines to a native
--                jsonb @> over eql_v2.to_stevec_query(value)::jsonb.
--                XOR-aware: one index covers hm- and oc-bearing selectors.
--
-- Replaces the pre-2.3 eql_v2.ste_vec / eql_v2.hmac_256_terms GIN indexes
-- (hmac_256_terms was removed in EQL 2.3 — see cipherstash/eql#223).

CREATE INDEX
json_ste_vec_small_encrypted_10000_jsonb_array_index
ON json_ste_vec_small_encrypted_10000 USING GIN (
    eql_v2.jsonb_array(value)
);

CREATE INDEX
json_ste_vec_small_encrypted_10000_stevec_query_index
ON json_ste_vec_small_encrypted_10000 USING GIN (
    (eql_v2.to_stevec_query(value)::jsonb) jsonb_path_ops
);
```

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 1.26ms | N/A |
| 100,000 | 10 | 7.87ms | N/A |
| 1,000,000 | 10 | 63.99ms | N/A |
| 10,000,000 | 10 | 609.06μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Seq Scan on json_ste_vec_small_encrypted_v3_10000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_10000",
          "Async Capable": false,
          "Filter": "((eql_v3.to_ste_vec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"75146c96a91ff82146e29e87b5accb95\", \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}]}'::jsonb)::eql_v3.jsonb_query)::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 9999,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000",
          "Startup Cost": 0.0,
          "Total Cost": 4678.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 4.68
    }
  }
]
```

**100,000 rows**

```
Limit
  Seq Scan on json_ste_vec_small_encrypted_v3_100000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_100000",
          "Async Capable": false,
          "Filter": "((eql_v3.to_ste_vec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"75146c96a91ff82146e29e87b5accb95\", \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}]}'::jsonb)::eql_v3.jsonb_query)::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 99990,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_100000",
          "Startup Cost": 0.0,
          "Total Cost": 46680.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 4.67
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Seq Scan on json_ste_vec_small_encrypted_v3_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_1000000",
          "Async Capable": false,
          "Filter": "((eql_v3.to_ste_vec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"75146c96a91ff82146e29e87b5accb95\", \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}]}'::jsonb)::eql_v3.jsonb_query)::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 998623,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 466498.6
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 4.67
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Seq Scan on json_ste_vec_small_encrypted_v3_10000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_10000000",
          "Async Capable": false,
          "Filter": "((eql_v3.to_ste_vec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"75146c96a91ff82146e29e87b5accb95\", \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}]}'::jsonb)::eql_v3.jsonb_query)::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10007217,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 4555447.77
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 4.55
    }
  }
]
```

</details>

![Query Performance - JSON/field_eq/extractor](query_json_field_eq_extractor_chart.png)

## field_eq/functional

**Description:** Field-level equality via per-selector `hmac_256(col, 'sel')`

**SQL Query:**
```sql
SELECT id FROM {TABLE} WHERE eql_v2.hmac_256(value, '<selector-hash>') = eql_v2.hmac_256($1::eql_v2_encrypted) LIMIT 10
```

**Parameter:** `<sampled-sv-element-as-eql_v2_encrypted>`

**Table: `json_ste_vec_small_encrypted_{rows}`. Would engage `hash (eql_v2.hmac_256(col, '<sel>'))` if one existed; benches/main only creates the `hmac_256_terms` GIN (one index for all selectors), so this scenario serves as a baseline showing the cost of the per-selector recipe without a matching index.**

**Indexes available on the table:**
```sql
-- EQL 2.3 functional GIN indexes for the json ste_vec bench.
--
-- jsonb_array  — whole-document containment (contains/functional):
--                eql_v2.jsonb_array(value) @> eql_v2.jsonb_array($1).
-- stevec_query — typed field-level containment (field_eq/extractor):
--                value @> $1::eql_v2.stevec_query inlines to a native
--                jsonb @> over eql_v2.to_stevec_query(value)::jsonb.
--                XOR-aware: one index covers hm- and oc-bearing selectors.
--
-- Replaces the pre-2.3 eql_v2.ste_vec / eql_v2.hmac_256_terms GIN indexes
-- (hmac_256_terms was removed in EQL 2.3 — see cipherstash/eql#223).

CREATE INDEX
json_ste_vec_small_encrypted_10000_jsonb_array_index
ON json_ste_vec_small_encrypted_10000 USING GIN (
    eql_v2.jsonb_array(value)
);

CREATE INDEX
json_ste_vec_small_encrypted_10000_stevec_query_index
ON json_ste_vec_small_encrypted_10000 USING GIN (
    (eql_v2.to_stevec_query(value)::jsonb) jsonb_path_ops
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `json_ste_vec_small_encrypted_v3_10000_field_eq_idx`
- 100,000: `json_ste_vec_small_encrypted_v3_100000_field_eq_idx`
- 1,000,000: `json_ste_vec_small_encrypted_v3_1000000_field_eq_idx`
- 10,000,000: `json_ste_vec_small_encrypted_v3_10000000_field_eq_idx`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 105.86μs | N/A |
| 100,000 | 10 | 113.05μs | N/A |
| 1,000,000 | 10 | 115.84μs | N/A |
| 10,000,000 | 10 | 110.36μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_v3_10000_field_eq_idx on json_ste_vec_small_encrypted_v3_10000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_10000",
          "Async Capable": false,
          "Index Cond": "(eql_v3.eq_term((value -> '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbKvscXsWK)o!=#~jnHht4j<aWKyNLI?+QW)C7G?<XlfQHRah(ogZIFQBpQ-f<V=y^Mwgunr<BSa{jn$K~2YEYvp!g!a*yo2CAfn(>8%o`tXorx$mQNTenJF@JOviE;0q!uRzj7**vItTAx5h0C0fHflj2R7kCxN)GLcDv-n=J6x@J*=uU+qXW8o?(?MHlzkWzBxNsZ02dTwHiA~Fpuh\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_10000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::eql_v3.jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_10000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 2250.54
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 2.79
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_v3_100000_field_eq_idx on json_ste_vec_small_encrypted_v3_100000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_100000",
          "Async Capable": false,
          "Index Cond": "(eql_v3.eq_term((value -> '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbKF|4gse#%Y{Z@!BO^s55=UXrxZPWh{wh@ZYQEF}!cgT1z&>SwO|13o==OSB$RM4QzL_iN$h^QNJmX4ej52pk$E2=2F5D2RpAZ{@+r*oSJ50$0(wP^^x(nBxu$KY8l9%z=IzYxRmc<R8FQgC4UF|H>kT+7{nkk)<zjP*{o&8avr$Etr$xYds3alIA+EZH^{yqLGu)#zy\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_100000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::eql_v3.jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_100000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 100000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 22289.54
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 2.77
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_v3_1000000_field_eq_idx on json_ste_vec_small_encrypted_v3_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_1000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3.eq_term((value -> '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbLHfh4R!57VeyiH{p`_nNK5Yq^hBi+_D1CnxcMnz@trQ9FE8>CMem^_9Ed%F?|^KHy;*5#8m3?N91KFeJ0@NFj-!SnYv&REzD@qyegdh{CB_WB3FT7qNeHZPXVH{Bq1s+bqz==51VdJcWyr&6rSb5>kekQ>$vT#2_#THV@b)2s4;}q+4w_f{&NE>Ltt>e8`j_#KNRsGH;;3\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_1000000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::eql_v3.jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_1000000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 998723,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.68,
          "Total Cost": 222802.33
        }
      ],
      "Startup Cost": 0.68,
      "Total Cost": 2.91
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_v3_10000000_field_eq_idx on json_ste_vec_small_encrypted_v3_10000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_10000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3.eq_term((value -> '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbK#JWHCb!&3R+nlpj;bE=2La>dMcnX0kPYPDTv1ZWJdz~a6RaqmB81SgpV6=@s=9W3#>*V@2A$adw|{cXA<nmpH0moIvTV?Q9(jCFWrum1%e1~K1NkVIV^5XJqSTc(p1O{sZ^CIzc)+)s3JtB{9O=PmsKi2B^M^GdA+blYFVAg%D<$%2`mZ#TP}h1}V}^PLAoJ4{qktEWKY7@`<3vY@~\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_10000000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::eql_v3.jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_10000000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10008218,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.69,
          "Total Cost": 2113149.5
        }
      ],
      "Startup Cost": 0.69,
      "Total Cost": 2.8
    }
  }
]
```

</details>

![Query Performance - JSON/field_eq/functional](query_json_field_eq_functional_chart.png)

## field_order/functional

**Description:** Field-level ORDER BY via ORE extractor on `value -> 'sel'`

**SQL Query:**
```sql
SELECT id FROM {TABLE} ORDER BY <ore_extractor>(value -> '<selector-hash>'::text) LIMIT 10
```

**Table: `json_ste_vec_small_encrypted_{rows}`. Index: functional btree on `<ore_extractor>(value -> '<selector>'::text)` using the appropriate opclass for the term type. `<ore_extractor>` is selected at bench startup based on which orderable tag the sampled sv element carries:
  - `oc` → `eql_v2.ore_cllw` (Standard mode, ORE CLLW — requires the `eql_v2.ore_cllw_ops` btree opclass from EQL #221)
  - `op` → `eql_v2.ope_cllw` (Compat mode, OPE CLLW)
  - `ob` → `eql_v2.ore_block_u64_8_256` (Block ORE — root scalars only)
When the table's `oc` index is present, the plan engages Index Scan + LIMIT (no Sort node). When absent (older bench run / index not yet rebuilt), falls back to Seq Scan + Top-N sort.**

**Indexes available on the table:**
```sql
-- EQL 2.3 functional GIN indexes for the json ste_vec bench.
--
-- jsonb_array  — whole-document containment (contains/functional):
--                eql_v2.jsonb_array(value) @> eql_v2.jsonb_array($1).
-- stevec_query — typed field-level containment (field_eq/extractor):
--                value @> $1::eql_v2.stevec_query inlines to a native
--                jsonb @> over eql_v2.to_stevec_query(value)::jsonb.
--                XOR-aware: one index covers hm- and oc-bearing selectors.
--
-- Replaces the pre-2.3 eql_v2.ste_vec / eql_v2.hmac_256_terms GIN indexes
-- (hmac_256_terms was removed in EQL 2.3 — see cipherstash/eql#223).

CREATE INDEX
json_ste_vec_small_encrypted_10000_jsonb_array_index
ON json_ste_vec_small_encrypted_10000 USING GIN (
    eql_v2.jsonb_array(value)
);

CREATE INDEX
json_ste_vec_small_encrypted_10000_stevec_query_index
ON json_ste_vec_small_encrypted_10000 USING GIN (
    (eql_v2.to_stevec_query(value)::jsonb) jsonb_path_ops
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `json_ste_vec_small_encrypted_v3_10000_field_order_idx`
- 100,000: `json_ste_vec_small_encrypted_v3_100000_field_order_idx`
- 1,000,000: `json_ste_vec_small_encrypted_v3_1000000_field_order_idx`
- 10,000,000: `json_ste_vec_small_encrypted_v3_10000000_field_order_idx`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 266.06μs | N/A |
| 100,000 | 10 | 295.96μs | N/A |
| 1,000,000 | 10 | 258.60μs | N/A |
| 10,000,000 | 10 | 251.19μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_v3_10000_field_order_idx on json_ste_vec_small_encrypted_v3_10000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_10000",
          "Async Capable": false,
          "Index Name": "json_ste_vec_small_encrypted_v3_10000_field_order_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10000,
          "Plan Width": 36,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.41,
          "Total Cost": 13830.12
        }
      ],
      "Startup Cost": 0.41,
      "Total Cost": 14.24
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_v3_100000_field_order_idx on json_ste_vec_small_encrypted_v3_100000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_100000",
          "Async Capable": false,
          "Index Name": "json_ste_vec_small_encrypted_v3_100000_field_order_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 100000,
          "Plan Width": 36,
          "Relation Name": "json_ste_vec_small_encrypted_v3_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.42,
          "Total Cost": 137793.58
        }
      ],
      "Startup Cost": 0.42,
      "Total Cost": 14.2
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_v3_1000000_field_order_idx on json_ste_vec_small_encrypted_v3_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_1000000",
          "Async Capable": false,
          "Index Name": "json_ste_vec_small_encrypted_v3_1000000_field_order_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 998723,
          "Plan Width": 36,
          "Relation Name": "json_ste_vec_small_encrypted_v3_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.55,
          "Total Cost": 1377543.19
        }
      ],
      "Startup Cost": 0.55,
      "Total Cost": 14.34
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_v3_10000000_field_order_idx on json_ste_vec_small_encrypted_v3_10000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_10000000",
          "Async Capable": false,
          "Index Name": "json_ste_vec_small_encrypted_v3_10000000_field_order_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10008218,
          "Plan Width": 36,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.56,
          "Total Cost": 13325379.78
        }
      ],
      "Startup Cost": 0.56,
      "Total Cost": 13.87
    }
  }
]
```

</details>

![Query Performance - JSON/field_order/functional](query_json_field_order_functional_chart.png)

