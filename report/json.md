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

- 10,000: `json_ste_vec_small_encrypted_10000_jsonb_array_index`
- 100,000: `json_ste_vec_small_encrypted_100000_jsonb_array_index`
- 1,000,000: `json_ste_vec_small_encrypted_1000000_jsonb_array_index`
- 10,000,000: `json_ste_vec_small_encrypted_10000000_jsonb_array_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 1 | 238.33μs | N/A |
| 100,000 | 1 | 287.01μs | N/A |
| 1,000,000 | 1 | 433.92μs | N/A |
| 10,000,000 | 1 | 853.79μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Bitmap Heap Scan on json_ste_vec_small_encrypted_10000
    Bitmap Index Scan using json_ste_vec_small_encrypted_10000_jsonb_array_index
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
          "Alias": "json_ste_vec_small_encrypted_10000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 4,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db49711c282c9945019a0d5d828669b07666c22835fea10dec1a7c1b6a6f19643526e9fc2f711acceb0290e4deab64aa2b340c662c9\\\"}\",\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db5d403405be41b6282af7f22db8d658eb80dd800b412c08aa4b2cc3a75b9f4ce55d3df3c2335e40f7e45297ada\\\"}\",\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e403ea3be1fea72fbc01ec952a09371fc9612b428b7ded13cecfef3b2cd8fcf7cb40c4b548f4c91d187ad027907127860abcc58b92b1670f85e5178cfeda25766f3f6de0327c7813de3e52dbd36b43d96e847acf1153f5e023bdff8fdff1a784fd2a8f33ddb78b3849e8e5372a2519a2e3ac83c226bbd0643ee330d80e925bfe30c189b749ee761281403e3e9332f03f5c731be4a94c42f14eedd5afde4f40601554d424a5a\\\"}\",\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\",\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0984d4957290b0eb89f172928170998bc114ac343a673440e682d03234d35c039ae0ba9a5bba6f3901e018ba8c10f88db4cfa3e37e6357\\\"}\"}'::jsonb[])",
              "Index Name": "json_ste_vec_small_encrypted_10000_jsonb_array_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 68.86
            }
          ],
          "Recheck Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db49711c282c9945019a0d5d828669b07666c22835fea10dec1a7c1b6a6f19643526e9fc2f711acceb0290e4deab64aa2b340c662c9\\\"}\",\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db5d403405be41b6282af7f22db8d658eb80dd800b412c08aa4b2cc3a75b9f4ce55d3df3c2335e40f7e45297ada\\\"}\",\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e403ea3be1fea72fbc01ec952a09371fc9612b428b7ded13cecfef3b2cd8fcf7cb40c4b548f4c91d187ad027907127860abcc58b92b1670f85e5178cfeda25766f3f6de0327c7813de3e52dbd36b43d96e847acf1153f5e023bdff8fdff1a784fd2a8f33ddb78b3849e8e5372a2519a2e3ac83c226bbd0643ee330d80e925bfe30c189b749ee761281403e3e9332f03f5c731be4a94c42f14eedd5afde4f40601554d424a5a\\\"}\",\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\",\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0984d4957290b0eb89f172928170998bc114ac343a673440e682d03234d35c039ae0ba9a5bba6f3901e018ba8c10f88db4cfa3e37e6357\\\"}\"}'::jsonb[])",
          "Relation Name": "json_ste_vec_small_encrypted_10000",
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
  Bitmap Heap Scan on json_ste_vec_small_encrypted_100000
    Bitmap Index Scan using json_ste_vec_small_encrypted_100000_jsonb_array_index
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
          "Alias": "json_ste_vec_small_encrypted_100000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 4,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db4971212d5f2ebe58b72e7b8fa9d528196612fc962cd5f14cc85a882d495c5b8ceb74335081e41aebe9001cefa\\\"}\",\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e403ea40004cb28aaac8579a40c9c68653f7a1c8d9218ad7259f4c37e28164a5a9f1db7ecb4dea34907606105b4452af42e910a89810c847757038d7ebf79c9a6872a60a39e10541852a9e017fe49d2cf3ec12edbb2421f0d9b4f528de812be40d57dd4943c3a26424f3fec93108c779a185f6fa2ffe1f2d0e2140cb54042b17262a629b52ceca7af4d8f8c9aed1d963e206c1665b0e991d8a6e0fa9ce055393f253aae9a3d4deb0c90f023b5010edb624f036e23c373a974fdd10f0664040cdbf9692db261\\\"}\",\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0a3129231c38a147def14a10589ac6578debca348aa74b3bc9d81b55e6679aafb3f791a0549e371ad4d1a962a1c24fcd3c33beaf6012e9\\\"}\",\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\",\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db49711c141b2cd54aca1d3dc8a37a02a141772209033e93fe56b3ef7bce24c1636c1edc48440f71a57f084f0498568a3af3e15f7fe\\\"}\"}'::jsonb[])",
              "Index Name": "json_ste_vec_small_encrypted_100000_jsonb_array_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 90.19
            }
          ],
          "Recheck Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db4971212d5f2ebe58b72e7b8fa9d528196612fc962cd5f14cc85a882d495c5b8ceb74335081e41aebe9001cefa\\\"}\",\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e403ea40004cb28aaac8579a40c9c68653f7a1c8d9218ad7259f4c37e28164a5a9f1db7ecb4dea34907606105b4452af42e910a89810c847757038d7ebf79c9a6872a60a39e10541852a9e017fe49d2cf3ec12edbb2421f0d9b4f528de812be40d57dd4943c3a26424f3fec93108c779a185f6fa2ffe1f2d0e2140cb54042b17262a629b52ceca7af4d8f8c9aed1d963e206c1665b0e991d8a6e0fa9ce055393f253aae9a3d4deb0c90f023b5010edb624f036e23c373a974fdd10f0664040cdbf9692db261\\\"}\",\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0a3129231c38a147def14a10589ac6578debca348aa74b3bc9d81b55e6679aafb3f791a0549e371ad4d1a962a1c24fcd3c33beaf6012e9\\\"}\",\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\",\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db49711c141b2cd54aca1d3dc8a37a02a141772209033e93fe56b3ef7bce24c1636c1edc48440f71a57f084f0498568a3af3e15f7fe\\\"}\"}'::jsonb[])",
          "Relation Name": "json_ste_vec_small_encrypted_100000",
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
  Bitmap Heap Scan on json_ste_vec_small_encrypted_1000000
    Bitmap Index Scan using json_ste_vec_small_encrypted_1000000_jsonb_array_index
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
          "Alias": "json_ste_vec_small_encrypted_1000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 4,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\",\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db5d4045f7eae249da635745e1a5e6691a7feb66fed367ce633d7792fef78397cb5ba02b5c0da766c5ace2e9b3c\\\"}\",\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e403ea40004cb28aaac8579a40c9c68653f7a1c8d93be87493ade620ca595abd48e66e220c9e30c77842a002a700b059d4d59250090b87f4a38fa321bf852c5d7458ab0d670443ebf9a445e27bfe098819e341e0c431b2b50e889d713e682b1937939d1370f0dd64421ea3610fae0a279833dc61c14ef788dffca9f26246050d98430966dac692632292260560226b1ea6cec1e2f57a80fe979f14c65bf9dd364449f136ba3\\\"}\",\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0984d495716bf9ae2c0a31447b038fd5bd56472d846a1a2cb71a3b9f5436ef8fc2c6c25c58e2e9e6fc05934730282d950e9b98d221f899\\\"}\",\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db4984a02b345757574904a5f1c307b11290aaa1b1b4542856950ff97176a23e23b1ca676713a1a2384ea826486\\\"}\"}'::jsonb[])",
              "Index Name": "json_ste_vec_small_encrypted_1000000_jsonb_array_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 111.51
            }
          ],
          "Recheck Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\",\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db5d4045f7eae249da635745e1a5e6691a7feb66fed367ce633d7792fef78397cb5ba02b5c0da766c5ace2e9b3c\\\"}\",\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e403ea40004cb28aaac8579a40c9c68653f7a1c8d93be87493ade620ca595abd48e66e220c9e30c77842a002a700b059d4d59250090b87f4a38fa321bf852c5d7458ab0d670443ebf9a445e27bfe098819e341e0c431b2b50e889d713e682b1937939d1370f0dd64421ea3610fae0a279833dc61c14ef788dffca9f26246050d98430966dac692632292260560226b1ea6cec1e2f57a80fe979f14c65bf9dd364449f136ba3\\\"}\",\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0984d495716bf9ae2c0a31447b038fd5bd56472d846a1a2cb71a3b9f5436ef8fc2c6c25c58e2e9e6fc05934730282d950e9b98d221f899\\\"}\",\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db4984a02b345757574904a5f1c307b11290aaa1b1b4542856950ff97176a23e23b1ca676713a1a2384ea826486\\\"}\"}'::jsonb[])",
          "Relation Name": "json_ste_vec_small_encrypted_1000000",
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
  Bitmap Heap Scan on json_ste_vec_small_encrypted_10000000
    Bitmap Index Scan using json_ste_vec_small_encrypted_10000000_jsonb_array_index
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
          "Alias": "json_ste_vec_small_encrypted_10000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 4,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\",\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e4113e9f962fbe6fe249a5de0f12ba7ed560c524ef29c96018408b83c94c4c76ef043485fdb4c0d3dbe1c73986f14fea459d4976516c1b96cdd3820f5b5f9a3517a94cf901aea6edf910824dc75cb38166bc34693f784dde67958dbda26a643bbca03014b39e7d2aef06e009af176dccb11c27947e4ff7f9394632439395b3c101d8e8c721c6c5600da3d64a9aeea4520829440901344c2377277039617\\\"}\",\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db4971212d687dc77e2922544be2e3a767b9e6e92cae4e7f344ae9bf59a6986cfacd75f6cd0867c19ad4280b0919b6a3e929b02cd6ddf62d400008e47943e3d6fcb89d5046e32912438f622e77d\\\"}\",\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0984d494d6304819280964b36414fc29e4ea7f18c10a0b42d088191c96642d1a2434e91c0d4dcb0fa2fd386a20f67a128a29dc864feec2\\\"}\",\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db49711c282c9945019a194351fb299f75ed665afb97d57ea0658a61a53b755e69da35ab919\\\"}\"}'::jsonb[])",
              "Index Name": "json_ste_vec_small_encrypted_10000000_jsonb_array_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 132.79
            }
          ],
          "Recheck Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\",\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e4113e9f962fbe6fe249a5de0f12ba7ed560c524ef29c96018408b83c94c4c76ef043485fdb4c0d3dbe1c73986f14fea459d4976516c1b96cdd3820f5b5f9a3517a94cf901aea6edf910824dc75cb38166bc34693f784dde67958dbda26a643bbca03014b39e7d2aef06e009af176dccb11c27947e4ff7f9394632439395b3c101d8e8c721c6c5600da3d64a9aeea4520829440901344c2377277039617\\\"}\",\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db4971212d687dc77e2922544be2e3a767b9e6e92cae4e7f344ae9bf59a6986cfacd75f6cd0867c19ad4280b0919b6a3e929b02cd6ddf62d400008e47943e3d6fcb89d5046e32912438f622e77d\\\"}\",\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0984d494d6304819280964b36414fc29e4ea7f18c10a0b42d088191c96642d1a2434e91c0d4dcb0fa2fd386a20f67a128a29dc864feec2\\\"}\",\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db49711c282c9945019a194351fb299f75ed665afb97d57ea0658a61a53b755e69da35ab919\\\"}\"}'::jsonb[])",
          "Relation Name": "json_ste_vec_small_encrypted_10000000",
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

- 10,000: `json_ste_vec_small_encrypted_10000_field_eq_idx`
- 100,000: `json_ste_vec_small_encrypted_100000_field_eq_idx`
- 1,000,000: `json_ste_vec_small_encrypted_1000000_field_eq_idx`
- 10,000,000: `json_ste_vec_small_encrypted_10000000_field_eq_idx`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 109.04μs | N/A |
| 100,000 | 10 | 106.46μs | N/A |
| 1,000,000 | 10 | 99.38μs | N/A |
| 10,000,000 | 10 | 109.18μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_10000_field_eq_idx on json_ste_vec_small_encrypted_10000
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
          "Alias": "json_ste_vec_small_encrypted_10000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbLP(i?F+N4}@Yd7+CEvBh%4WvWGA<h{FGbNV1!h_~GM+R{4<R%gGA^5z1!J`t;RabS41$BL|cbtCn0spJ@&#ERRK(o5wj{F{|OF?atat$?IP%@Rdo{qt=sck<h&lvY~~b>-Clm>I7>+FpI2@T`6kjE%$~vnW*H{GSQRxTsN%B(Hto#H*A)ql=&pvnJm04OwTPpuh\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_10000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Index Name": "json_ste_vec_small_encrypted_10000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 2046.54
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 2.58
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_100000_field_eq_idx on json_ste_vec_small_encrypted_100000
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
          "Alias": "json_ste_vec_small_encrypted_100000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbJ#Az`rN2!?|a+B*-Bmmq?~X$Xb;T3=`{)vYGWIVl1O4v-;wGe0L(<c%CCop1ndYd4l$1RikA3uK(p;O;JEcOB<sFtF|SZh2Lr+*6Rqqt$PLt1C8K7-gCWQ@yqWfUP(Y>${xlw#6&~c2+OgUrj#oFGkUC;JCyf+BZwxob;Y*Mape$W+(md@rfSN5Ye@09b#rnIlsO#puh\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_100000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Index Name": "json_ste_vec_small_encrypted_100000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 100000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 19741.54
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 2.52
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_1000000_field_eq_idx on json_ste_vec_small_encrypted_1000000
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
          "Alias": "json_ste_vec_small_encrypted_1000000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbJeS~GW(rh_Ip#L(=Tk4TQhWW<9ljWmSq7yFQK{3){*^3mrvZVb*R$1K>{#a~UP>9qrQVA&6A(*plc?p!)G6K6VxXRF-Ao<%8ZuHeK?nJ3--0Nm!1=ln2Xui4i>S)gYG;I(F^bd2`!ND6isp!LMOUBn>CRQ!LHS7`%z%ZEdL=*l4D?dx-|<xS-To1jdGfEmP~zy\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_1000000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Index Name": "json_ste_vec_small_encrypted_1000000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1002404,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.68,
          "Total Cost": 187629.75
        }
      ],
      "Startup Cost": 0.68,
      "Total Cost": 2.55
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_10000000_field_eq_idx on json_ste_vec_small_encrypted_10000000
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
          "Alias": "json_ste_vec_small_encrypted_10000000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbJLhWn-J_XuAF@lJV1yj2RsW<G@;SoGJ^1z{wePh$|Pa&~|P6&(iAjnLjV?%U*uGw-s<T3ooXc-G07xL}j6&H#JO#ij$#MPRB#D8@rmF0_>Y00Q2F%S?!W`})*vltb>7kZ*|BK4On8t^sSE)U-JiRV)L<Ap5Fa3feN+iXv@<@H0YaMqt6p@vX2W;eVQs-4qb+fS|w\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_10000000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Index Name": "json_ste_vec_small_encrypted_10000000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 9986163,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.69,
          "Total Cost": 1876261.54
        }
      ],
      "Startup Cost": 0.69,
      "Total Cost": 2.57
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
| 10,000 | 10 | 506.68μs | N/A |
| 100,000 | 10 | 477.53μs | N/A |
| 1,000,000 | 10 | 465.99μs | N/A |
| 10,000,000 | 10 | 432.23μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Seq Scan on json_ste_vec_small_encrypted_10000
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
          "Alias": "json_ste_vec_small_encrypted_10000",
          "Async Capable": false,
          "Filter": "((eql_v2.to_stevec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}]}'::jsonb)::eql_v2.stevec_query)::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 9999,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 4474.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 4.47
    }
  }
]
```

**100,000 rows**

```
Limit
  Seq Scan on json_ste_vec_small_encrypted_100000
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
          "Alias": "json_ste_vec_small_encrypted_100000",
          "Async Capable": false,
          "Filter": "((eql_v2.to_stevec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}]}'::jsonb)::eql_v2.stevec_query)::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 99990,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 44132.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 4.41
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Seq Scan on json_ste_vec_small_encrypted_1000000
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
          "Alias": "json_ste_vec_small_encrypted_1000000",
          "Async Capable": false,
          "Filter": "((eql_v2.to_stevec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}]}'::jsonb)::eql_v2.stevec_query)::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1002304,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 432237.06
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 4.31
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Seq Scan on json_ste_vec_small_encrypted_10000000
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
          "Alias": "json_ste_vec_small_encrypted_10000000",
          "Async Capable": false,
          "Filter": "((eql_v2.to_stevec_query(value))::jsonb @> (('{\"sv\": [{\"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}]}'::jsonb)::eql_v2.stevec_query)::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 9985164,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 4313101.2
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 4.32
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

- 10,000: `json_ste_vec_small_encrypted_10000_field_eq_idx`
- 100,000: `json_ste_vec_small_encrypted_100000_field_eq_idx`
- 1,000,000: `json_ste_vec_small_encrypted_1000000_field_eq_idx`
- 10,000,000: `json_ste_vec_small_encrypted_10000000_field_eq_idx`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 105.03μs | N/A |
| 100,000 | 10 | 102.71μs | N/A |
| 1,000,000 | 10 | 103.47μs | N/A |
| 10,000,000 | 10 | 106.50μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_10000_field_eq_idx on json_ste_vec_small_encrypted_10000
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
          "Alias": "json_ste_vec_small_encrypted_10000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbLP(i?F+N4}@Yd7+CEvBh%4WvWGA<h{FGbNV1!h_~GM+R{4<R%gGA^5z1!J`t;RabS41$BL|cbtCn0spJ@&#ERRK(o5wj{F{|OF?atat$?IP%@Rdo{qt=sck<h&lvY~~b>-Clm>I7>+FpI2@T`6kjE%$~vnW*H{GSQRxTsN%B(Hto#H*A)ql=&pvnJm04OwTPpuh\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_10000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Index Name": "json_ste_vec_small_encrypted_10000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 2046.54
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 2.58
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_100000_field_eq_idx on json_ste_vec_small_encrypted_100000
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
          "Alias": "json_ste_vec_small_encrypted_100000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbJ#Az`rN2!?|a+B*-Bmmq?~X$Xb;T3=`{)vYGWIVl1O4v-;wGe0L(<c%CCop1ndYd4l$1RikA3uK(p;O;JEcOB<sFtF|SZh2Lr+*6Rqqt$PLt1C8K7-gCWQ@yqWfUP(Y>${xlw#6&~c2+OgUrj#oFGkUC;JCyf+BZwxob;Y*Mape$W+(md@rfSN5Ye@09b#rnIlsO#puh\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_100000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Index Name": "json_ste_vec_small_encrypted_100000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 100000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 19741.54
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 2.52
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_1000000_field_eq_idx on json_ste_vec_small_encrypted_1000000
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
          "Alias": "json_ste_vec_small_encrypted_1000000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbJeS~GW(rh_Ip#L(=Tk4TQhWW<9ljWmSq7yFQK{3){*^3mrvZVb*R$1K>{#a~UP>9qrQVA&6A(*plc?p!)G6K6VxXRF-Ao<%8ZuHeK?nJ3--0Nm!1=ln2Xui4i>S)gYG;I(F^bd2`!ND6isp!LMOUBn>CRQ!LHS7`%z%ZEdL=*l4D?dx-|<xS-To1jdGfEmP~zy\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_1000000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Index Name": "json_ste_vec_small_encrypted_1000000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1002404,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.68,
          "Total Cost": 187629.75
        }
      ],
      "Startup Cost": 0.68,
      "Total Cost": 2.55
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_10000000_field_eq_idx on json_ste_vec_small_encrypted_10000000
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
          "Alias": "json_ste_vec_small_encrypted_10000000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbJLhWn-J_XuAF@lJV1yj2RsW<G@;SoGJ^1z{wePh$|Pa&~|P6&(iAjnLjV?%U*uGw-s<T3ooXc-G07xL}j6&H#JO#ij$#MPRB#D8@rmF0_>Y00Q2F%S?!W`})*vltb>7kZ*|BK4On8t^sSE)U-JiRV)L<Ap5Fa3feN+iXv@<@H0YaMqt6p@vX2W;eVQs-4qb+fS|w\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_10000000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Index Name": "json_ste_vec_small_encrypted_10000000_field_eq_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 9986163,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.69,
          "Total Cost": 1876261.54
        }
      ],
      "Startup Cost": 0.69,
      "Total Cost": 2.57
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

- 10,000: `json_ste_vec_small_encrypted_10000_field_order_idx`
- 100,000: `json_ste_vec_small_encrypted_100000_field_order_idx`
- 1,000,000: `json_ste_vec_small_encrypted_1000000_field_order_idx`
- 10,000,000: `json_ste_vec_small_encrypted_10000000_field_order_idx`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 324.65μs | N/A |
| 100,000 | 10 | 315.29μs | N/A |
| 1,000,000 | 10 | 367.56μs | N/A |
| 10,000,000 | 10 | 367.54μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_10000_field_order_idx on json_ste_vec_small_encrypted_10000
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
          "Alias": "json_ste_vec_small_encrypted_10000",
          "Async Capable": false,
          "Index Name": "json_ste_vec_small_encrypted_10000_field_order_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10000,
          "Plan Width": 36,
          "Relation Name": "json_ste_vec_small_encrypted_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.41,
          "Total Cost": 13014.19
        }
      ],
      "Startup Cost": 0.41,
      "Total Cost": 13.42
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_100000_field_order_idx on json_ste_vec_small_encrypted_100000
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
          "Alias": "json_ste_vec_small_encrypted_100000",
          "Async Capable": false,
          "Index Name": "json_ste_vec_small_encrypted_100000_field_order_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 100000,
          "Plan Width": 36,
          "Relation Name": "json_ste_vec_small_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.42,
          "Total Cost": 127618.52
        }
      ],
      "Startup Cost": 0.42,
      "Total Cost": 13.18
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_1000000_field_order_idx on json_ste_vec_small_encrypted_1000000
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
          "Alias": "json_ste_vec_small_encrypted_1000000",
          "Async Capable": false,
          "Index Name": "json_ste_vec_small_encrypted_1000000_field_order_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1002404,
          "Plan Width": 36,
          "Relation Name": "json_ste_vec_small_encrypted_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.55,
          "Total Cost": 1238292.08
        }
      ],
      "Startup Cost": 0.55,
      "Total Cost": 12.9
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_10000000_field_order_idx on json_ste_vec_small_encrypted_10000000
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
          "Alias": "json_ste_vec_small_encrypted_10000000",
          "Async Capable": false,
          "Index Name": "json_ste_vec_small_encrypted_10000000_field_order_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 9986163,
          "Plan Width": 36,
          "Relation Name": "json_ste_vec_small_encrypted_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.56,
          "Total Cost": 12366683.86
        }
      ],
      "Startup Cost": 0.56,
      "Total Cost": 12.94
    }
  }
]
```

</details>

![Query Performance - JSON/field_order/functional](query_json_field_order_functional_chart.png)

