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

- 10,000: `json_ste_vec_small_encrypted_v3_10000_jsonb_array_index`
- 100,000: `json_ste_vec_small_encrypted_v3_100000_jsonb_array_index`
- 1,000,000: `json_ste_vec_small_encrypted_v3_1000000_jsonb_array_index`
- 10,000,000: `json_ste_vec_small_encrypted_v3_10000000_jsonb_array_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 1 | 233.88μs | N/A |
| 100,000 | 1 | 256.09μs | N/A |
| 1,000,000 | 1 | 419.36μs | N/A |
| 10,000,000 | 1 | 1.06ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Bitmap Heap Scan on json_ste_vec_small_encrypted_v3_10000
    Bitmap Index Scan using json_ste_vec_small_encrypted_v3_10000_jsonb_array_index
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
              "Index Cond": "(eql_v3.jsonb_array((value)::jsonb) @> '{\"{\\\"s\\\": \\\"75146c96a91ff82146e29e87b5accb95\\\", \\\"hm\\\": \\\"98b7664512c8c0125772e43e47dde0ce\\\"}\",\"{\\\"s\\\": \\\"b7cb88c99e692f82aa89ac83e6d52a83\\\", \\\"oc\\\": \\\"e2847ed445b99a8145a992feffd27dbccc23403644ed2feffa7aa72ca2e78d163b006e9765c25c979709adfac3eb53f92dcfcff868232545184a5122622c6d2e57\\\"}\",\"{\\\"s\\\": \\\"25519d3ecf26f1d3833f11d9757135a7\\\", \\\"oc\\\": \\\"e2847ed5ae112c2089404a378b7e03856b6281edd02159fedc2e7e61244c43b14ce9d67921f590c328d1587160f9af113ae95aaf453a37514c268eb4317678bfe4365cfa8f689edce9796f5ec194f8e6a48882b07607ae6e03c16a6a3d985d5edbe516bbca6094d9b73343a57de5f0311c4e25df6eaf7c094fe5d1b97ea3696a3043f38c68c7560242188228687fbb7880bbe891b3aaa83029995d9553b71bc27d8de087dfd65eb12f1a976887c322c729\\\"}\",\"{\\\"s\\\": \\\"6b769ff71409e303be5ca17b08063375\\\", \\\"oc\\\": \\\"e2847ed445b99a8145a992fefe400f7a9a1f0253ffa837b76c1602d4a28ae81114\\\"}\",\"{\\\"s\\\": \\\"1c16090f62b45fc9a1404b2135beab94\\\", \\\"oc\\\": \\\"e108bb37379dfda0f0143886d8b88e748150ce18ba0d1fa98dda6bf96cb47d12150661d4710365e26993420af2ccd1d836df55ccbc9bb7a6fb4f5589c09032c4bf\\\"}\"}'::jsonb[])",
              "Index Name": "json_ste_vec_small_encrypted_v3_10000_jsonb_array_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 68.86
            }
          ],
          "Recheck Cond": "(eql_v3.jsonb_array((value)::jsonb) @> '{\"{\\\"s\\\": \\\"75146c96a91ff82146e29e87b5accb95\\\", \\\"hm\\\": \\\"98b7664512c8c0125772e43e47dde0ce\\\"}\",\"{\\\"s\\\": \\\"b7cb88c99e692f82aa89ac83e6d52a83\\\", \\\"oc\\\": \\\"e2847ed445b99a8145a992feffd27dbccc23403644ed2feffa7aa72ca2e78d163b006e9765c25c979709adfac3eb53f92dcfcff868232545184a5122622c6d2e57\\\"}\",\"{\\\"s\\\": \\\"25519d3ecf26f1d3833f11d9757135a7\\\", \\\"oc\\\": \\\"e2847ed5ae112c2089404a378b7e03856b6281edd02159fedc2e7e61244c43b14ce9d67921f590c328d1587160f9af113ae95aaf453a37514c268eb4317678bfe4365cfa8f689edce9796f5ec194f8e6a48882b07607ae6e03c16a6a3d985d5edbe516bbca6094d9b73343a57de5f0311c4e25df6eaf7c094fe5d1b97ea3696a3043f38c68c7560242188228687fbb7880bbe891b3aaa83029995d9553b71bc27d8de087dfd65eb12f1a976887c322c729\\\"}\",\"{\\\"s\\\": \\\"6b769ff71409e303be5ca17b08063375\\\", \\\"oc\\\": \\\"e2847ed445b99a8145a992fefe400f7a9a1f0253ffa837b76c1602d4a28ae81114\\\"}\",\"{\\\"s\\\": \\\"1c16090f62b45fc9a1404b2135beab94\\\", \\\"oc\\\": \\\"e108bb37379dfda0f0143886d8b88e748150ce18ba0d1fa98dda6bf96cb47d12150661d4710365e26993420af2ccd1d836df55ccbc9bb7a6fb4f5589c09032c4bf\\\"}\"}'::jsonb[])",
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000",
          "Startup Cost": 68.86,
          "Total Cost": 73.13
        }
      ],
      "Startup Cost": 68.86,
      "Total Cost": 73.13
    }
  }
]
```

**100,000 rows**

```
Limit
  Bitmap Heap Scan on json_ste_vec_small_encrypted_v3_100000
    Bitmap Index Scan using json_ste_vec_small_encrypted_v3_100000_jsonb_array_index
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
          "Alias": "json_ste_vec_small_encrypted_v3_100000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 4,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "(eql_v3.jsonb_array((value)::jsonb) @> '{\"{\\\"s\\\": \\\"6b769ff71409e303be5ca17b08063375\\\", \\\"oc\\\": \\\"e2847ed445baee82925a7a5a663fa0a488c796509325fbe46fee6db4348e604e9fca13c9c6dab8f19e6380c88111eca6e1024946c57cd2189644e055f6e3da741d\\\"}\",\"{\\\"s\\\": \\\"75146c96a91ff82146e29e87b5accb95\\\", \\\"hm\\\": \\\"98b7664512c8c0125772e43e47dde0ce\\\"}\",\"{\\\"s\\\": \\\"b7cb88c99e692f82aa89ac83e6d52a83\\\", \\\"oc\\\": \\\"e2847ed445baeddd580ed0f4e600b1a1b0a795e861d0738c0807142caf160adc5367814641383016a79c782539e7c3c74094a26998670debf9\\\"}\",\"{\\\"s\\\": \\\"1c16090f62b45fc9a1404b2135beab94\\\", \\\"oc\\\": \\\"e108bb37379dfda0f0143886d759d250a567842a7c3f5af2a6b3f2a58a8160fc3e4079dc9e54b19e671a8834488db7592a060356887c3e718a86beaefae5648294\\\"}\",\"{\\\"s\\\": \\\"25519d3ecf26f1d3833f11d9757135a7\\\", \\\"oc\\\": \\\"e2847ed5ad759eaaa77577c5c0fa51b44f31e10c15d77798a69fe38252a01a79d62483c88ca828f2f55b01f2ce53421931e02a7425637d9aab24caf3f8184fa93b84fb2a933202bd7b27a38fa1d6eaeddeee178e882b82275ca68f35401ef1ced3167a7e0df4852a35c4042b081517fc9e45eb66b68285f26d619787ae9a2873ec2f0bee0f17ff420f38d30c9366a12709cb076e0da9aae5109dcb3d2f23105c3fba4c802a8d948bac8f609f84f2f73702fa5a4eeb1362738de9e4bd2b2e91b1a83867e0bba8c1b56b6d5b1f0b274539a841071fc02399a4d7\\\"}\"}'::jsonb[])",
              "Index Name": "json_ste_vec_small_encrypted_v3_100000_jsonb_array_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 90.19
            }
          ],
          "Recheck Cond": "(eql_v3.jsonb_array((value)::jsonb) @> '{\"{\\\"s\\\": \\\"6b769ff71409e303be5ca17b08063375\\\", \\\"oc\\\": \\\"e2847ed445baee82925a7a5a663fa0a488c796509325fbe46fee6db4348e604e9fca13c9c6dab8f19e6380c88111eca6e1024946c57cd2189644e055f6e3da741d\\\"}\",\"{\\\"s\\\": \\\"75146c96a91ff82146e29e87b5accb95\\\", \\\"hm\\\": \\\"98b7664512c8c0125772e43e47dde0ce\\\"}\",\"{\\\"s\\\": \\\"b7cb88c99e692f82aa89ac83e6d52a83\\\", \\\"oc\\\": \\\"e2847ed445baeddd580ed0f4e600b1a1b0a795e861d0738c0807142caf160adc5367814641383016a79c782539e7c3c74094a26998670debf9\\\"}\",\"{\\\"s\\\": \\\"1c16090f62b45fc9a1404b2135beab94\\\", \\\"oc\\\": \\\"e108bb37379dfda0f0143886d759d250a567842a7c3f5af2a6b3f2a58a8160fc3e4079dc9e54b19e671a8834488db7592a060356887c3e718a86beaefae5648294\\\"}\",\"{\\\"s\\\": \\\"25519d3ecf26f1d3833f11d9757135a7\\\", \\\"oc\\\": \\\"e2847ed5ad759eaaa77577c5c0fa51b44f31e10c15d77798a69fe38252a01a79d62483c88ca828f2f55b01f2ce53421931e02a7425637d9aab24caf3f8184fa93b84fb2a933202bd7b27a38fa1d6eaeddeee178e882b82275ca68f35401ef1ced3167a7e0df4852a35c4042b081517fc9e45eb66b68285f26d619787ae9a2873ec2f0bee0f17ff420f38d30c9366a12709cb076e0da9aae5109dcb3d2f23105c3fba4c802a8d948bac8f609f84f2f73702fa5a4eeb1362738de9e4bd2b2e91b1a83867e0bba8c1b56b6d5b1f0b274539a841071fc02399a4d7\\\"}\"}'::jsonb[])",
          "Relation Name": "json_ste_vec_small_encrypted_v3_100000",
          "Startup Cost": 90.19,
          "Total Cost": 94.45
        }
      ],
      "Startup Cost": 90.19,
      "Total Cost": 94.45
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Bitmap Heap Scan on json_ste_vec_small_encrypted_v3_1000000
    Bitmap Index Scan using json_ste_vec_small_encrypted_v3_1000000_jsonb_array_index
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
          "Alias": "json_ste_vec_small_encrypted_v3_1000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 4,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "(eql_v3.jsonb_array((value)::jsonb) @> '{\"{\\\"s\\\": \\\"6b769ff71409e303be5ca17b08063375\\\", \\\"oc\\\": \\\"e2847ed446d572477e0e74ae828304897b9f568962af078cdd5e9f431c2c05d49d78229e2d75115eee93156538ce6055ce65b627f446147bd3\\\"}\",\"{\\\"s\\\": \\\"75146c96a91ff82146e29e87b5accb95\\\", \\\"hm\\\": \\\"98b7664512c8c0125772e43e47dde0ce\\\"}\",\"{\\\"s\\\": \\\"b7cb88c99e692f82aa89ac83e6d52a83\\\", \\\"oc\\\": \\\"e2847ed445b99b00a005cc452ce307db927ed0c8b81d13e19cbb4c97c25a98a345ce77a106ed1accffff18f5c5a8d521f49164c2570fe0e15944dabc246a97232eed758beac7af182c7c455eb7763c5421a9eec34836c60124\\\"}\",\"{\\\"s\\\": \\\"25519d3ecf26f1d3833f11d9757135a7\\\", \\\"oc\\\": \\\"e2847ed5ae112c208a42fb629a590c3da6f55830676a44954cfacabcf6e315666343ea53c93db241fd37c6ba98ac8f703af8286961605cb53c8ca249738e035ca41ac9c92fe8ca38c498b62165de133166c7f9de09cdce5c1c54456b27e990e2d61075c67d683c61e87bcead8a94af1bf1d9c91f88e6dd8f38e76312d2e77c622afc00c6bfbd5bdebe01a36c6e63b205f42dd9798d6befe357093808609fed789160e166fefd9c137a72b2db8795d532d964e2a9ff3df18176\\\"}\",\"{\\\"s\\\": \\\"1c16090f62b45fc9a1404b2135beab94\\\", \\\"oc\\\": \\\"e108bb37379dfda0f0143886d8b97d5813f5decb57cd129b4a3cfe86ba1339f27bee6833a277032ea11fd3cdaea0249e4dfd806fd691c652212db9a671e096a540\\\"}\"}'::jsonb[])",
              "Index Name": "json_ste_vec_small_encrypted_v3_1000000_jsonb_array_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 111.51
            }
          ],
          "Recheck Cond": "(eql_v3.jsonb_array((value)::jsonb) @> '{\"{\\\"s\\\": \\\"6b769ff71409e303be5ca17b08063375\\\", \\\"oc\\\": \\\"e2847ed446d572477e0e74ae828304897b9f568962af078cdd5e9f431c2c05d49d78229e2d75115eee93156538ce6055ce65b627f446147bd3\\\"}\",\"{\\\"s\\\": \\\"75146c96a91ff82146e29e87b5accb95\\\", \\\"hm\\\": \\\"98b7664512c8c0125772e43e47dde0ce\\\"}\",\"{\\\"s\\\": \\\"b7cb88c99e692f82aa89ac83e6d52a83\\\", \\\"oc\\\": \\\"e2847ed445b99b00a005cc452ce307db927ed0c8b81d13e19cbb4c97c25a98a345ce77a106ed1accffff18f5c5a8d521f49164c2570fe0e15944dabc246a97232eed758beac7af182c7c455eb7763c5421a9eec34836c60124\\\"}\",\"{\\\"s\\\": \\\"25519d3ecf26f1d3833f11d9757135a7\\\", \\\"oc\\\": \\\"e2847ed5ae112c208a42fb629a590c3da6f55830676a44954cfacabcf6e315666343ea53c93db241fd37c6ba98ac8f703af8286961605cb53c8ca249738e035ca41ac9c92fe8ca38c498b62165de133166c7f9de09cdce5c1c54456b27e990e2d61075c67d683c61e87bcead8a94af1bf1d9c91f88e6dd8f38e76312d2e77c622afc00c6bfbd5bdebe01a36c6e63b205f42dd9798d6befe357093808609fed789160e166fefd9c137a72b2db8795d532d964e2a9ff3df18176\\\"}\",\"{\\\"s\\\": \\\"1c16090f62b45fc9a1404b2135beab94\\\", \\\"oc\\\": \\\"e108bb37379dfda0f0143886d8b97d5813f5decb57cd129b4a3cfe86ba1339f27bee6833a277032ea11fd3cdaea0249e4dfd806fd691c652212db9a671e096a540\\\"}\"}'::jsonb[])",
          "Relation Name": "json_ste_vec_small_encrypted_v3_1000000",
          "Startup Cost": 111.51,
          "Total Cost": 115.78
        }
      ],
      "Startup Cost": 111.51,
      "Total Cost": 115.78
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Bitmap Heap Scan on json_ste_vec_small_encrypted_v3_10000000
    Bitmap Index Scan using json_ste_vec_small_encrypted_v3_10000000_jsonb_array_index
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
          "Alias": "json_ste_vec_small_encrypted_v3_10000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 4,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "(eql_v3.jsonb_array((value)::jsonb) @> '{\"{\\\"s\\\": \\\"6b769ff71409e303be5ca17b08063375\\\", \\\"oc\\\": \\\"e2847ed445b99b00a005cc452b2f322ff38b63e1fd75ec0c254ef4eab3454d73767587fdd4e961c3d6719c8851f75df976\\\"}\",\"{\\\"s\\\": \\\"1c16090f62b45fc9a1404b2135beab94\\\", \\\"oc\\\": \\\"e108bb37379dfda0f0143886d8b88f3dc8ef1dbc11bad17ae69d7bd0c112796ba6416baf0e3b0dc577886a15389fc364d1b3ceb56488adecb738298ff9b47cd306\\\"}\",\"{\\\"s\\\": \\\"75146c96a91ff82146e29e87b5accb95\\\", \\\"hm\\\": \\\"98b7664512c8c0125772e43e47dde0ce\\\"}\",\"{\\\"s\\\": \\\"25519d3ecf26f1d3833f11d9757135a7\\\", \\\"oc\\\": \\\"e2847ed5ae112b8eee58a8745ea7dfee54ece27100c9bdd278919b961e699db652bba421a06e89490da16222946fbfb0930425811f6a467a6c47a0fb2a5c1137f336085f4b277c939e2e775adc108ba19fdec36e352996de6a29fe5dd44c35750a4ba83fefcab64f8d5b71ee2d5918284de3e48a14c237e38ba457ae00a5cf594ae52bb6d35fffb84c9b0937a5d35a60a7b4dfeae483a9d4abdbbdc797201cca110864ac480c384077184935682e302ee65f3283118bcf4707\\\"}\",\"{\\\"s\\\": \\\"b7cb88c99e692f82aa89ac83e6d52a83\\\", \\\"oc\\\": \\\"e2847ed445baeddc1bf42210e9f10a92d31e6e326398d47cb4f593328e1832b2656e3b102da2deda13\\\"}\"}'::jsonb[])",
              "Index Name": "json_ste_vec_small_encrypted_v3_10000000_jsonb_array_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 132.79
            }
          ],
          "Recheck Cond": "(eql_v3.jsonb_array((value)::jsonb) @> '{\"{\\\"s\\\": \\\"6b769ff71409e303be5ca17b08063375\\\", \\\"oc\\\": \\\"e2847ed445b99b00a005cc452b2f322ff38b63e1fd75ec0c254ef4eab3454d73767587fdd4e961c3d6719c8851f75df976\\\"}\",\"{\\\"s\\\": \\\"1c16090f62b45fc9a1404b2135beab94\\\", \\\"oc\\\": \\\"e108bb37379dfda0f0143886d8b88f3dc8ef1dbc11bad17ae69d7bd0c112796ba6416baf0e3b0dc577886a15389fc364d1b3ceb56488adecb738298ff9b47cd306\\\"}\",\"{\\\"s\\\": \\\"75146c96a91ff82146e29e87b5accb95\\\", \\\"hm\\\": \\\"98b7664512c8c0125772e43e47dde0ce\\\"}\",\"{\\\"s\\\": \\\"25519d3ecf26f1d3833f11d9757135a7\\\", \\\"oc\\\": \\\"e2847ed5ae112b8eee58a8745ea7dfee54ece27100c9bdd278919b961e699db652bba421a06e89490da16222946fbfb0930425811f6a467a6c47a0fb2a5c1137f336085f4b277c939e2e775adc108ba19fdec36e352996de6a29fe5dd44c35750a4ba83fefcab64f8d5b71ee2d5918284de3e48a14c237e38ba457ae00a5cf594ae52bb6d35fffb84c9b0937a5d35a60a7b4dfeae483a9d4abdbbdc797201cca110864ac480c384077184935682e302ee65f3283118bcf4707\\\"}\",\"{\\\"s\\\": \\\"b7cb88c99e692f82aa89ac83e6d52a83\\\", \\\"oc\\\": \\\"e2847ed445baeddc1bf42210e9f10a92d31e6e326398d47cb4f593328e1832b2656e3b102da2deda13\\\"}\"}'::jsonb[])",
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000000",
          "Startup Cost": 132.79,
          "Total Cost": 137.05
        }
      ],
      "Startup Cost": 132.79,
      "Total Cost": 137.05
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
| 10,000 | 10 | 113.66μs | N/A |
| 100,000 | 10 | 115.15μs | N/A |
| 1,000,000 | 10 | 111.50μs | N/A |
| 10,000,000 | 10 | 96.58μs | N/A |

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
          "Index Cond": "(eql_v3.eq_term(eql_v3.jsonb_path_query_first((value)::jsonb, '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbKl&U3G&rdmu!jlvOTNtai|WzTf%GxRRf|8D&*yFcQeZw~`a2wXjcee|dUu?(RM*o=Mf^e@IT*g|>~0V||-vzKbT*s~X`M^rdAlgWgZ#Jz9V6vZ`F(4Uv%?$9NLh8{gxllTC75JJ6cXpEbP!uwg-8nnb9kNg%aApC#s*+tb_lY_4JjIjc(PWht(iJW)gKqCBRpuh\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_10000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_10000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 2242.54
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 2.78
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
          "Index Cond": "(eql_v3.eq_term(eql_v3.jsonb_path_query_first((value)::jsonb, '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbL>G+POZxYGacl>M3T^U9ILZDpH%1hfdNmea4A8^XTV*dGo;>8yO5r?BT&aSI_ZsSuV&t8=ymMaG~D6e7#MN-qMs)Y%I_YVN-rsOB%cplB^itOiGKiEl{QCd4G!M9XzS&)=SJlG6IPjHQ5xoo1+$vm3D)DE!$9mVU$_o~@jtT%uE4Q&7M?5}sa{E9tIM-1n8!Nah=}AiNxXpuh\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_100000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_100000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 100000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 22303.54
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 2.78
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
          "Index Cond": "(eql_v3.eq_term(eql_v3.jsonb_path_query_first((value)::jsonb, '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbJ%T#O?hLqA`E(Oz(edIR9ZY@HZ~!poM<8VHyRM^Y>!N3+f4h3PAmRMTaIoe$WB`ON`M4Kyne+%ye<b0zPQH(a~0;Ay@XrYrO0+7ko}&+dcQfNE+{e}K-~vZL;QL*|J8=3$OIJBbH13@uqV;UZFD6pWpDeR(o2X~ZBH2XvuKJdUgV+TX<kfjSNxcFxD>{{cPA*_4ri{z#dizy\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_1000000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_1000000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1001642,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.68,
          "Total Cost": 222842.41
        }
      ],
      "Startup Cost": 0.68,
      "Total Cost": 2.9
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
          "Index Cond": "(eql_v3.eq_term(eql_v3.jsonb_path_query_first((value)::jsonb, '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbKdL{%RNuK>0?jR{}{TcEhaWm?it8o3@A!B9O_Ve_vbf!P^7`HpOMhg7r=0>?cU1z3gGN?;ss-t|Pc*N4jA6b4XX?4ja(m^ns%IO=0v;tCgV#q5`TSV2hKIfV#*0<a8LH}>7WP+Ji7W{F$7iY3M9G5^FM;|!vaI5(ygH_cN#w3lA&0TG8q6@l&@FR^NUj6QUOpuh\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_10000000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_10000000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 9998476,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.69,
          "Total Cost": 2112453.02
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
| 10,000 | 10 | 807.07μs | N/A |
| 100,000 | 10 | 6.13ms | N/A |
| 1,000,000 | 10 | 72.04ms | N/A |
| 10,000,000 | 10 | 636.13μs | N/A |

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
          "Filter": "(eql_v3.jsonb_array((value)::jsonb) @> '{\"{\\\"s\\\": \\\"75146c96a91ff82146e29e87b5accb95\\\", \\\"hm\\\": \\\"98b7664512c8c0125772e43e47dde0ce\\\"}\"}'::jsonb[])",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000",
          "Startup Cost": 0.0,
          "Total Cost": 4645.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 4.64
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
          "Filter": "(eql_v3.jsonb_array((value)::jsonb) @> '{\"{\\\"s\\\": \\\"75146c96a91ff82146e29e87b5accb95\\\", \\\"hm\\\": \\\"98b7664512c8c0125772e43e47dde0ce\\\"}\"}'::jsonb[])",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 100000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_100000",
          "Startup Cost": 0.0,
          "Total Cost": 46444.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 4.64
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
          "Filter": "(eql_v3.jsonb_array((value)::jsonb) @> '{\"{\\\"s\\\": \\\"75146c96a91ff82146e29e87b5accb95\\\", \\\"hm\\\": \\\"98b7664512c8c0125772e43e47dde0ce\\\"}\"}'::jsonb[])",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1001642,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 464757.03
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 4.64
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
          "Filter": "(eql_v3.jsonb_array((value)::jsonb) @> '{\"{\\\"s\\\": \\\"75146c96a91ff82146e29e87b5accb95\\\", \\\"hm\\\": \\\"98b7664512c8c0125772e43e47dde0ce\\\"}\"}'::jsonb[])",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 9998476,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 4527343.95
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 4.53
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
| 10,000 | 10 | 111.68μs | N/A |
| 100,000 | 10 | 115.59μs | N/A |
| 1,000,000 | 10 | 119.66μs | N/A |
| 10,000,000 | 10 | 97.53μs | N/A |

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
          "Index Cond": "(eql_v3.eq_term(eql_v3.jsonb_path_query_first((value)::jsonb, '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbKl&U3G&rdmu!jlvOTNtai|WzTf%GxRRf|8D&*yFcQeZw~`a2wXjcee|dUu?(RM*o=Mf^e@IT*g|>~0V||-vzKbT*s~X`M^rdAlgWgZ#Jz9V6vZ`F(4Uv%?$9NLh8{gxllTC75JJ6cXpEbP!uwg-8nnb9kNg%aApC#s*+tb_lY_4JjIjc(PWht(iJW)gKqCBRpuh\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_10000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_10000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 2242.54
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 2.78
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
          "Index Cond": "(eql_v3.eq_term(eql_v3.jsonb_path_query_first((value)::jsonb, '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbL>G+POZxYGacl>M3T^U9ILZDpH%1hfdNmea4A8^XTV*dGo;>8yO5r?BT&aSI_ZsSuV&t8=ymMaG~D6e7#MN-qMs)Y%I_YVN-rsOB%cplB^itOiGKiEl{QCd4G!M9XzS&)=SJlG6IPjHQ5xoo1+$vm3D)DE!$9mVU$_o~@jtT%uE4Q&7M?5}sa{E9tIM-1n8!Nah=}AiNxXpuh\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_100000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_100000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 100000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 22303.54
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 2.78
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
          "Index Cond": "(eql_v3.eq_term(eql_v3.jsonb_path_query_first((value)::jsonb, '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbJ%T#O?hLqA`E(Oz(edIR9ZY@HZ~!poM<8VHyRM^Y>!N3+f4h3PAmRMTaIoe$WB`ON`M4Kyne+%ye<b0zPQH(a~0;Ay@XrYrO0+7ko}&+dcQfNE+{e}K-~vZL;QL*|J8=3$OIJBbH13@uqV;UZFD6pWpDeR(o2X~ZBH2XvuKJdUgV+TX<kfjSNxcFxD>{{cPA*_4ri{z#dizy\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_1000000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_1000000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1001642,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.68,
          "Total Cost": 222842.41
        }
      ],
      "Startup Cost": 0.68,
      "Total Cost": 2.9
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
          "Index Cond": "(eql_v3.eq_term(eql_v3.jsonb_path_query_first((value)::jsonb, '75146c96a91ff82146e29e87b5accb95'::text)) = eql_v3.eq_term(('{\"a\": false, \"c\": \"mBbKdL{%RNuK>0?jR{}{TcEhaWm?it8o3@A!B9O_Ve_vbf!P^7`HpOMhg7r=0>?cU1z3gGN?;ss-t|Pc*N4jA6b4XX?4ja(m^ns%IO=0v;tCgV#q5`TSV2hKIfV#*0<a8LH}>7WP+Ji7W{F$7iY3M9G5^FM;|!vaI5(ygH_cN#w3lA&0TG8q6@l&@FR^NUj6QUOpuh\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_10000000\"}, \"s\": \"75146c96a91ff82146e29e87b5accb95\", \"v\": 3, \"hm\": \"98b7664512c8c0125772e43e47dde0ce\"}'::jsonb)::jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_10000000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 9998476,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.69,
          "Total Cost": 2112453.02
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

## field_gt/functional

**Description:** Unknown query

****

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
| 10,000 | 10 | 234.47μs | N/A |
| 100,000 | 10 | 848.51μs | N/A |
| 1,000,000 | 10 | 258.98μs | N/A |
| 10,000,000 | 10 | 392.79μs | N/A |

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
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_10000",
          "Async Capable": false,
          "Index Cond": "(eql_v3.ore_cllw(eql_v3.jsonb_path_query_first((value)::jsonb, '1c16090f62b45fc9a1404b2135beab94'::text)) > eql_v3.ore_cllw(('{\"a\": false, \"c\": \"mBbKl&U3G&rdmu!jlvOTNtai|6gK>*kN_ZyJpH9*c*$KTw9`)`)x;o={1z-A{D1D*Mb%o9gRb|Cu>!45`J)1foOj_sBK&5czy\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_10000\"}, \"s\": \"1c16090f62b45fc9a1404b2135beab94\", \"v\": 3, \"oc\": \"e108bb37379dfda0f0143886d8b88e748150ce18ba0d1fa98dda6bf96cb47d12150661d4710365e26993420af2ccd1d836df55ccbc9bb7a6fb4f5589c09032c4bf\"}'::jsonb)::jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_10000_field_order_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 2646,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.66,
          "Total Cost": 6593.76
        }
      ],
      "Startup Cost": 0.66,
      "Total Cost": 25.58
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
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_100000",
          "Async Capable": false,
          "Index Cond": "(eql_v3.ore_cllw(eql_v3.jsonb_path_query_first((value)::jsonb, '1c16090f62b45fc9a1404b2135beab94'::text)) > eql_v3.ore_cllw(('{\"a\": false, \"c\": \"mBbL>G+POZxYGacl>M3T^U9IL6fdmJ|9;<-fD;-|!-w$DNLmafTf`uqt(>D=qElQ`P{2JBo?e$L>8?`T_m$H~<{Pshyc~U?zy\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_100000\"}, \"s\": \"1c16090f62b45fc9a1404b2135beab94\", \"v\": 3, \"oc\": \"e108bb37379dfda0f0143886d759d250a567842a7c3f5af2a6b3f2a58a8160fc3e4079dc9e54b19e671a8834488db7592a060356887c3e718a86beaefae5648294\"}'::jsonb)::jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_100000_field_order_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 62310,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.67,
          "Total Cost": 85355.09
        }
      ],
      "Startup Cost": 0.67,
      "Total Cost": 14.37
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
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_1000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3.ore_cllw(eql_v3.jsonb_path_query_first((value)::jsonb, '1c16090f62b45fc9a1404b2135beab94'::text)) > eql_v3.ore_cllw(('{\"a\": false, \"c\": \"mBbJ%T#O?hLqA`E(Oz(edIR9Z6r5ZRt;(F4$<bk-6`fMke26}0fy5vf2XvuKJdUgV+TX<kfjSNxcFxD>{{cPA*_4ri{z#dizy\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_1000000\"}, \"s\": \"1c16090f62b45fc9a1404b2135beab94\", \"v\": 3, \"oc\": \"e108bb37379dfda0f0143886d8b97d5813f5decb57cd129b4a3cfe86ba1339f27bee6833a277032ea11fd3cdaea0249e4dfd806fd691c652212db9a671e096a540\"}'::jsonb)::jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_1000000_field_order_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 23672,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.8,
          "Total Cost": 91165.11
        }
      ],
      "Startup Cost": 0.8,
      "Total Cost": 39.31
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
      "Plan Width": 4,
      "Plans": [
        {
          "Alias": "json_ste_vec_small_encrypted_v3_10000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3.ore_cllw(eql_v3.jsonb_path_query_first((value)::jsonb, '1c16090f62b45fc9a1404b2135beab94'::text)) > eql_v3.ore_cllw(('{\"a\": false, \"c\": \"mBbKdL{%RNuK>0?jR{}{TcEha6s0(h*T(hF&eWwwx?&mD6;1%QQN$qQ45E=ZH>MOf%~L$ImtO1v5r;(;f$kkIv1)ycK6Hbizy\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_v3_10000000\"}, \"s\": \"1c16090f62b45fc9a1404b2135beab94\", \"v\": 3, \"oc\": \"e108bb37379dfda0f0143886d8b88f3dc8ef1dbc11bad17ae69d7bd0c112796ba6416baf0e3b0dc577886a15389fc364d1b3ceb56488adecb738298ff9b47cd306\"}'::jsonb)::jsonb_entry))",
          "Index Name": "json_ste_vec_small_encrypted_v3_10000000_field_order_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1190152,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.81,
          "Total Cost": 3713359.52
        }
      ],
      "Startup Cost": 0.81,
      "Total Cost": 32.01
    }
  }
]
```

</details>

![Query Performance - JSON/field_gt/functional](query_json_field_gt_functional_chart.png)

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
| 10,000 | 10 | 257.22μs | N/A |
| 100,000 | 10 | 279.27μs | N/A |
| 1,000,000 | 10 | 286.05μs | N/A |
| 10,000,000 | 10 | 343.30μs | N/A |

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
          "Total Cost": 13797.16
        }
      ],
      "Startup Cost": 0.41,
      "Total Cost": 14.21
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
          "Total Cost": 137868.42
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
          "Plan Rows": 1001642,
          "Plan Width": 36,
          "Relation Name": "json_ste_vec_small_encrypted_v3_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.55,
          "Total Cost": 1378881.81
        }
      ],
      "Startup Cost": 0.55,
      "Total Cost": 14.32
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
          "Plan Rows": 9998476,
          "Plan Width": 36,
          "Relation Name": "json_ste_vec_small_encrypted_v3_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.56,
          "Total Cost": 13318010.53
        }
      ],
      "Startup Cost": 0.56,
      "Total Cost": 13.88
    }
  }
]
```

</details>

![Query Performance - JSON/field_order/functional](query_json_field_order_functional_chart.png)

