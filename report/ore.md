# ORE Queries

[← Back to overview](./BENCHMARK_REPORT.md)

Per-tier query performance. Each scenario lists its SQL, the indexes available on the target table, the indexes the planner actually picked per tier, the timing table, and the full EXPLAIN plan in a collapsed block.

## range_gt_10

**Description:** Range query (greater than) returning 10 results

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 10
```

**Parameter:** `5000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. The bare-form `<` / `>` operators inline to `eql_v2.ore_block_u64_8_256(a) op eql_v2.ore_block_u64_8_256(b)` post-2.3, so the index engages without query rewriting. Query: WHERE value > 5000 LIMIT 10.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 1.24ms | 28.22ms |
| 100,000 | 10 | 1.38ms | 29.37ms |
| 1,000,000 | 10 | 1.15ms | 28.73ms |
| 10,000,000 | 10 | 1.44ms | 28.25ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Seq Scan on integer_encrypted_10000
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
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bcb371609a9d6c573eed7e891d65e32763a0777fe30e584652309016ccaf8c0b4e4101f904e3cbeedebd563ec8f9bba67000d22aecbe5b547dd4e291e7a913ab7de63374518292d0046f8248fbf7c2abe5e8d96c0c07f4006b44f6b28fc569cee4c70913e19a951f1b30c21047b44bc16ea55ffaef4d32b98f01a0e0b24286638e29b77706c07bd44000b951b5976af1bbc9665f7e342ada753179ccd9003f14119b0199d54835f99e140f5e4431a7524fff57064614978f575978d4ba7c81a776ecd4f632e402d63d7fa4be00559e25b737cae53bb7ebd547de3bf796fa7823a3aebb321a191d501068035259eeec5cb805e3cf3d651f159698f511a3394b871299d5275978baf403a46ae93a636b33cd\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4950,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 7766.5
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 15.69
    }
  }
]
```

**100,000 rows**

```
Limit
  Seq Scan on integer_encrypted_100000
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
          "Alias": "integer_encrypted_100000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bca47ad7534549731c44e6f8ae43f935528296fe8d3533fe139de063f95576d9b55dbdda14c67c41e5fba98e168b989ff6d69ab07c4facafffc88a9c07d08b03a392cd10c4eddcc204656c32fe8322644e9d1668a79977496654e3679ea13c53bc94a0d230f1011e831d43ddf27a31c39e12508fbd608bb1dc684c649dd97bf573aeb0a5078fe6e72c00230490ed497305581711594898b52e4ad8ad040692f613b8594678f4c2ae77b9b2e64f691626ed188e9045730c0df6031ffdd36fd3ca6d212874284261cc54a165fc2d832ebf4c5e66a2387dc40e05f258034a674550b1b9e9ed526c8cdc80ae6883d36a103a05a3ced71df03b5a404a40f752d0524e97eb2e09f5399b6d4fad3034fcdefcef8d\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50500,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77911.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 15.43
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Seq Scan on integer_encrypted_1000000
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
          "Alias": "integer_encrypted_1000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bc035addeee789fb2e823f5127d1ea3eb2abec3e929c3f195e7794ec84783ac99293e9193130a55e1b1410f10b6329d3c464483505f17295f734aefb6e1337d2b9c98fdaf8a235d84ef4ff1960efc83ff09e1bd8f63dd6faaab9b80ccd1384130d8cf4ac09c2fa90bb31be4e1ddebe36774abd33f2d9c2e51eeaed60f34c42a440d7aa1747824c7baa7b9e8b070ad454113251385cf58966ca6a198532ac2cb83f883562dbd945ba80181f5ab2bddbdfdc35957e867f11ba1f413f9d34594bd08be25ec5a8691863d709c0b1bd322e54fa37a4aa7b1397ccbb7e83bfd3fcc86581a771819df23434b5fd4d43ec3b5d7199c5b80d404129c2e570861640f43ed28a0c970002f9a35e1369df809d1309d87e\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 736191.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 22.09
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Seq Scan on integer_encrypted_10000000
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
          "Alias": "integer_encrypted_10000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bce9f7ed622e356cf9c68940a09e4fa6c7f53e37fd1a20685f2a2a72ae6d7919eab25bcf13844f317689fd847005d206ec1816218ec9a2dcb8e446dcf2a0eaed8de0d485642d9c06eaf1387f6430753d5d0435e7f068d641d168118ba14c382047edb764caa4062c1d3d353c202f15b1f45d74b553b3a74b49d2d75ed36d6f31c1f349913841fe6a723724c9efb37675e297f1ad661829c03080963461a8413f5a45b1e535f4bcb9ee04e9863c1917c32e5122fa229ac0fd79e87119c2d386d6b82903c0afb914c6826c542f72db96910f3f52f5d7dd5cb377b9802a473b2d60c753b5ed34ead5511b8551bbcb77c7b69d4c46f685f891bf30c1880f3355a9ac89335779f285fc8a4d2730540f91cf1da7\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 3333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7361905.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 22.09
    }
  }
]
```

</details>

![Query Performance - ORE/range_gt_10](query_ore_range_gt_10_chart.png)

## range_gt_100

**Description:** Range query (greater than) returning 100 results

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 100
```

**Parameter:** `5000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value > 5000 LIMIT 100.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 100 | 6.52ms | 45.74ms |
| 100,000 | 100 | 6.56ms | 48.51ms |
| 1,000,000 | 100 | 6.85ms | 47.69ms |
| 10,000,000 | 100 | 7.12ms | 46.06ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Seq Scan on integer_encrypted_10000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bc846ca961e9c7d073e4b69bb55d2d389f16d58a6d5fbea545a848998537d5026fe275212f67f28a6157e1747db9eb445488d3e56ab96df32df2090cf3a11c95bf0beb228000a33380009da004dd79117bd926f4a8ac0d7c82898a8a3b73e31c229035b21f79d827929aabc6b37077d26b88a1edd8c4e6d5e0681558d1cbebd09ffec80d242137bd9f745ffc6b89fde4a526a0239da0cb97d29db6b2f95e9e0d3ce1ec8d22cc092c95b9453b291a802142c9b0e2162ddc7d9ae4eb3e27081699918005f53afdb84d5dbe2916b02c1dbb14f838f2811d0ec550f07e0ab51a3595556cea1ae203c1c9066cac83d049092fef5912a7d22dc6a89818a0581d00bc975f4870d3f0412c3d9c7fad7d138855e56c\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4950,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 7766.5
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 156.9
    }
  }
]
```

**100,000 rows**

```
Limit
  Seq Scan on integer_encrypted_100000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_100000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bcc8d0e399915c1409fbe0693675ccdf11458ba4640908d8197d60f34cf19ad8ae8795552ae5ac3976d25075691f279e335e9abcff0102d9d2ace321522e17591b35726bca2c1efa066d734337789854fd221d52799ebc7842fd15d7fc500f27027ce0cf3cd9273fa6a217ffc95881800ab06d6c75adf5b21cbf8b7ba6f8e962d79ae0b9fda4729a8f027c1b0f0651935bc5639fc10d3c3b5230824c6178cc45fecd0bbf22dbd5268049788ab40b3b5a334d6e81d475617b8cf23046eb1068fe3e115047cef9ef39f4e34278fd33f05b6cb875bdd160d57a0061bd8ebaacf429f9a96fafcd18d2a901ab3b987a57147a3bd39d0e5c3a9bb87a75e24ef2ad1c006eb08af175e138160fad88f898f057d696\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50500,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77911.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 154.28
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Seq Scan on integer_encrypted_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_1000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bcabf433c453472e46375cfb9e11c18e0e3f87e0245b92ddf5108247cb9915ec904903ad62305a9920d42b6890aaebccbc85fc49a32976d85be76bd6c90134b27d4444f4a53289bc92fecfdff9804d948dabc7c704e9f45fe89c47b81710a4fc20f6f274a24618cb01bc9fc89ab94fb800c232512f1fe71f2b607e83af896d4addbbea739c4552e5b465ac7792a211caae25294f851a90ac7aca8391001d9c65b8fb3853c2942b7ec27cd30f5731b657d56e7061d0fd71ad3348733ad44cb48607536f1f9c8fe94a540c405bcc1abfbe88f58171c51a06f1a8e6917a7a5087b5eba87d49f194bb76656a9fde0bce63c2d40df25f1eb5fade7feddbd8c04ebd7776e92ec8e7a769ffe9a5c80e4e63a81412\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 736191.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 220.86
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Seq Scan on integer_encrypted_10000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bcaa4cb850266e2f346f8fea794a1917aa7ea9eee7fede9bb46574390fb91029c306c2863f3160b1d50a1c22209038b95afeceab30c37721d6be0b1a58a93bcb76bcbe32bb1aefd0644c058db25ca04e297e4d068a0b8154770a4df0950b96dd0b7c15c69de74428d561f73ebb11c691418b8751e55d5e175eeddc212dfd1565fee086c748f11988efad13c1cb0847b010da5b7306c8541003e226ff21554299470f0d33e560263c005d1086470860a48208c87ad25db365bcc4f196b76bb5b392c6745177154bd309c0f9dbf69da7910ab86af23d190a66afe6e5d6c184df20b7c5f4d03ba7c948fa47e31377be3fe4bfe46ac3932a492ae9eed49d131b605e5da48d3390054fd79722aff6a220503c14\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 3333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7361905.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 220.86
    }
  }
]
```

</details>

![Query Performance - ORE/range_gt_100](query_ore_range_gt_100_chart.png)

## range_highly_selective_gt_10

**Description:** Highly selective range query (~0.011% selectivity) returning up to 10 results

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 10
```

**Parameter:** `2147000000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value > 2_147_000_000 LIMIT 10. The threshold sits 483k values below `i32::MAX`. Even at 10k rows the planner picks Index Scan: with 0.011% selectivity it expects ~1 matching row, which it finds at the top of the b-tree in a single page read. Useful as the upper-bound demonstration of how cheap a selective range lookup becomes when the functional index engages.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `integer_encrypted_10000_ore_index`
- 100,000: `integer_encrypted_100000_ore_index`
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 3 | 735.89μs | 26.70ms |
| 100,000 | 8 | 866.03μs | 27.93ms |
| 1,000,000 | 10 | 1.09ms | 29.18ms |
| 10,000,000 | 10 | ⚠️ 2.225s | ⚠️ 1.822s |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using integer_encrypted_10000_ore_index on integer_encrypted_10000
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
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c74696450f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12000c1ce2da49187443be7a630b7bba4e4c788d889b7eef636659fb4d6b64dbab777c3973caa72a76ec3c372074623e365e1efa586a135a120a71549033398f2cbce5b88891afc9d3a622967aa8d8dc53b113eaf91f7e0fe64b07e4809682e3b02d669a9ea612ff80f029011631431896c4512ad6391d90c7827c6faa85e075cb13d2b5674e86d899e6f3bd910ba29df139ba69d18cbf9ae9dfe04592c10d1fcad9a67ab809095c71f799a08037dfd15c2849c19a43d11c03a535751da2c46b581f8a33a8feb983049bf6468551c1f7de504a324f2655f7405b318fd2f8284c5d43ad52934069c2e69a7c9d0aa2e0116866002104f3fedf883ea021aafd8440838f441c1246ad022cfe7f46c402f36d88bc44059f55ecaf8b83a1166da49675be7aadb67a15af6ec4fa71ddfef80fb72c9e167c4f075207100662462a5ccaae1c\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_10000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 229.9
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 46.41
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using integer_encrypted_100000_ore_index on integer_encrypted_100000
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
          "Alias": "integer_encrypted_100000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c74696450f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12000c1ce2da49187443be7a630b7bba4e4c788d889b7eef636659fb4d6b64dbab777c3973caa72a76ec3c372074623e36d31ebf310f4f64f5edf80e48d5106acd24e0a932f1899d44451ec171617bf9462f3b3defdb109a9c8fdd011ecf6056ff152dea945a72395abcb7c3047ebecd7025b972bba9cac08526c182538e90b3968303422a68b4cfc5275cc59708420f3998c08e8a04db9784be0462bfac2daa18038c49e66523aa3300cc1adce2c2d6d5c9c658a3f5447ad97739ddde9a0fc196dd572f73ff74a4fb2774f1613afbf0665b8832dfbdd46f4cea035ad86dfae1bd64becab39499529c7281a1579080976002a94cc889e36c17232a74ee3b43fa84665959170b38495e31073f45698dcb54e08daddec7cf516627647098be4ffd4a52bc80d28b3d570fab7b6b2e4513c448e2b2513c966a52abd60cef02d033dd26\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_100000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 500,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.67,
          "Total Cost": 2258.41
        }
      ],
      "Startup Cost": 0.67,
      "Total Cost": 45.82
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Seq Scan on integer_encrypted_1000000
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
          "Alias": "integer_encrypted_1000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c74696450f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12000c1ce2da49187443be7a630b7bba4e4c788d889b7eef636659fb4d6b64dbab777c3973caa72a76ec3c372074623e362e90356e07af380dcec26319c745d1f2fed64b395086551b2c70af35183d7f698b2d5da79484f9e665ef27d45f2673fe0241c2e74a550eacd6cad81f62657082ec6c0ecb99acbb0b2594f42efca5e9d12d0a59e968716bc97bce90740302854fc4e5c4a509f1dfadeab14f35ceae0f198453666e9b5e2b4ecaceda7d074369b16e0a01b920fae9a8c4a0c4d743cf34e087a2d8f67f7f35d0ce89ee60290ce36132112bb3fb0162ecf5c2bebdc7c6adfb7bdb44d3afd4087837f34e699596e816e7cdefcfbb415f5d49044f7783a2b04e78994d0ffb764389ea1ed6e1c0f9fd615fd7683bc907f5782ceb86426df1600e840e931f4650a1564a11bf0d6376d3605a1380581345c978a908c837b71af201\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 736191.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 22.09
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Seq Scan on integer_encrypted_10000000
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
          "Alias": "integer_encrypted_10000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c74696450f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12000c1ce2da49187443be7a630b7bba4e4c788d889b7eef636659fb4d6b64dbab777c3973caa72a76ec3c372074623e362d7a510fc7d46008fe6067002171f9642fae8a000509fee2c127df0e97786422914d978ab9a267c42d6802b197cedd4559d1eb60928041eb30f70beeadee64deec4637be4a8efb1694de349958bec1bb971eae1fa61b8dc6e310804b3081824d4bcb368e19a464a5ea9e42bc2ea057960076d3adf73a65b2d3c9e9e85647a60642cb99df28a65ba53b4161aecb75493963c82402757169b0472e51253b8b505d7d10ba32c7ce5e036e8c3c794313cb63963d4d2ed31503d5ff23821e0a6df22caa1283805c306ba89e0bfee044ed2c9a274c230000dc215364beb1f904b2c4d11a096bd5c272f875742ebb122a8627346963f9f5fdb330379f6957e5b995ab947459e8bdd5b96d0fcc5df978bbd25b91\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 3333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7361905.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 22.09
    }
  }
]
```

</details>

![Query Performance - ORE/range_highly_selective_gt_10](query_ore_range_highly_selective_gt_10_chart.png)

## range_lt_10

**Description:** Range query (less than) returning 10 results

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 LIMIT 10
```

**Parameter:** `5000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value < 5000 LIMIT 10.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 1.46ms | 28.11ms |
| 100,000 | 10 | 1.19ms | 29.48ms |
| 1,000,000 | 10 | 989.54μs | 28.59ms |
| 10,000,000 | 10 | 1.39ms | 28.28ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Seq Scan on integer_encrypted_10000
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
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bc57f85ffccf8e70b0898a351a513249073689e3f752ccf5980e9a9f53238814cf822bfd393e5a011182187e00da33bb8e6816d22227c1bea13e524cbd9a9a680a44d56bcedb7f86701ed51a92e89385df2b1c75b0f37f923eb260624b5d284a0ffc7ea45fde7e851f983f4b08643d36fb7fcb1bf05d594a5a4ae8f78d65fa246681c47ea1923a30a28600082254cfb4a371204ff38164bfcb55e812a6827ed264429a54cb049ca7b40bd569a3ef78dedb9eca1ec189311e985499160fecd9b7a91388def686639bf0ab60f9ae38f5a6243f7ba78a20e0a12b482c5e4d2820140406f08d55a5d121fcbcb0e4b1ce0cb777e218d05d46ccbcf520504b84d2cf59a25e57c4dbeb6178d720ec23120460764d\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5049,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 7791.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 15.43
    }
  }
]
```

**100,000 rows**

```
Limit
  Seq Scan on integer_encrypted_100000
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
          "Alias": "integer_encrypted_100000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bcdcffe978b21683de0de0c34696ae1e7af3620b38927983ed81e4f0a065da02e5df334f1bd8a45773dcd6cde85981d31d68e57fbfd5ed1fe323c0d7927ddae1fec43c67f1613027cdff914a690c2a2ec1aab26220e32cfc55846614b8e3e400035ac430c1fedd962fc35c40bc56a064e9a5584fba907c54662db4c092da217fee9ea90c53009a4cdbdabcee3ecf0314f9b11a0dc17ec276ca15c9c5b605ad14b4e6ef140b436ec03c1f4e21131f1bf4ccef35cadbd3e34054ae2dab502bcec0c14ab401f134c0fad17bc2db1c36840e4a81a3140bd0ff75feee8a040f7bb6f48cb75154629e7ab32688a47b460cae90074ecc9740fde20a9ce14882b427996dfa631adde672809609a140b1d657de6213\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49499,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77660.75
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 15.69
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Seq Scan on integer_encrypted_1000000
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
          "Alias": "integer_encrypted_1000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bcfb185e1980702303c643f954653b34c7357713c723e207ed0348fa34b10abd7c02e289d963ba377b5c7efc4b21dc7c1b7c1461a37ac1123462fd512b9ffde212ffeb78136ed8e1145ea64fc75bf8d9b117ea36c05c23168c3c84a2df2149c38ba0e2faafd9aa349783678d28df4ecc90c977513fb33fe57dcdc8ad323fe71810665a52896d94d088912715b9fbf1d4fa0255f24c6bafd8ce9ec470b4284988eccef7f419e59d1dbc7f68f68a4fb28d0ee70c02726f6923f6196c8adb4d7cc881f926e5600d0cb867a2d852a558659762521e9b97aef8283ed807060737357bb04aff873a5830ca19431eb2304d736d1c12c4fbd453db24e5a381f3eb2dffd7ba3cd7dd444d7bb5319153602a5803220f\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 736191.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 22.09
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Seq Scan on integer_encrypted_10000000
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
          "Alias": "integer_encrypted_10000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bc8132d7923212529301fdc6ac7b34477530d11b3d7ec96fbb24491111827ba4a3c088470e675b26d0b3aa505dd61edb45c65b90ff960e05ebff9737d17c88cbb558c31143c25512d0b1377d41fa5efe8a134c4440068d78d886e67309efe82ea5e601688cc40ba279889879468b0c723cf3120005de8478891920aaf9d8def79d932a8304099244e4f99115dec9f0dfda776b98a8846acaa601597928cc375d2753118e357b12b8a445214de7611c1508a3cbaa2383db1c89b44e1f63133eefe866d93bdfdac4036b45f79dcef01cc656810b3b8ac238854d972f1893d8c1e0da2700f76f7255948f7293c67d933361fb089969df4efcef5339159b540a434b03bc923e2088967a68b93ce0958da998e6\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 3333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7361905.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 22.09
    }
  }
]
```

</details>

![Query Performance - ORE/range_lt_10](query_ore_range_lt_10_chart.png)

## range_lt_100

**Description:** Range query (less than) returning 100 results

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 LIMIT 100
```

**Parameter:** `5000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value < 5000 LIMIT 100.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 100 | 6.72ms | 46.04ms |
| 100,000 | 100 | 6.52ms | 47.22ms |
| 1,000,000 | 100 | 7.03ms | 48.06ms |
| 10,000,000 | 100 | 7.35ms | 46.83ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Seq Scan on integer_encrypted_10000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bce5c3c861e0d404bee8c16e5ebce826e0d8e85dd403f45a75081065aefadf5fc7c59b86264a4847de423f3625a82374ebf02cb1fb0b4b0001fc8d58ae7cf9a3fc6041ad3d2d4fdd03e1a6c86f9b23525ad50aa6911b529a79e5578e20381b28f95691a92f787294ee30604140fffe39a8cb6075aba4c7a2811c54806fadc353a79e9a9a6252965bcfbe5faa4ece57440ba74f2e58b7b6c49adc3e8cfffe43b1d3b0268747f275d14e68ffb3eca6a6590aea2bf07f64018b090e4944355fda0616d6e296e8042fcd4579f5568c5deeb407ac9a78f485a653c609e0a7d010933314ad2fdbc047a4b0057d99c082e15983941f75332acb480a95b683749cfebf77eb6503281f91e69eab8462513b6bc21ee2\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5049,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 7791.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 154.31
    }
  }
]
```

**100,000 rows**

```
Limit
  Seq Scan on integer_encrypted_100000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_100000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bc0fc2aba22d9169f6b28d24ca48b501dca3db2a5e16d50e4f7f14473c2138ce0768831cfe327fb1b6cf7f233d3073a6f71fcc86a1cea7478f892fc2039d2f98a4b3fde24cd6792d1d8379215fbeb931c41780d00691592a0d7b1b921b9a04101c70140d87534627f2a92c7fa32d36be3ea3da049bc5e5cf203af86eacf599a7b85e328dbb7d03c22a60556c07ab23547da67943a66e7c5211bce92361f2d2add82ccfb15e6113160936f5c77efd2ad3ad91a585ac53c9bd1d7ed93d359ab6bcaebca1c92b97ce6a9ab37ad67ca277d41c137b8a0cbc08dba59590bf723a804be8fa60762c91b8e9c035ef15484204d84d052e1340c9da3d002839f93015b48c3e50816eebc7f77825d62b76d5ed31fa3e\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49499,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77660.75
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 156.89
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Seq Scan on integer_encrypted_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_1000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bcf28f42cb49ccd75fa7e54ba418fcd991111d5f4129246ac88a77e5214e642a4d8e97c073f3af5c652c1a165ef4225398c7c94f18c903c437dd465304087d70187cd10a0cbb348a4edc9d7f90a724c418ac57d9cf92471feb1eebac3c604ec5da99a479e9795e1ac97c63e3403b53d8d2400bbdc1639920baa792384076900dbbabc7b6b325ec56694fed33e9cb5c7f3b40a91d4cf56964e266614447f25d4d0796485905d4b0879b044001bf16ba3a55ced53a94b0e00c756aa3dbbc5b260b64c1ce599612c69183dade17ea9fdd5ac44d7d5a4fee1348e425d2e7cae3460949f89f43ae5a195ecae00794c045006bd1bed6b3d094b0b25e3b5694af78aba0032c8554291e77e1860458e907cc5c2687\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 736191.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 220.86
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Seq Scan on integer_encrypted_10000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bcb7119eb60232106d51c55cf33e10845b9cab118f2dbd69bd1f8a51a71e69c708eb9ce74d7dc21d89173cc27449d218a91a171d8fd58eda154a787dd1bfa2a724778bf424e2c12e1114db4554bbb670d5fb7891bbde0ed2c974313108074d538713145b575cc0c169dc3a98e0f851c70b8326dc7ba511d564f54639564a2ff6e0f44baeacdfa25bc448f723529a07b966a0a301c32bb00f8c1e1295573f1cabff2137451e37ab5672bf0a862111155910682b8456ee9a9b9cfd6958353b73b6208e17dd215d207a796afb8790e07cdd9c5f92da803eac86a083bb3654de1638eab39c210ba3d70e2aea44bba80a2e4d33a82da6ba938116cefa072c21cad029eda93f4a7dc3d1daf7f4681a89a0ac7c44\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 3333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7361905.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 220.86
    }
  }
]
```

</details>

![Query Performance - ORE/range_lt_100](query_ore_range_lt_100_chart.png)

## range_lt_hybrid_ordered_10

**Description:** Ordered range query (hybrid form: natural WHERE, extractor ORDER BY)

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 ORDER BY eql_v2.ore_block_u64_8_256(value) LIMIT 10
```

**Parameter:** `5000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value < 5000 ORDER BY eql_v2.ore_block_u64_8_256(value) LIMIT 10. The sort key matches the index expression syntactically, so rows stream out of the index already ordered — no Sort node. See §4 of the EQL query-performance guide for the natural-form sort-key trap that this shape avoids.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `integer_encrypted_10000_ore_index`
- 100,000: `integer_encrypted_100000_ore_index`
- 1,000,000: `integer_encrypted_1000000_ore_index`
- 10,000,000: `integer_encrypted_10000000_ore_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 1.20ms | 28.08ms |
| 100,000 | 10 | 1.35ms | 30.54ms |
| 1,000,000 | 10 | 1.10ms | 28.18ms |
| 10,000,000 | 10 | 1.19ms | 29.87ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using integer_encrypted_10000_ore_index on integer_encrypted_10000
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
      "Plan Width": 68,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bca312a3e2e91346fccfa7e1e73602090d983d69dadc05e5db49676b273551a87fc355b31dc329804542f93d17c67a463498b75ff7ec9503249fac50bc3a8029b9440d50832ebf92d5c23ef249c4f08a7fccf67d8e82bc15a9ad391669869dba89b8a6c082544bc8a5a3095186eccf71494a2beec0115605b56a0603d26e2c5e2bd4de6f8c3e1b9f07bd4bb6ba6f972899977547dc254b31f019886c76ce8ac1aea86f7d397773651ea1a24c8ba34bf18d1ff9df0ae42bedee88abb323942e6b7b266dba24babd21e6f757dfa02ff5b88be00f15909ff72a17c74ced3f6b2376ff8ca7c50958342bead3f9a2dcaee0e108f1f465fc354acc8f5cd4a09902cc52707da38f7311e51ef77d82ea93de2537ed\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_10000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5049,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 9909.19
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 20.16
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using integer_encrypted_100000_ore_index on integer_encrypted_100000
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
      "Plan Width": 68,
      "Plans": [
        {
          "Alias": "integer_encrypted_100000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bc32fb06f4dc0ab36ab1c28b25ebd021e08aab8952a40ba32ffb2c8a1e417cd771d03133c8160da2c0efaa4a739ea7f4457b87052666b3f7e18d82b85be132e368d47a2da4af0dab2a70887fafb9b37924f239ce2639d8de1171faf4fd661c3501a4d1d9cc66f9a8767d019a189870d99127441e4956b25e60f159e5e50d6c363943d096aa7e476ce737393d8344b703a0aee037c9718fe7fa9ec646833b65f67102290d0cd8c6d444b823fe27f4edc44652fb3f3c3f42e88acf5c01ffdbb8b38a1c91da2b65db407f6337d5ed460fa8a76499f883082b0c7dff04b17443c3c257734e1c7b26e782b6e3c7b8effac4da10fa0996ecf03f1aaa2bd2663ebf2814cbf57504215241512f4c07c27e1044c377\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_100000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49499,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.67,
          "Total Cost": 98196.07
        }
      ],
      "Startup Cost": 0.67,
      "Total Cost": 20.51
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using integer_encrypted_1000000_ore_index on integer_encrypted_1000000
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
      "Plan Width": 68,
      "Plans": [
        {
          "Alias": "integer_encrypted_1000000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bcfe25aa74c45f540ea9869d814b87456326344ecaddb887948106a4a8afd92807632d2dc6494c7ca7dcf2b8f88ca9266b8fc24c8a69d516e89e3d19f4d0cd8fc3f7b4c2919ee8fab6e47b4940e53e2d087adfbe796bfe69d06957aa10be28e290d167b5bcca6c632090aa425874ecf5a30de441cb145253360f9bc3b5b4b7d71d17e635c5e3ae729975e9a85f8ecf3d8a6969eb441f44b90e9aa3082190734dc9276fdaf7f7b31d220226c3d4d8187a3f7a76394691dceeee7ba0c1d94337413f495037588a28d454f595705a1c7b3ac3b2e86e94617b0c2549599ae518cb3ba46dd8bacd2a0a66a31962fff3557cf93fb56c476e2adbdc0f49ee7ebdf01a741bfecd7038cc43e48a35b3e1dfce63417a\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_1000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 333333,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.8,
          "Total Cost": 847836.63
        }
      ],
      "Startup Cost": 0.8,
      "Total Cost": 26.24
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using integer_encrypted_10000000_ore_index on integer_encrypted_10000000
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
      "Plan Width": 68,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bc289c53daf516e8568c88adb4f2d3b23cbbf7a042fdcd1d6cbad70010a62ee830bc54df044bb9e29efe363cab6fc4ee4c3e39d2f8a9ee67cc65c649dee782e170148ef852e50bceb66186e0fea1be6e01145a88d33344152b04143be84e12a338d80e1915b300915d044bfb40037e18ccbfb0519ec06bcdecfe08e4f3e501109828f3c1421d805f26c94939e3a42cba0d0f1b60be9525c804f26323400d823355c5c05a6d8c6f6a275b307e44243dbee6725d69b0cd9a7c34a92203edaec1cc29b595f8e360f7b3c46a996626e1e734f75be6d272d3bdf86dea2c60209b0d4cf1f744cee86267dc086ca4d62f8f45d0c1a753db67911a34e7dd5545b580597778307a13ab6d7bca8c3f6edcd759b75d54\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_10000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 3333333,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.94,
          "Total Cost": 13113848.76
        }
      ],
      "Startup Cost": 0.94,
      "Total Cost": 40.28
    }
  }
]
```

</details>

![Query Performance - ORE/range_lt_hybrid_ordered_10](query_ore_range_lt_hybrid_ordered_10_chart.png)

## range_selective_gt_100

**Description:** Selective range query (~0.17% selectivity) returning up to 100 results

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 100
```

**Parameter:** `2140000000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value > 2_140_000_000 LIMIT 100. The threshold sits 7.5M values below `i32::MAX`, so on `Faker.fake::<i32>()` uniform random data only ~0.17% of rows match. At this selectivity the planner switches from the Seq Scan + LIMIT shape it picked for the non-selective baselines (`range_gt_*` with threshold 5000) to Index Scan — walking the b-tree from the top and returning the first 100 matches is cheaper than scanning the whole table. This is the same functional-btree path the EQL query-performance guide §4 documents; the non-selective baselines demonstrate that the planner correctly *avoids* the index when selectivity is too low for the lookup to win.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `integer_encrypted_10000_ore_index`
- 100,000: `integer_encrypted_100000_ore_index`
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 21 | 1.20ms | 30.31ms |
| 100,000 | 100 | 1.80ms | 41.75ms |
| 1,000,000 | 100 | ⚠️ 1.319s | ⚠️ 1.425s |
| 10,000,000 | 100 | ⚠️ 1.418s | ⚠️ 1.525s |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Bitmap Heap Scan on integer_encrypted_10000
    Bitmap Index Scan using integer_encrypted_10000_ore_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 50,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c54c4cd50f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12bbb0bd496b07fcd3fb5f6023b99f693a88b0a0e8c51a1c10c2af20a0e7cfc0a7e40c25d147a0ac4821b8eae3d8d762b97e769d90c4aa9a8664f6041d91455ab95f38d16a8d06669d4393487ea6b1fde1e9532e3d075810c7cb75e2312c1ccba071e065a23c2c3d3a795fb836994b62dd2cba44bf078dcbe8ede1f0da70b421fcb6a55bdd32f285afe0a8220222afee937aedbb92a2b194a84788b6d45abcff27fa4549e99bcb6c1455f529609f6140fb5cbc35040f21dfd54508018a0a25bd0c4c9d35e526b7ad0843121738b0feee112e30d6cb8d6a50f1dd5c5bb7c50353132130126f16db97a124be86b4b499375ac323ae8f3b4262a34287177780d745228248f4271ef33e241274647edd2e7bb9ddcedb3a20db9a682cba094412acfce00bdc7d690e9502058a1b91da23040b9c8851684336236c699500b7d827d84454\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
              "Index Name": "integer_encrypted_10000_ore_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 50,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 16.91
            }
          ],
          "Recheck Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c54c4cd50f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12bbb0bd496b07fcd3fb5f6023b99f693a88b0a0e8c51a1c10c2af20a0e7cfc0a7e40c25d147a0ac4821b8eae3d8d762b97e769d90c4aa9a8664f6041d91455ab95f38d16a8d06669d4393487ea6b1fde1e9532e3d075810c7cb75e2312c1ccba071e065a23c2c3d3a795fb836994b62dd2cba44bf078dcbe8ede1f0da70b421fcb6a55bdd32f285afe0a8220222afee937aedbb92a2b194a84788b6d45abcff27fa4549e99bcb6c1455f529609f6140fb5cbc35040f21dfd54508018a0a25bd0c4c9d35e526b7ad0843121738b0feee112e30d6cb8d6a50f1dd5c5bb7c50353132130126f16db97a124be86b4b499375ac323ae8f3b4262a34287177780d745228248f4271ef33e241274647edd2e7bb9ddcedb3a20db9a682cba094412acfce00bdc7d690e9502058a1b91da23040b9c8851684336236c699500b7d827d84454\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 16.92,
          "Total Cost": 226.86
        }
      ],
      "Startup Cost": 16.92,
      "Total Cost": 226.86
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using integer_encrypted_100000_ore_index on integer_encrypted_100000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_100000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c54c4cd50f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12bbb0bd496b07fcd3fb5f6023b99f693a88b0a0e8c51a1c10c2af20a0e7cfc0a7e40c25d147a0ac4821b8eae3d8d762b953531b3b060ae34090ce80ad8482fc3f7762b9c33883b4da2c2f076642d3ed31a6aeff2b3575f09ff7f6310123141304841cc74996d88850fc4ed1f989f4a1693019e01ac64f4c7f46a67c7a1aa6e5e810b697da0815820d5c58f730940200e6676e103c4bd6746b27e374fef5fc76de8f5efe5f547f0c42e8fc48d4a0ce0dfb5f299cb6b36b9d5b76aeb113732d9b8d2acfe5afb7d9e6d59c140578ab4a9d48f298ceddf20a7bf028c04ce603ea91f30a2c09e03e7b20561ab94b23f0dd629b98cfeafbf8400e34f14ab8272b5a00358d9b060a0844768cc814e8a764148f29f3063d95818f26c5b7fae5e61e5d954045faa5cf50958bf6b96b5850afae14f3ab58ea7120338014aa91e85a496c5b13\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_100000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 500,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.67,
          "Total Cost": 2258.41
        }
      ],
      "Startup Cost": 0.67,
      "Total Cost": 452.22
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Seq Scan on integer_encrypted_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_1000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c54c4cd50f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12bbb0bd496b07fcd3fb5f6023b99f693a88b0a0e8c51a1c10c2af20a0e7cfc0a7e40c25d147a0ac4821b8eae3d8d762b973c9cba3bed1de176e1e0c737f3e06a87d24c01f532180c5c6077db31a11b5db39e39761370cfda38eade049692bb691af812eb7057175beb7df44a1e4280960b53b78b78d98b66568e00c60af82a8f01eeb4ca38ddb63ccee9b03ed6cf2aee3bc1c9ff7c05c1ae8754ef48c774f9c062b25b984fe11a69cec39ee6752ecd2595afb793db146724c17eb97d35c60e41ced6ebd7927cf90b486116e1968e7f6a4cac146ca749ef16bb38375c397df01f3df43ac1126fadc672d1709aa748f3b7d085f49723bde2c9e9e142f3b7084bb70b8f036275265f3f5ccb89805e1cf4308fce7df350e57c3355a25e958fe8ff0923307e5aa76c1a2ec1f8cfbe9b781402a8d8650468262388a0b9a78afedcf53ed\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 736191.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 220.86
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Seq Scan on integer_encrypted_10000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c54c4cd50f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12bbb0bd496b07fcd3fb5f6023b99f693a88b0a0e8c51a1c10c2af20a0e7cfc0a7e40c25d147a0ac4821b8eae3d8d762b90a6e72435955f17eca00bb5f24313355ddd769245e20eb3cf26894ef43403ef5ba9310bcb83c432fa5351944383f549d9f8f0b826c16146da7220bfb64a1fb262f2b0495330e0bd235b375daddc8c3f0bb7c0e0744c137e356ad043adcd0cbe7047298e4b5a14084aa7e52acbf28531f78b25828bdde1af75b0089afef8c1e1c70d8cce9adfac3f687fe207d7b779e3aad4631976390402c9123618ca76ae8d5fe52ddf11d8ad8f27c860cb5b864516351a6f5c65e2ed17e8cf28f9fb9998b51f4e924d5de50227a42d29284139555cfb259e673df19abdb2a6bd8a2e262c8ef98668b023d9add2135c159f4a0980e7a207ad001f0d74b0a7e02c5b159f12b35ab41f1e7454438c7be1d02bd5a0bd7ba\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 3333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7361905.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 220.86
    }
  }
]
```

</details>

![Query Performance - ORE/range_selective_gt_100](query_ore_range_selective_gt_100_chart.png)

