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
| 10,000 | 1 | 656.52μs | N/A |
| 100,000 | 1 | 645.65μs | N/A |
| 1,000,000 | 1 | 676.06μs | N/A |
| 10,000,000 | 1 | 6.82ms | N/A |

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
              "Index Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db5d403405be41b6281799edd00e30a94f9e9193354b0e6ba3baf2c6af614ac5a368c1c84c1dd7af44964ae5c37\\\"}\",\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\",\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0984d495728fc59c3d4cdb0afb3f6743cd38d571229b0cabe1724da310a9443188a37ee3d9bf6fccd071ba8e87bbfca1a30653d0347ce9\\\"}\",\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db5d4045f7d44340b1cc862fbaad02a5399801314b886b61e5022bf8ab286152eae9a107fb5132ae2356805eed63123651b59cdb8812aff8de1adfb9296\\\"}\",\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e403ea400059d16db41156f63a81f7cb843b48288bd2ca84ed4480023da19df02dfc310ca9ddd879e882faabad5da91611763355a824d2a1f7541f467c3e6303932d9295985e269a17511171a8dcc2a9a55e4f879c3d102b840471f234b35d626db5be91ae0cc42e92ebdbe2554f22cd822950484d05c4606227948b7e8c9a671e43d180f8cdc5202cf1e2ab22129c039ffa51a1d60ca1ec6703da6c0a2a0e557955b7fd525\\\"}\"}'::jsonb[])",
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
          "Recheck Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db5d403405be41b6281799edd00e30a94f9e9193354b0e6ba3baf2c6af614ac5a368c1c84c1dd7af44964ae5c37\\\"}\",\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\",\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0984d495728fc59c3d4cdb0afb3f6743cd38d571229b0cabe1724da310a9443188a37ee3d9bf6fccd071ba8e87bbfca1a30653d0347ce9\\\"}\",\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db5d4045f7d44340b1cc862fbaad02a5399801314b886b61e5022bf8ab286152eae9a107fb5132ae2356805eed63123651b59cdb8812aff8de1adfb9296\\\"}\",\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e403ea400059d16db41156f63a81f7cb843b48288bd2ca84ed4480023da19df02dfc310ca9ddd879e882faabad5da91611763355a824d2a1f7541f467c3e6303932d9295985e269a17511171a8dcc2a9a55e4f879c3d102b840471f234b35d626db5be91ae0cc42e92ebdbe2554f22cd822950484d05c4606227948b7e8c9a671e43d180f8cdc5202cf1e2ab22129c039ffa51a1d60ca1ec6703da6c0a2a0e557955b7fd525\\\"}\"}'::jsonb[])",
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
              "Index Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e403ea3bf3e70b0c1d4fee40ed753c4721d387e6366e17790ee3dcceb9b405cc3a7626f103937179f717f2318739be8f2324ed6df930789ed12a3780257d23fd3e09f2f54783f60623a46b146cb2db20edeef9cae784a07a862ddef19609c8bab7ce20583608757c08e7eae49cce2c6173fb30818a9802173c5d7e52950065099c954e60965007c77853ab7711fd562c81acd531c7f664b29b46066e02dd277b071de52815f79df308339ec66dadbee31b9b0aa33d1844eee59eb5a00a9031cfdfb218d2c0ae71295740c447d57a3ad8dc3491d6ec270dadfdbd3eb58561be183766064be7d\\\"}\",\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0a3129231c37571881cb77fb3c9d751bb8b49b4d14f537a23c3e4e9b7319426c4c40fc9162bc835b0f4cb7c036e2bc2cbc39d6e9d8f70d\\\"}\",\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db49711c282c994501ae344019314a8944c8416417f0886fba295d9084472d37e9f101d8b58f11e9acfa357fab1a9a15fe5155f76de\\\"}\",\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db4984978a08a03d61cf4490a4e3f3557c4d1c6cd2cdb1c230af94ceec60ca6509c14eb1334393241f979a4aa7fd93f22939a55dc53916bd6808053c2ae\\\"}\",\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\"}'::jsonb[])",
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
          "Recheck Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e403ea3bf3e70b0c1d4fee40ed753c4721d387e6366e17790ee3dcceb9b405cc3a7626f103937179f717f2318739be8f2324ed6df930789ed12a3780257d23fd3e09f2f54783f60623a46b146cb2db20edeef9cae784a07a862ddef19609c8bab7ce20583608757c08e7eae49cce2c6173fb30818a9802173c5d7e52950065099c954e60965007c77853ab7711fd562c81acd531c7f664b29b46066e02dd277b071de52815f79df308339ec66dadbee31b9b0aa33d1844eee59eb5a00a9031cfdfb218d2c0ae71295740c447d57a3ad8dc3491d6ec270dadfdbd3eb58561be183766064be7d\\\"}\",\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0a3129231c37571881cb77fb3c9d751bb8b49b4d14f537a23c3e4e9b7319426c4c40fc9162bc835b0f4cb7c036e2bc2cbc39d6e9d8f70d\\\"}\",\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db49711c282c994501ae344019314a8944c8416417f0886fba295d9084472d37e9f101d8b58f11e9acfa357fab1a9a15fe5155f76de\\\"}\",\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db4984978a08a03d61cf4490a4e3f3557c4d1c6cd2cdb1c230af94ceec60ca6509c14eb1334393241f979a4aa7fd93f22939a55dc53916bd6808053c2ae\\\"}\",\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\"}'::jsonb[])",
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
              "Index Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0a31280642a2229a3aecc4a2654601c1b001eeab4bfdef1dbb73cb26108cac8afe910a62c7fd4c62b0aca2ec020660f9b674c4cc63338f\\\"}\",\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\",\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db49711c282c9945019a0d5d828669b07666c2283603c56362cb07c395755fd5b70463afcdf62e729bf0720901dc833eb2598fc2d30695311e9b6808878\\\"}\",\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e4113e9f962fbe6fe249a5de0f12ba7ed57fbb827b92512c89c85a8e7547b64d9afb98dcf225ec8a0a15dfec7f6cb105cbad25ccfaeecffbca7ebd49b28c326a228fa6adf01a8be94bbce35bf50c67a30b32d035b22bef36839b79c8de4c7c739e0c53abfdf4f0ca5aeabc2656b12ac8221bea884cce06446907876dd5205dba1a3558d9ba94c14aa0979ff3f8fe2eb4847c3e4d35881773c86a6617b94cc541d2c76e514b37c2158a1eb5547b9ba51d0189e07d46aad2ba4499f19ba02fe08e3af94ab7b738c8c7c7b75d5e4830dd94e56e0e1475ea17ed946b7198391c3aa343958b777da5edb0fbf62e998e0a54d9f965f2f8c7d23dcfbc460112b93\\\"}\",\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db49711c281dcb2bf47fbf7c225980c05de594118306d69b6c184e1cc2ad34bedbe3dcffb4fe4af5e8bb70430838ebceac45f95cd0fd5a28c077149e166\\\"}\"}'::jsonb[])",
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
          "Recheck Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0a31280642a2229a3aecc4a2654601c1b001eeab4bfdef1dbb73cb26108cac8afe910a62c7fd4c62b0aca2ec020660f9b674c4cc63338f\\\"}\",\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\",\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db49711c282c9945019a0d5d828669b07666c2283603c56362cb07c395755fd5b70463afcdf62e729bf0720901dc833eb2598fc2d30695311e9b6808878\\\"}\",\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e4113e9f962fbe6fe249a5de0f12ba7ed57fbb827b92512c89c85a8e7547b64d9afb98dcf225ec8a0a15dfec7f6cb105cbad25ccfaeecffbca7ebd49b28c326a228fa6adf01a8be94bbce35bf50c67a30b32d035b22bef36839b79c8de4c7c739e0c53abfdf4f0ca5aeabc2656b12ac8221bea884cce06446907876dd5205dba1a3558d9ba94c14aa0979ff3f8fe2eb4847c3e4d35881773c86a6617b94cc541d2c76e514b37c2158a1eb5547b9ba51d0189e07d46aad2ba4499f19ba02fe08e3af94ab7b738c8c7c7b75d5e4830dd94e56e0e1475ea17ed946b7198391c3aa343958b777da5edb0fbf62e998e0a54d9f965f2f8c7d23dcfbc460112b93\\\"}\",\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db49711c281dcb2bf47fbf7c225980c05de594118306d69b6c184e1cc2ad34bedbe3dcffb4fe4af5e8bb70430838ebceac45f95cd0fd5a28c077149e166\\\"}\"}'::jsonb[])",
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
              "Index Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\",\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db4984978a142fec9212464150df63ead0019c5aea95d18cae26444a3adb52323f334c27b5160748995b0c439d9\\\"}\",\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0a3129231b4307797b00191560318039c584de487ee74f560d4b1072eabd3976a769916eb6f153159f2ccbade20db5df0a850887f249e5\\\"}\",\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e403ea401d9cfd4da45af6657561c1674a194da130bbf07340f326282a2f70ef712d546d62de403a3f09bb6ef9431f3a41e4926baf42eda50714491bc1daf441fb57b2dd418faab4eaebc35fbe3465196f4897f3d46f158f1e288a934e6d457e4f1b8f96f53559ef10aa975a21c24f94efe82373e774ea195217c167c4dcef249369b97c08f9b7265aae6291afd2265e1a638a9a152989a1e6fad7615bf9f631787a4294c451451fd011f0032b065920f66e11d42e398b15877cad0045ed8fc676c5a48957b505d9e356a6c897d633d3b3b9a923cb1aa1167546acc9940\\\"}\",\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db49711c141b2cd54aca1d3dc8a37a02a1417721ffad5915de66f57f85d0a6b7fd780bbf1436fdcbe694fe012bb66b7e7d3749dfc97\\\"}\"}'::jsonb[])",
              "Index Name": "json_ste_vec_small_encrypted_10000000_jsonb_array_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 1412.04
            }
          ],
          "Recheck Cond": "(eql_v2.jsonb_array(value) @> '{\"{\\\"s\\\": \\\"cfef1685f0e51a1fe1da955e7ad10044\\\", \\\"hm\\\": \\\"fb73e0f40bc991cf4b1090d0801f6447\\\"}\",\"{\\\"s\\\": \\\"2bab9d9c2aa600f519eb82a8ac3b7cdb\\\", \\\"oc\\\": \\\"8d5c371db4984978a142fec9212464150df63ead0019c5aea95d18cae26444a3adb52323f334c27b5160748995b0c439d9\\\"}\",\"{\\\"s\\\": \\\"9a2d817b8ec7abe623a1fcb4d9681003\\\", \\\"oc\\\": \\\"8c01e097b7b04c9c7bfc0a3129231b4307797b00191560318039c584de487ee74f560d4b1072eabd3976a769916eb6f153159f2ccbade20db5df0a850887f249e5\\\"}\",\"{\\\"s\\\": \\\"0342d803ea283195499ef8b163ee9a3f\\\", \\\"oc\\\": \\\"8d5c371e403ea401d9cfd4da45af6657561c1674a194da130bbf07340f326282a2f70ef712d546d62de403a3f09bb6ef9431f3a41e4926baf42eda50714491bc1daf441fb57b2dd418faab4eaebc35fbe3465196f4897f3d46f158f1e288a934e6d457e4f1b8f96f53559ef10aa975a21c24f94efe82373e774ea195217c167c4dcef249369b97c08f9b7265aae6291afd2265e1a638a9a152989a1e6fad7615bf9f631787a4294c451451fd011f0032b065920f66e11d42e398b15877cad0045ed8fc676c5a48957b505d9e356a6c897d633d3b3b9a923cb1aa1167546acc9940\\\"}\",\"{\\\"s\\\": \\\"746e042de28c05e98d1ff821a43d52b5\\\", \\\"oc\\\": \\\"8d5c371db49711c141b2cd54aca1d3dc8a37a02a1417721ffad5915de66f57f85d0a6b7fd780bbf1436fdcbe694fe012bb66b7e7d3749dfc97\\\"}\"}'::jsonb[])",
          "Relation Name": "json_ste_vec_small_encrypted_10000000",
          "Startup Cost": 1412.04,
          "Total Cost": 1416.3
        }
      ],
      "Startup Cost": 1412.04,
      "Total Cost": 1416.3
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

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 876.44μs | N/A |
| 100,000 | 10 | 818.53μs | N/A |
| 1,000,000 | 10 | 828.38μs | N/A |
| 10,000,000 | 10 | 1.07ms | N/A |

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
          "Filter": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbL@G`;sJm|S-`?Cwj>xA*wOX1h2r!q*wIn&*V1rcv`oiXv~-%&DI4^GK~Y{DbaJ%hB4I=(4Z`P>ZW^-VIB@?KxQN7!INY%@pi<nAf@qb^h-we!b(la49cObr=Um?~bZ>3PxY3!`sl!FRy)enoy#S5*H7|APH0?>!qTs*di3Ee>-+VPwF}!3HTUOacd@Wluel<cc8!\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_10000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 7744.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 1548.8
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
          "Filter": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbJ_R~jd|H&w|RXh&htXQ^z&Z+yTsZISaZ%?ReCB~KB+Urapc9}h&f#Dw(Xc7FT&eecbfQ3{tBvu^E)Q_(|2H3%0(Tq<1BfGbP6<s@eDQ0<@GN?&P&%Mnr{2(2bu6U7Mmi44k?&fxNq9qj4>A~-EN-|!lmzBO#p_0~Ch#2}dpc%3&{;m<w7NU-VPJ%T<?=Q1dkXelzxT>`OYpaY=5\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_100000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 500,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77390.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 1547.8
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
          "Filter": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbLvX)6;+mT}NuDf52N=XjpPbB%Lqu+E5rN(5YFX~dM-@;6C=yjIaXcYA8sQ&XUhMz|SntOC9LIWg8lh(~O4v}g*@@V6406|l!StPJtUXo{gxtN`&Cpxf!OyzNr8AO4>@_~b*?X4luki<7a_T~Aww5d`O<u}|v^CA0UI3QOC?#30mN`EeJ)hCJ@8Sw{%ST}XhPygW`DkbumVBlZV0l`EjY\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_1000000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 773893.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 1547.79
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
          "Filter": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbKbog(r(h<~in$X~uUNvV{?Y+$nk2-)8<GX^xWNhXV1|1AtbZ_$(I7BD=qd9%tq)aqAh`xjfnr1jQ&J{$Xa?E)iS11ekJ=_(y9GU|v8%*#b5Z}Oa~i-k7h{bg%HN=l~<t29<3-nMg>RA!pBIIK4~JQlZL<GI?Uqr@P~iuEYKrOUhWFeu;`M(5QPr=v+2x9Ypb?BIG22c7_+zy\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_10000000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49950,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7731165.49
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 1547.78
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

- 10,000: `json_ste_vec_small_encrypted_10000_hmac_terms_index`
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 1.45ms | N/A |
| 100,000 | 10 | 7.72ms | N/A |
| 1,000,000 | 10 | 612.43μs | N/A |
| 10,000,000 | 10 | 749.24μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Bitmap Heap Scan on json_ste_vec_small_encrypted_10000
    Bitmap Index Scan using json_ste_vec_small_encrypted_10000_hmac_terms_index
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
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 100,
          "Plan Width": 4,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "(eql_v2.hmac_256_terms(value) @> '[{\"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}]'::jsonb)",
              "Index Name": "json_ste_vec_small_encrypted_10000_hmac_terms_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 100,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 67.55
            }
          ],
          "Recheck Cond": "(eql_v2.hmac_256_terms(value) @> '[{\"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}]'::jsonb)",
          "Relation Name": "json_ste_vec_small_encrypted_10000",
          "Startup Cost": 67.58,
          "Total Cost": 191.91
        }
      ],
      "Startup Cost": 67.58,
      "Total Cost": 80.01
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
          "Filter": "(eql_v2.hmac_256_terms(value) @> '[{\"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}]'::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 27140.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 271.4
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
          "Filter": "(eql_v2.hmac_256_terms(value) @> '[{\"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}]'::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 271393.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 271.39
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
          "Filter": "(eql_v2.hmac_256_terms(value) @> '[{\"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}]'::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 9990066,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 2711157.33
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 2.71
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

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 982.98μs | N/A |
| 100,000 | 10 | 984.25μs | N/A |
| 1,000,000 | 10 | 904.07μs | N/A |
| 10,000,000 | 10 | 916.01μs | N/A |

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
          "Filter": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbL@G`;sJm|S-`?Cwj>xA*wOX1h2r!q*wIn&*V1rcv`oiXv~-%&DI4^GK~Y{DbaJ%hB4I=(4Z`P>ZW^-VIB@?KxQN7!INY%@pi<nAf@qb^h-we!b(la49cObr=Um?~bZ>3PxY3!`sl!FRy)enoy#S5*H7|APH0?>!qTs*di3Ee>-+VPwF}!3HTUOacd@Wluel<cc8!\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_10000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 7744.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 1548.8
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
          "Filter": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbJ_R~jd|H&w|RXh&htXQ^z&Z+yTsZISaZ%?ReCB~KB+Urapc9}h&f#Dw(Xc7FT&eecbfQ3{tBvu^E)Q_(|2H3%0(Tq<1BfGbP6<s@eDQ0<@GN?&P&%Mnr{2(2bu6U7Mmi44k?&fxNq9qj4>A~-EN-|!lmzBO#p_0~Ch#2}dpc%3&{;m<w7NU-VPJ%T<?=Q1dkXelzxT>`OYpaY=5\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_100000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 500,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77390.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 1547.8
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
          "Filter": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbLvX)6;+mT}NuDf52N=XjpPbB%Lqu+E5rN(5YFX~dM-@;6C=yjIaXcYA8sQ&XUhMz|SntOC9LIWg8lh(~O4v}g*@@V6406|l!StPJtUXo{gxtN`&Cpxf!OyzNr8AO4>@_~b*?X4luki<7a_T~Aww5d`O<u}|v^CA0UI3QOC?#30mN`EeJ)hCJ@8Sw{%ST}XhPygW`DkbumVBlZV0l`EjY\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_1000000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5000,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 773893.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 1547.79
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
          "Filter": "(eql_v2.eq_term((value -> 'cfef1685f0e51a1fe1da955e7ad10044'::text)) = eql_v2.eq_term(('{\"a\": false, \"c\": \"mBbKbog(r(h<~in$X~uUNvV{?Y+$nk2-)8<GX^xWNhXV1|1AtbZ_$(I7BD=qd9%tq)aqAh`xjfnr1jQ&J{$Xa?E)iS11ekJ=_(y9GU|v8%*#b5Z}Oa~i-k7h{bg%HN=l~<t29<3-nMg>RA!pBIIK4~JQlZL<GI?Uqr@P~iuEYKrOUhWFeu;`M(5QPr=v+2x9Ypb?BIG22c7_+zy\", \"i\": {\"c\": \"value\", \"t\": \"json_ste_vec_small_encrypted_10000000\"}, \"s\": \"cfef1685f0e51a1fe1da955e7ad10044\", \"v\": 2, \"hm\": \"fb73e0f40bc991cf4b1090d0801f6447\"}'::jsonb)::eql_v2.ste_vec_entry))",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49950,
          "Plan Width": 4,
          "Relation Name": "json_ste_vec_small_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7731165.49
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 1547.78
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

- 10,000: `json_ste_vec_small_encrypted_10000_ore_cllw_idx`
- 100,000: `json_ste_vec_small_encrypted_100000_ore_cllw_idx`
- 1,000,000: `json_ste_vec_small_encrypted_1000000_ore_cllw_idx`
- 10,000,000: `json_ste_vec_small_encrypted_10000000_ore_cllw_idx`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 742.44μs | N/A |
| 100,000 | 10 | 773.53μs | N/A |
| 1,000,000 | 10 | 767.52μs | N/A |
| 10,000,000 | 10 | 840.93μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_10000_ore_cllw_idx on json_ste_vec_small_encrypted_10000
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
          "Index Name": "json_ste_vec_small_encrypted_10000_ore_cllw_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10000,
          "Plan Width": 36,
          "Relation Name": "json_ste_vec_small_encrypted_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.41,
          "Total Cost": 6094.41
        }
      ],
      "Startup Cost": 0.41,
      "Total Cost": 6.5
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_100000_ore_cllw_idx on json_ste_vec_small_encrypted_100000
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
          "Index Name": "json_ste_vec_small_encrypted_100000_ore_cllw_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 100000,
          "Plan Width": 36,
          "Relation Name": "json_ste_vec_small_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.42,
          "Total Cost": 60652.42
        }
      ],
      "Startup Cost": 0.42,
      "Total Cost": 6.48
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_1000000_ore_cllw_idx on json_ste_vec_small_encrypted_1000000
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
          "Index Name": "json_ste_vec_small_encrypted_1000000_ore_cllw_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1000000,
          "Plan Width": 36,
          "Relation Name": "json_ste_vec_small_encrypted_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.55,
          "Total Cost": 606432.55
        }
      ],
      "Startup Cost": 0.55,
      "Total Cost": 6.61
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using json_ste_vec_small_encrypted_10000000_ore_cllw_idx on json_ste_vec_small_encrypted_10000000
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
          "Index Name": "json_ste_vec_small_encrypted_10000000_ore_cllw_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 9990066,
          "Plan Width": 36,
          "Relation Name": "json_ste_vec_small_encrypted_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.56,
          "Total Cost": 6043897.25
        }
      ],
      "Startup Cost": 0.56,
      "Total Cost": 6.61
    }
  }
]
```

</details>

![Query Performance - JSON/field_order/functional](query_json_field_order_functional_chart.png)

