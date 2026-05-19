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

- 10,000: `integer_encrypted_10000_ore_index`
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 0 | 742.91μs | 791.36μs |
| 100,000 | 10 | 1.07ms | 27.84ms |
| 1,000,000 | 10 | 1.13ms | 26.15ms |
| 10,000,000 | 10 | 1.42ms | 29.93ms |

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
      "Plan Rows": 1,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb2880ab8dff4e902cd42f1f78e2a00cd45ef86778be080fc3efff00d742aefd8a5fce6c9afbbeba6a66336d9a6d59594f98f041e5a088784066aeb96e0338f9ab0233d852c7e76bd15bdb7503c53de6cfd7300736ce33bb85a9c41d126fc1210208442157094a709942bb76509ac8bd93f7cdea9eaa4d12b7a92afc94010cd12a57998777d9098b88f8cbb68626b8638da7bb9209e63dad0e8399f9c60e9ebe5913c586f7cde03f334684050bc52a4b2a11994ff55ce6f37b7ad1be166b1900bb512647ce3fe98788aa62aa22a14e9a6b1c130e2c3153cb6eaf6d20342a93f01851a3d7d705e8574cb503552b3d4d1ac7234e31b9289e4673ed102417ae4254de47ef07d92021f993b4023fca8baa7eee13\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_10000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 8.8
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 8.8
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28831f47848bc094f2ca9e69887ba1df68e5ba364e2d11e73cc67d75e109d9972933e4d6bc938fab902e0c42e475d5bfb077ffee84edc376e0a24c9e599962dc17a44a6944a4e9fe1582c3a122d0f4f4703a2590152c9c26bf07c478d899bdc26b5ae4b1f77edbe7be1a106889376a566ededad016f70ffd86e3ea78485b8c5bb447169d3d2a97c3e67abb644da4e5c9bbfa48de6673182781205f8f947d545569c2f8df2e3cf379594655f69cf3fd9e6563033c72368a1b0ae65612b1b631d8ca5c52f9e90731cb689e729744ec7d848c7753c6fd3aa5f307d501e0fb23cebfd89a06c7c9a976df7eb242d30d469c54b1e225e6694846f70d0c369dd8b65c1d60409dbb0a9b0a2becd859342af04029d9\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49500,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77661.0
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28c54b24c27c88a416a4d4f8c2f1dd2ddbbc33f37b6419432763569b2a3f019ccbc33da18da5aedec1d60dbda21124e3ccc747a2ec12b0fe8ae000f461226678d1f10e42dc69853c3c2f633cac70080819ed1b328b2824c03736056a0aada3537afd384846f7fbdbd1129cd4daad453fd2802105db3ab3d06d1379f63826f048fae3edfdbb61ae129bdcbb96dcf84a391e5c7e164ee86dc9fb6d9eb52ff0fb95ba9ad5182b63e6ed0c5ca53638b0a49acffb4c339fb4c6c5619e61e653b24a61ad76fcc820d87f990a72fe1cd21215a68d9eec1e72a5edf7289e51dc2ee5d6c75cd81fdf472e8650d89f3b8b7f37add11d371979be85ca7d04dd1730d3b54c16f38d4e8952996828e03cd007a907d3911f\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 495003,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 776611.81
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 15.69
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bce035ad53ca2690eed1e71042c260c9e6ee809838442e212025aab171a1e201c10b8f1ed816c170862c88e71d8f1bbb6ea753a69763773f04aa828616f2dd406571a5105ffb7f0fd2645e30ab97bd8db9695307bc53fef51485e3703569531275349e644604a43d3dc36351733b7ab42841f7e31c17334884cab9c34c825f3e4f1eb5ca73ed4ae08cd4472b1de7923671aa4023c454c0477c0b6bb635e84ffa6a933f1b8bbd2247eb853b54615ff841f7df80eb76849f616aef146bd19016cb3248887cbf00ecfd99745dc8bbf3aaeb514286a2cab99df87029a71ff87e63358b9d0688b079fb4c720e5d1461b38f5b2cfb3c42be361bdce13d4ea5b336f9e8e58d8844c007786f9f4c3abb0fb42e9fc9\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5050002,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7791074.54
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 15.43
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
| 10,000 | 100 | 4.08ms | N/A |
| 100,000 | 100 | 6.66ms | 40.94ms |
| 1,000,000 | 100 | 6.90ms | 42.23ms |
| 10,000,000 | 100 | 8.14ms | 45.46ms |

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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28c3de831b580bad7b43591631d23328ad696fd611935e2bca21f57607b2575507a3fdbebfbff5375b676422984bd68289b44cc2c336e281f9422c3bc1816bc9d9c9973e43528fd3eabcb1cef47301c102eee6b32979d2fa979a6ebd7254cf165e853bd650b7c35c75c82f2974cab5a499cdcea693b6aa963c3b700859ac7a0babde09819a9be82c670981ed28a4a8d463824f7916ab96d880f8073234c0971736d6bf359914abfc4b6f7e23600c643002fabe55013d2b633854b1622e606bece9d68824af9935ef296212789b9b90c935fab697912e2a3cbf2d25f93c3149f77bdfe742f6a8930c8e8bea21ad317f13fcc376f67ddcf89a6934ac767b08e6a4a21a1b6781fa508350d8b6dc3b5261c3d9\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10000,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 9029.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 90.29
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28ac44c27f3041b06da4d9618b51f54c145415e35b543c58382e169bda1c6f30bb88f28fe8a2ec025af4e97153f51291e841c8768ed42d54b27320db1f5dc6e6b28c4075fc644a4f450d52c5fa7aac80bd70dfa2d5c4c2ac0a473f461b9b9d35a215b317c77f38cd6ecb488b8302567e619f0e0caeabf6991b30da22f70ebc4b1945470e124d59b8de2646953005f5ce75b8b6fb82f7a1b0a66372e274842b6d03bbd1ae6ecb2fcbe4d4ed52c1d1769588a336027f59e7401c2c1f944a6ceffb56497e96056e191d2f95a89943d3ef2e83e5f7281a139f6ffdc99567a8d4f852d9f47fe327ced10d90a6edbfd37aa5411ed0c46563363ed0811ca0eb38d102c538d2282b002594ca6ae08b94c4f37b1639\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49500,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77661.0
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28b37cbb1cfbc2bb14ae878c87d85bd083898504117cfd67e18d0fd82afbcc9f4928d88348be860795bd2435dbddc70dce31fa243fd6f2ead5386c7572637ec7f28698e24ef7a7baad0cd7dfc4245c0c8f02bb7ae014b67f9a1bf0b7d3226bdf0add09b1a599853de61061bfd970b373a20e11e76da65a3cb1fb29694d93a17f32c446b9104ec6994082374ba49d7542a042858e5667024ff39fa024508f0a218c432ebfd7247871c5eb42dadb26b507e6c97c4b14aacf5095ab9cc5094092929385481898099ece5ad0f22e5c2a98259488f00bd7f22b5857a71ef64f7dbc96cdf27a60aede3674a9c3a3d69d1d0df1b9cc0c2c8d34b963ee945c7b977fae07a3546c51bfafef57dde9e863d6a04b580c\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 495003,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 776611.81
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 156.89
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bcf22472e31028b9267367f6ae1d7fda813082cc9cede20d048630743a6358ec81ec542cc9f6638716caebf2ce1f3762a70e5907b839e860e32106815dca3b98bb4969532c49f6ad74a4315ffe634396b78b23adbb4846f88d9a3cd0598023a858f9a7753d96f8339f75788a1d5b79f25feced5e50cffeb151ea84df8e7ecb1ce650d5f8df0c69ef1e48f9d6e3957861dcc67a7072167564a1813f95f16d860d59631db8e9ea7428bcf21fd4706f395b021f320f2ce0202437cd94de9512285cfa2ccb8080509ce91e0c4567e5894e9aef6af305ac3deb09565ed88596b21fd8f80021e5ee348af72b7c6e39195a8c5359a5a4b90d1dbd6d0275879c823624b255a7aae72714955c9987ab9437af111958\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5050002,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7791074.54
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 154.28
    }
  }
]
```

</details>

![Query Performance - ORE/range_gt_100](query_ore_range_gt_100_chart.png)

## range_highly_selective_gt_10

**Description:** Highly selective range query (~0.011% selectivity) with LIMIT 10

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 10
```

**Parameter:** `2147000000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value > 2_147_000_000 LIMIT 10. Threshold sits 483k values below `i32::MAX` (~0.011% selectivity). Engages the ORE btree at every tier (with current stats — see the note on `range_selective_gt_100`). Useful as the upper-bound demonstration of how cheap a selective range lookup becomes when the functional index engages.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_100000_ore_index
ON integer_encrypted_100000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 100,000: `integer_encrypted_100000_ore_index`
- 1,000,000: `integer_encrypted_1000000_ore_index`
- 10,000,000: `integer_encrypted_10000000_ore_index`

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 100,000 | 10 | 816.12μs | 25.38ms |
| 1,000,000 | 10 | 1.00ms | 26.51ms |
| 10,000,000 | 10 | ⚠️ 3.157s | ⚠️ 3.605s |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af391252a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78832bb9ab1c62d6173375d24578f37a3a1894af5fffa9bb07282f385b6a950f871d007ea7bab7ad6a555492b307b4323b6a0c9f644f87ace1c0e39936b3f00b7cfc882bb8ddc9bd6606c33bee757feadf9948dc9332d2118d56095edd4676a06f0cb82a2148b065e6241ce081d0bc4769bb95c04e6df1c46b9c0dad3bbcb14b1afb04acbfdafb94970db5068d05522399730df553f64858c61849c02816ad40a7820c53b2fec9048b1f83c60cb68438df3f70f0fe5de01bf1cb14666a15d6683b51109923b670ea2edd9c9e1d9dfb2c9e2958b162e0aea4a8e4e2da1edbf5098268676b624ce346d9c57bb07af2963f2cc36456dc1393ea62b0432b9a728fb37c6a662030cd221dd9adccdd9e9882f9b957455a041b274ede526664dc187da9e3bbb2287080e546cb01ff37c50653a44ec80b8a742feaa49c11f1a9be11370ca4be\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_100000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 500,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.67,
          "Total Cost": 2258.39
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
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_1000000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af391252a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78832bb9ab1c62d6173375d24578f37a3a1894af5fffa9bb07282f385b6a950f871d007ea7bab7ad6a555492b307b4323b6a4af7e0aa55135c397e0fa4ce7d937f443694b9a95bc868d1c3fd7aaa9d1f0a8f40a4d51bfdb2f4c2c674bd5158e3c1653ada51e865066f7ec747762cd12a682eef4346017834926bcffe8445550841224994419726c802db3018c64a08c25450306c42b467065081b01c875d693c437e92ab5193991747158b126e8d88624868745380b71916e1818c362c13d939cf73059d549966075559355c9d8ef1d5bca7622d45e3dc1b4c0c973387884fb86f4d4f5aa95e50eb0f50f1b1a05b5daae0172a230f61ab1b972b9a0f6abd25280e867f07c5f8268d37915c344379e510f69e98417826c0e4ee20e364b9de36a37ba941fefcfdd01c279e3b5d30cd7abd1d8ae4d1b4791550eb806ac193d57facb855\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_1000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5000,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.8,
          "Total Cost": 22558.17
        }
      ],
      "Startup Cost": 0.8,
      "Total Cost": 45.91
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
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c74696450f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12000c1ce2da49187443be7a630b7bba4e4c788d889b7eef636659fb4d6b64dbab777c3973caa72a76ec3c372074623e36b9581b419d45178f30a9a1425f682756e3bdfbb522e3ae4b80e8ce1396a389b264d8022ba559ca8817d1bad7b3dd81c3fae9777aad6410b408d953019daf32eb05ddaa96c7c307d44f5f562258b020e845f7225c1f7a24d2bc579cea64ed2466d3443af5da3191984de5ef5d676a3be46ac355a6e12de53fa351bf664472b87e4f8a84afee754fb4de1d907d8c93dfe73cbc2cfbdbf0d7d0b71f106de7d77b5dfe34a4fc4fc1723d32268d84dfa3c2b3eeba29fbd74be16a38e18ba0d22a1efa91de07ff11fb8be9a6a341f7da6d641af43018f1987d5b2aceb44e68592d1fa032f56cb4dba6226a4cddcb76d6e47b214ec07204ac1fd370df4588df09fb18e9fe9bb541594b3c0e15632acdd041728e\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_10000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50000,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.94,
          "Total Cost": 225527.79
        }
      ],
      "Startup Cost": 0.94,
      "Total Cost": 46.04
    }
  }
]
```

</details>

![Query Performance - ORE/range_highly_selective_gt_10](query_ore_range_highly_selective_gt_10_chart.png)

## range_highly_selective_gt_count

**Description:** Highly selective range count (~0.011% selectivity), no LIMIT

**SQL Query:**
```sql
SELECT count(*) FROM {TABLE} WHERE value > $1
```

**Parameter:** `2147000000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: `SELECT count(*) FROM tbl WHERE value > 2_147_000_000`. Tighter selectivity than `range_selective_gt_count`; near-floor cost for an indexed lookup.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_100000_ore_index
ON integer_encrypted_100000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 100,000: `integer_encrypted_100000_ore_index`
- 1,000,000: `integer_encrypted_1000000_ore_index`
- 10,000,000: `integer_encrypted_10000000_ore_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 100,000 | 1 | 807.51μs | N/A |
| 1,000,000 | 1 | 4.95ms | N/A |
| 10,000,000 | 1 | 19.96ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**100,000 rows**

```
Aggregate
  Bitmap Heap Scan on integer_encrypted_100000
    Bitmap Index Scan using integer_encrypted_100000_ore_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Aggregate",
      "Parallel Aware": false,
      "Partial Mode": "Simple",
      "Plan Rows": 1,
      "Plan Width": 8,
      "Plans": [
        {
          "Alias": "integer_encrypted_100000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 500,
          "Plan Width": 0,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af391252a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78832bb9ab1c62d6173375d24578f37a3a1894af5fffa9bb07282f385b6a950f871d007ea7bab7ad6a555492b307b4323b6ab7e2c1f55322142e37dbf5dd91eb7a7bdb8b746f527c886ab1e99cdf9b944e151e387651575f34fcc4eb366fee9abcdda7ff1e803bb9902875f941584e339dfa7db0dd38368f2a7aa569a2171965de63409712f37005fe38e4722ca65cbcebe40d83105e97896cd86ad2a2b1d94e74af298f59243d4cbf0ed9f346dd654ac5d2e6fb6fca5dd9b495d5ecbc86fc237cc70c5f6a6b2e33f3c87a44d2b6f84fab520ad7a004306c4e9f50734ae750895ec943fcdaed1be8585dcbff2bb9d0ec7c2cc240cba764f65666712f4c0f0791c6072188c29424a9f05304515dc0a02c0d39fbaf59387bde6f5d62ab01b6ed5ef4cdc7b43db7dea70c420d2278b4187fc120ac496490f05d5fd913da1603c392aebd\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
              "Index Name": "integer_encrypted_100000_ore_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 500,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 160.42
            }
          ],
          "Recheck Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af391252a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78832bb9ab1c62d6173375d24578f37a3a1894af5fffa9bb07282f385b6a950f871d007ea7bab7ad6a555492b307b4323b6ab7e2c1f55322142e37dbf5dd91eb7a7bdb8b746f527c886ab1e99cdf9b944e151e387651575f34fcc4eb366fee9abcdda7ff1e803bb9902875f941584e339dfa7db0dd38368f2a7aa569a2171965de63409712f37005fe38e4722ca65cbcebe40d83105e97896cd86ad2a2b1d94e74af298f59243d4cbf0ed9f346dd654ac5d2e6fb6fca5dd9b495d5ecbc86fc237cc70c5f6a6b2e33f3c87a44d2b6f84fab520ad7a004306c4e9f50734ae750895ec943fcdaed1be8585dcbff2bb9d0ec7c2cc240cba764f65666712f4c0f0791c6072188c29424a9f05304515dc0a02c0d39fbaf59387bde6f5d62ab01b6ed5ef4cdc7b43db7dea70c420d2278b4187fc120ac496490f05d5fd913da1603c392aebd\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 160.54,
          "Total Cost": 2109.63
        }
      ],
      "Startup Cost": 2110.88,
      "Strategy": "Plain",
      "Total Cost": 2110.89
    }
  }
]
```

**1,000,000 rows**

```
Aggregate
  Gather
    Aggregate
      Bitmap Heap Scan on integer_encrypted_1000000
        Bitmap Index Scan using integer_encrypted_1000000_ore_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Aggregate",
      "Parallel Aware": false,
      "Partial Mode": "Finalize",
      "Plan Rows": 1,
      "Plan Width": 8,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Gather",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 2,
          "Plan Width": 8,
          "Plans": [
            {
              "Async Capable": false,
              "Node Type": "Aggregate",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Partial Mode": "Partial",
              "Plan Rows": 1,
              "Plan Width": 8,
              "Plans": [
                {
                  "Alias": "integer_encrypted_1000000",
                  "Async Capable": false,
                  "Node Type": "Bitmap Heap Scan",
                  "Parallel Aware": true,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 2083,
                  "Plan Width": 0,
                  "Plans": [
                    {
                      "Async Capable": false,
                      "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af391252a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78832bb9ab1c62d6173375d24578f37a3a1894af5fffa9bb07282f385b6a950f871d007ea7bab7ad6a555492b307b4323b6a68bb915611bd3f625e5b43b4320b3524ff4338f2716f86a41af55822a88371f7c6843276017ffcada49bb3378e1ba7991cd161e58cb7fd0d164126754aea479dbb1d0d228b5acf2b410f07f62830f11b9ad71120555232477ed3666e7661182ab09cb974ad2ba53d6ed63fa42cd6b47ec06a4198a68500c47ccc591d1e929547fb00959b8d8d7cf5fae1b23ef5f6ad4c4e00d6a749a2e7b4355b1910a78427dfcb03a1f3fc943c62020b039dbe29c419208d2955a2141058034d019b35af3d17aa52e525d1dc782b7489d43b41f384be8e0a879f61ebcf355a3fbcd769a2184677d624833157e17727fdaf11ef57356c29832637616cce711ae5dfd5c0bba8fd79dc1ae9b50bc17cb388336c8a5c3ee3\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                      "Index Name": "integer_encrypted_1000000_ore_index",
                      "Node Type": "Bitmap Index Scan",
                      "Parallel Aware": false,
                      "Parent Relationship": "Outer",
                      "Plan Rows": 5000,
                      "Plan Width": 0,
                      "Startup Cost": 0.0,
                      "Total Cost": 1598.3
                    }
                  ],
                  "Recheck Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af391252a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78832bb9ab1c62d6173375d24578f37a3a1894af5fffa9bb07282f385b6a950f871d007ea7bab7ad6a555492b307b4323b6a68bb915611bd3f625e5b43b4320b3524ff4338f2716f86a41af55822a88371f7c6843276017ffcada49bb3378e1ba7991cd161e58cb7fd0d164126754aea479dbb1d0d228b5acf2b410f07f62830f11b9ad71120555232477ed3666e7661182ab09cb974ad2ba53d6ed63fa42cd6b47ec06a4198a68500c47ccc591d1e929547fb00959b8d8d7cf5fae1b23ef5f6ad4c4e00d6a749a2e7b4355b1910a78427dfcb03a1f3fc943c62020b039dbe29c419208d2955a2141058034d019b35af3d17aa52e525d1dc782b7489d43b41f384be8e0a879f61ebcf355a3fbcd769a2184677d624833157e17727fdaf11ef57356c29832637616cce711ae5dfd5c0bba8fd79dc1ae9b50bc17cb388336c8a5c3ee3\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                  "Relation Name": "integer_encrypted_1000000",
                  "Startup Cost": 1599.55,
                  "Total Cost": 19587.07
                }
              ],
              "Startup Cost": 19592.28,
              "Strategy": "Plain",
              "Total Cost": 19592.29
            }
          ],
          "Single Copy": false,
          "Startup Cost": 20592.28,
          "Total Cost": 20592.49,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 20592.49,
      "Strategy": "Plain",
      "Total Cost": 20592.5
    }
  }
]
```

**10,000,000 rows**

```
Aggregate
  Gather
    Aggregate
      Bitmap Heap Scan on integer_encrypted_10000000
        Bitmap Index Scan using integer_encrypted_10000000_ore_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "JIT": {
      "Functions": 7,
      "Options": {
        "Deforming": true,
        "Expressions": true,
        "Inlining": false,
        "Optimization": false
      }
    },
    "Plan": {
      "Async Capable": false,
      "Node Type": "Aggregate",
      "Parallel Aware": false,
      "Partial Mode": "Finalize",
      "Plan Rows": 1,
      "Plan Width": 8,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Gather",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 2,
          "Plan Width": 8,
          "Plans": [
            {
              "Async Capable": false,
              "Node Type": "Aggregate",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Partial Mode": "Partial",
              "Plan Rows": 1,
              "Plan Width": 8,
              "Plans": [
                {
                  "Alias": "integer_encrypted_10000000",
                  "Async Capable": false,
                  "Node Type": "Bitmap Heap Scan",
                  "Parallel Aware": true,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 20833,
                  "Plan Width": 0,
                  "Plans": [
                    {
                      "Async Capable": false,
                      "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c74696450f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12000c1ce2da49187443be7a630b7bba4e4c788d889b7eef636659fb4d6b64dbab777c3973caa72a76ec3c372074623e3636a48cacc6fa21cf8b6d7e9397f66d65db0bf1139c96af507702882ec0a2b2e6788ed111afd249f56cfd3ab4345e84ff9c24a3d868f851280738122f536435565d32364de6a0ce34331f73783093fe7d25b468c99da1ba0f27cdce6fe1cbd01142f5fa9c226ebefde9fdf008cc082e47e151a647218343e65430dc173e30dfd2e73d8194c5260d63c3576f2e31b1a7bb22ca6af9cbb6becdb745325c2dd4241fadc83b7a650ec9be1ec1c808b7a9ce3c7cee5eb0706c4bc89df7f7b3c3bb11352cca1986ab8b1bc3d9d02d2c30039258bf6c0a045f6933e4c532028601e760996e0e463b5cad6bf39064caebf7f8b6a2ae3946bdcb70fbebefb6d729ad9cb1406564f445922fb1653bf5c6183ea56fbd\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                      "Index Name": "integer_encrypted_10000000_ore_index",
                      "Node Type": "Bitmap Index Scan",
                      "Parallel Aware": false,
                      "Parent Relationship": "Outer",
                      "Plan Rows": 50000,
                      "Plan Width": 0,
                      "Startup Cost": 0.0,
                      "Total Cost": 15963.93
                    }
                  ],
                  "Recheck Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c74696450f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12000c1ce2da49187443be7a630b7bba4e4c788d889b7eef636659fb4d6b64dbab777c3973caa72a76ec3c372074623e3636a48cacc6fa21cf8b6d7e9397f66d65db0bf1139c96af507702882ec0a2b2e6788ed111afd249f56cfd3ab4345e84ff9c24a3d868f851280738122f536435565d32364de6a0ce34331f73783093fe7d25b468c99da1ba0f27cdce6fe1cbd01142f5fa9c226ebefde9fdf008cc082e47e151a647218343e65430dc173e30dfd2e73d8194c5260d63c3576f2e31b1a7bb22ca6af9cbb6becdb745325c2dd4241fadc83b7a650ec9be1ec1c808b7a9ce3c7cee5eb0706c4bc89df7f7b3c3bb11352cca1986ab8b1bc3d9d02d2c30039258bf6c0a045f6933e4c532028601e760996e0e463b5cad6bf39064caebf7f8b6a2ae3946bdcb70fbebefb6d729ad9cb1406564f445922fb1653bf5c6183ea56fbd\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                  "Relation Name": "integer_encrypted_10000000",
                  "Startup Cost": 15976.43,
                  "Total Cost": 195823.06
                }
              ],
              "Startup Cost": 195875.15,
              "Strategy": "Plain",
              "Total Cost": 195875.16
            }
          ],
          "Single Copy": false,
          "Startup Cost": 196875.15,
          "Total Cost": 196875.36,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 196875.36,
      "Strategy": "Plain",
      "Total Cost": 196875.37
    }
  }
]
```

</details>

![Query Performance - ORE/range_highly_selective_gt_count](query_ore_range_highly_selective_gt_count_chart.png)

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
integer_encrypted_100000_ore_index
ON integer_encrypted_100000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 100,000 | 10 | 1.18ms | 26.08ms |
| 1,000,000 | 10 | 1.40ms | 26.91ms |
| 10,000,000 | 10 | 1.72ms | 30.15ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28977e2038c9bfdf055edb7d95af168c357a4742566845a73341a9157fee57a29d7c99aa7d29f349b71dce75ccea9258e927fbf46181b2ebd19ccd72c6cd873af412909fb6499d5fea53ab7a99a7512aa81b2ce5f35aeda3664f230a0f3675c15f50c637bbf61b679f3a55fdfadf5783127a8a8ed1d3c6236620515a2b1dbb3bb4d0242a886534b0bf9f9afa38252d38fd97a389307c76a8f7ab072bc3d36c1ba72ef6d70f46e8318bfcdeabe4035cfba8d75aa6e57bcb89e6b44e74748912340c5a526ba9aea6fb640d93cb6704c9a578fdcde5c14c50dd4f691f79ebea98711adfb6e7d0f812890f795a3cc8eb2f0985097b90b11a0f97d2f88ee046aa4b6c1802c1130018e6183c7979d14fd837c5c7\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50499,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77910.75
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb289f6f8a138dfecd4f51e6843994b99155c20e7d38b6aa0b74319976d228427983c9a00aec8252439a95cc2223b69317469126ca9e9433b063a24bca6ffb1de9000dd6e670bddcddbf7fafa23bd4100c8c2c61eda1f4f534cc2ab712dd1b4896541ced1abe9b80931247500cabcdd89e09f328c897be6240b4dfbd83ab16823bf5be24fdfdadbf32a8b53d0ef5ee5605f86563edf1dcc504928a2d8d8c0f6cd5ba55c29ab5e05cb0f75ed1491b305f713dc0d7a8d396c6413824aa236eee821b78562d341020989e77f71ce73cab98a421a9e4ca3f8eb11eac4159d323ea5854688fa83d5ef1fa689e44adfaea2cc0c39a1fb6e2143d10f562ed2c3cbf918a37b5601c6712510c1991c7da14b6ca96111f\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 505002,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 779111.56
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 15.43
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bc3f0e300ff67bc4d9fee3701c15fd2a1379f439472f348072a34d3f9e6dfc5ce064217764ce8b4d315ad49b082c9b558dd12a8de0b0c42022f1ebe75a86cf5cedd7f8a9a8446da81e80db2e3c8959ac852eee567cae6de44594030ced59d4ce678e56033858cb3ba2f7553aeebcd6fec71a72837339d1ce83b8ed6a66f88568d6769afa9ef42abbbd8f68dccfc307ad6a1ac90dd2c3f8b146db56bbfafdad3f08d921085fd9663a490327334c5021a249c5f9e3778f9b4ebdfbc85a24b4ae50921d7e553e55e07f7aa2afbdcf19bc9c7f2cfa52237826319241571e936806059603958a1b582ccfd35346335856af92aa6d8a1587b331b3359c39837c88063ba6a17ceefa37b187db9920dd4f8ba88ee5\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4950001,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7766074.29
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 15.69
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
integer_encrypted_100000_ore_index
ON integer_encrypted_100000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 100,000 | 100 | 6.64ms | 41.12ms |
| 1,000,000 | 100 | 7.03ms | 42.94ms |
| 10,000,000 | 100 | 7.58ms | 45.56ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb2855f37d595c4b17cad9985d279e25d236a7e4bf156d822c4e0eca9d353f55f08faa76a1811f39a60f05447da98c711945c25425e51ca48bfeb705ba5d84a46a7f3ab7e92e569c3c24c8b11eb3d6b5dbed447a364f09de5e249d5542797cc6d5931312f248dd87c4726b540608f009c0830fbefb705dc40c50c7a21885ae98ff7f1525a4d0cae2cd1c660addc4c4621d1d6e51892e9c290d60def8db7f60deb3b368e45f5e9027e603f1e1a721f5ef04241151221a98f11d40e60a7b65c743a1dff02695195735bfafd7e96afc0d684943ecf3f8a2f22387a806b9b410e9d3b2e320ec7818a67cfbf365690fb462d52a336a33e10540067202b6e27c72ab3c4dc68af272032e19a165d640c4825ba126db\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50499,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77910.75
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28b0ed617644d86e661438a5cb8d06d59e1ad9146ecbca790f1610499b064f850c8ed344496e720ebbcbfb3ad83a47ca5451fb46e4f42f9448ca7a6d7a286ad3cd5437794a5cd59d6b813c6546e7a46b040cfdefc7e5b46b60abdcb1b9d5e62815ba1fb13d175424b5f4eee0cb65cf0baffd20c6111f859668dfdebd8aa8201d8ab37d98f34fe60383bbe9c0a44db67a25e139ff0453669312f5877239587c0b76be9ed8e6539fe6e899c944e1b301502f6a390cb7287c076865e9bffe0e7a8dcda516b86a4ddb0cb020a576fd711f8d25e436e9277f33386e448573a506374627acf0df6b463b1d88c29de682f856dac8b1cfb93d906377fb29ae04a960769138ca992d0e37f75534d59dd6fe588e22a1\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 505002,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 779111.56
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 154.28
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bc43a1ddec1ade1332d580d84e400ad9bf21691674d9edec0a1ca27d2c2d11e7c071cb97f10b33a98bc723350a42423ab0dd795c5a8347e3988a57f57599d058b161c97821aadcfaa8106ba32a7e21dbe4b46b57f4a2a1d79b56e96fd3d35520a235cda31312ecdcc17aec5b43aded4495b31006a25d72dca981c939e0351b3dfcfe98cf25896c488f13541336442e82cddbc625fb593b534672cac0b3a3dd30bd63be28eae0f3540007fc26fcbf7cb79633325cd94c3ce52b9260e85ec9368c777849d03910f89a6be4afbeb542104f507e5a4d4f9275d2922fa250c56b443739cccf1f5e74f71728219714d637e4773fdbebd76e6ec9a3d5fe303715a9d0f0a604840ba7f9df2af6aa1dc5cf16e406c6\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4950001,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7766074.29
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 156.89
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
integer_encrypted_100000_ore_index
ON integer_encrypted_100000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 100,000: `integer_encrypted_100000_ore_index`
- 1,000,000: `integer_encrypted_1000000_ore_index`
- 10,000,000: `integer_encrypted_10000000_ore_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 100,000 | 10 | 1.13ms | 25.93ms |
| 1,000,000 | 10 | 1.17ms | 26.26ms |
| 10,000,000 | 10 | 1.16ms | 29.07ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb289f03812fcf5012af1be7782f915c69643fae601e3c87ec84081ee8fdb7e889e15b6eaaca838a9df02978e1a51f037d4fe69b8272c62dfea90febd488a12592db18bba6f70b7f1efeac8cd9687c03c0eac94a71bd123902538e6168c2642adf72c37b4a3b701476a5d0c844c6c8bb2c803d885d1a9bd01e5bb4a3aaa8ca903f1ee3330e63affc2dfd0972972a04855f92ff1fc0fdbc57f26acf754e2a3dee0cf60ce4530d9bb5936a22809d62b19b2c0e207ea4eb962ade6139bf136c02665942cefc09eaf2a22d892d2d15b89656134535b63fa5bbb7066522f8e913d5677b80fedb22f1b825f32a27c14160afe1fb200d52152c81e459de932a2cf9115db8896b23a119680678f94a1445b911f62bd6\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_100000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50499,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.67,
          "Total Cost": 99025.11
        }
      ],
      "Startup Cost": 0.67,
      "Total Cost": 20.28
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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28f14f86c11ad24db7a85b38df5525b450a58a2565946fbe057a32b0657c03c3bbad2fcd559aa5c5c174a2b96a48a3e55614918fea4094079f4f29ecd1666fee6b2e8b07d7adbf0217ba111d6439736a33dbefd63f49dda88a15f2629fa1baf238176994aaf17d4c9c71a40f1d4f2abfbffb5d40d0d0218b99c37262592078e2fcf80fc9c118061dc72de06b2ff9166d50289154e84ea7fee809cb632c60e34d7d5d6a8c751ff42d546b65df02edeb3525e614c3f55d149527b3d6bf1527cbb515e39e55abebc816a5262fe6f577d9e614e57174ec377c9ec08fbd4bfb8c1a0a5c26e701e6ad4161ed32c8988efda49468d05407c63c31a15b05072ced68e1b4c43a06771096b83bfc8173c4309aafad15\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_1000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 505002,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.8,
          "Total Cost": 990179.83
        }
      ],
      "Startup Cost": 0.8,
      "Total Cost": 20.41
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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bc24f6a359f41fa3cb965233c1f31d65223e5fd83fa64ff6710ae83260a215b9d9b4d148c6297672bab63eb4affa1c28e64db59dd5af66b045c7d6be898b226b92ae831f7603e284bb1fefaa90bc751f632bb090d4438d5f600fb7289fd77b5485af1071fa0fc7f1bdcc45fbdffa5bb08e330bcf5987b536e9bba026c6f030ce6564f7107a72dbeaa665eceaacaa37f766d8bf8026a214aaaf8c79467ed8db91eb01414832c4742e38f909e8b234049d6e80c34d6533388ec2305aedc60868a03d63986a148a7952194a8ac88f122b8273a30ac4c14b1cb76adecce5a397aa7db67f507bd3d5b86e8e080e4c7cfb8bf96c59e31254b3fe5685f67c6414e79b01174f5146ddd77969c4a9f330134252ef67\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_10000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4950001,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.94,
          "Total Cost": 19385383.19
        }
      ],
      "Startup Cost": 0.94,
      "Total Cost": 40.1
    }
  }
]
```

</details>

![Query Performance - ORE/range_lt_hybrid_ordered_10](query_ore_range_lt_hybrid_ordered_10_chart.png)

## range_selective_gt_100

**Description:** Selective range query (~0.17% selectivity) with LIMIT 100

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 100
```

**Parameter:** `2140000000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value > 2_140_000_000 LIMIT 100. The threshold sits 7.5M values below `i32::MAX`, so ~0.17% of rows match on `Faker.fake::<i32>()` uniform random data. Engages the ORE btree at every tier (10k → 10M) — walking the b-tree from the top and returning the first 100 matches is cheaper than scanning the table once the planner knows the predicate is selective. **Note on stats**: this requires up-to-date planner stats on the functional index expression (`ANALYZE <table>` after re-ingest). Without current stats the planner falls back to default `>` selectivity (~14%) and picks Seq Scan, which is silent but produces misleading timing. The bench's `prepare:_table` now ANALYZE's automatically.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_100000_ore_index
ON integer_encrypted_100000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 100,000: `integer_encrypted_100000_ore_index`
- 1,000,000: `integer_encrypted_1000000_ore_index`
- 10,000,000: `integer_encrypted_10000000_ore_index`

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 100,000 | 100 | 1.78ms | 37.78ms |
| 1,000,000 | 100 | 1.77ms | 37.31ms |
| 10,000,000 | 100 | ⚠️ 2.092s | ⚠️ 2.262s |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af3965ab3800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78839e5375b93148d2c7258c7c5b10da6baaf6161425134bf7a220b78fc411dc6f7124f9da3212acff12515e2eed9100ed1b0b59be5024dc25f6a8065f37dd814d80a0b2e1f0ad63ff949f051af3bcf342e7a4fe514e3b615b65179fe591368fe6c49f641d463bc80a7990be776a3fa027e7e7cbc51712421cc3c0af44fae53a52dfe292deb3760f96908862e3e8f252c2c804a8b215b0954c642f4e442439ade6eee947aa7a94d28b673dd48824e9308d0e4adb98b6609b44cfb23caf80778095068939a422ed206a11dce801d6458c5fba71b80b0c5a10ffe5366024142f9b73c8106b5529fdfe6d5977e346e44b607887c257567f90808a79391d854bad9211eca4c8075488531d6aca9e2fd6b49168884c6945c0b42a2f0e6a197d130bcfb2866b7dfe72cdca395f8fe45d04fdd91415ed635c465ff905eeafa566ee3c8e1170\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_100000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 500,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.67,
          "Total Cost": 2258.39
        }
      ],
      "Startup Cost": 0.67,
      "Total Cost": 452.21
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
      "Plan Rows": 100,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_1000000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af3965ab3800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78839e5375b93148d2c7258c7c5b10da6baaf6161425134bf7a220b78fc411dc6f7124f9da3212acff12515e2eed9100ed1b073d3780842ccb57a441e061b6b66fe991185112718a128c043ca938df32b241f6e45e4e81cf0e4751f92e45cb27f7e119d0c4d130cec45d397af3fcc4dd71002f032ac7c433990b1ecdca2cb92ef978c4f1da0f9e69a7320a9ecbde63df70b00e33dab3974bca172e011577f6af2afe7845a822f4f6fd64decdc2ad38ce04a14579eb7f79ec6679580194de38990dab811c5240b08e14037fe0c4e764a0312e19e6be78965af67655eae30513e612befc14b4d8b7770544d1597fbdb59ccb28ac7f3230bb7153ab516be707fdf5091308ccf66a075f2335e10e4614b6cd82bfedea30e20512f458faa49cf77bbdf71ae65353453a63f551229161c6b00be5832b3da7a5137b10bc86a7d1f736086d4b\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_1000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5000,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.8,
          "Total Cost": 22558.17
        }
      ],
      "Startup Cost": 0.8,
      "Total Cost": 451.95
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
      "Plan Rows": 100,
      "Plan Width": 36,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c54c4cd50f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12bbb0bd496b07fcd3fb5f6023b99f693a88b0a0e8c51a1c10c2af20a0e7cfc0a7e40c25d147a0ac4821b8eae3d8d762b94da6a5c9730d37d11cfa0560bc682cabfa753c419e4719a7b1568265a12319e2b810be016244a215aa315ce894faf33a304c17197f68ddc0c5203ca82fd55a34870e5f445a834781ec684e007b489678f925aa5ea2083eda4bace925157a22ee9c1f48d64c33d18907cba79f728961e13d1457e95644139c3df24d89ffeb31d195b06d4893ca4d8a90cbadeed07fdba653fb4c62ee72e9e2c00336ba41dfc6160151c5370f7344881919a877daa7d473d628ed92b04e39da3e29347b705829051cc6cefdc8151e26582eaaa618fee547cf762daa98399af7347ed463730a56f6801c3495c5e0f880f51bf6820df041c0e16f67d4e85b0582a3f4d52fe493c66ab0f6378b831e6fe5edadb596c59260eb\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_10000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50000,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.94,
          "Total Cost": 225527.79
        }
      ],
      "Startup Cost": 0.94,
      "Total Cost": 451.99
    }
  }
]
```

</details>

![Query Performance - ORE/range_selective_gt_100](query_ore_range_selective_gt_100_chart.png)

## range_selective_gt_count

**Description:** Selective range count (~0.17% selectivity), no LIMIT

**SQL Query:**
```sql
SELECT count(*) FROM {TABLE} WHERE value > $1
```

**Parameter:** `2140000000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: `SELECT count(*) FROM tbl WHERE value > 2_140_000_000`. With no LIMIT the planner must process every matching row, which at low selectivity strongly favours Index Scan over Seq Scan. The companion to `range_selective_gt_100` — removes any LIMIT-related cost-model edge cases and demonstrates the index path in pure form.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_100000_ore_index
ON integer_encrypted_100000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 100,000: `integer_encrypted_100000_ore_index`
- 1,000,000: `integer_encrypted_1000000_ore_index`
- 10,000,000: `integer_encrypted_10000000_ore_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 100,000 | 1 | 1.29ms | N/A |
| 1,000,000 | 1 | 6.90ms | N/A |
| 10,000,000 | 1 | 77.18ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**100,000 rows**

```
Aggregate
  Bitmap Heap Scan on integer_encrypted_100000
    Bitmap Index Scan using integer_encrypted_100000_ore_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Aggregate",
      "Parallel Aware": false,
      "Partial Mode": "Simple",
      "Plan Rows": 1,
      "Plan Width": 8,
      "Plans": [
        {
          "Alias": "integer_encrypted_100000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 500,
          "Plan Width": 0,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af3965ab3800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78839e5375b93148d2c7258c7c5b10da6baaf6161425134bf7a220b78fc411dc6f7124f9da3212acff12515e2eed9100ed1b3b410846d7de95236e0317b3415302a6888fbc6f13063fa21f278f3cca63bb4dee6ebe2045c0bb3d3a504bc0e32e72833602c5ab537c714d66a443ddbe27c33369890a00727d6f44db50eb65329c562db4fbc43659d380570a03a635ad442016113818e676219bb7b7fe01410d59063b9ca35498a80fd766782d26f7e8527891453933e36f5a23f973d87f890b70eb95e187e56a3484893488d82d3c8820e9e7a92a9be2964082557387b2faa0cab17c265a85670e815170b5dab86902f8d69a421cfa2d627ce4ff0d55cc87c6bd478985fe5405d544caac31e38986322f6749b48339ef1567e140db9478db27ed9aefae3a2a6d6d489dccf404140e00c4d726b5175bd682d885cf0d4820c227565ba5\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
              "Index Name": "integer_encrypted_100000_ore_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 500,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 160.42
            }
          ],
          "Recheck Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af3965ab3800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78839e5375b93148d2c7258c7c5b10da6baaf6161425134bf7a220b78fc411dc6f7124f9da3212acff12515e2eed9100ed1b3b410846d7de95236e0317b3415302a6888fbc6f13063fa21f278f3cca63bb4dee6ebe2045c0bb3d3a504bc0e32e72833602c5ab537c714d66a443ddbe27c33369890a00727d6f44db50eb65329c562db4fbc43659d380570a03a635ad442016113818e676219bb7b7fe01410d59063b9ca35498a80fd766782d26f7e8527891453933e36f5a23f973d87f890b70eb95e187e56a3484893488d82d3c8820e9e7a92a9be2964082557387b2faa0cab17c265a85670e815170b5dab86902f8d69a421cfa2d627ce4ff0d55cc87c6bd478985fe5405d544caac31e38986322f6749b48339ef1567e140db9478db27ed9aefae3a2a6d6d489dccf404140e00c4d726b5175bd682d885cf0d4820c227565ba5\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 160.54,
          "Total Cost": 2109.63
        }
      ],
      "Startup Cost": 2110.88,
      "Strategy": "Plain",
      "Total Cost": 2110.89
    }
  }
]
```

**1,000,000 rows**

```
Aggregate
  Gather
    Aggregate
      Bitmap Heap Scan on integer_encrypted_1000000
        Bitmap Index Scan using integer_encrypted_1000000_ore_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Aggregate",
      "Parallel Aware": false,
      "Partial Mode": "Finalize",
      "Plan Rows": 1,
      "Plan Width": 8,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Gather",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 2,
          "Plan Width": 8,
          "Plans": [
            {
              "Async Capable": false,
              "Node Type": "Aggregate",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Partial Mode": "Partial",
              "Plan Rows": 1,
              "Plan Width": 8,
              "Plans": [
                {
                  "Alias": "integer_encrypted_1000000",
                  "Async Capable": false,
                  "Node Type": "Bitmap Heap Scan",
                  "Parallel Aware": true,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 2083,
                  "Plan Width": 0,
                  "Plans": [
                    {
                      "Async Capable": false,
                      "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af3965ab3800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78839e5375b93148d2c7258c7c5b10da6baaf6161425134bf7a220b78fc411dc6f7124f9da3212acff12515e2eed9100ed1bee67c3b98cb9923dd9cef23cf55584f226e3a79eb62b8613be6f582b0317a9e3ae5f2daa0b521732942028a34be14dd378699b686093617a625a235dcbf24b80386eeb12cfdd69516f276801b3fe1e407ecde76ec2a150503f37b82e184b69aa7f53ad67bf3a1ff155f17d92c59af076f897b200534b25aef12149f5493e8461a88084528a5d68abaa1b909e1123e5ccd90884a5e275f560bc74a5707fb45480cb2b36bea4f2551a126d492cc6080f9669a1ce78b14c3569d308bd3e41cd260cb2fc4babebb899dd5d600eb1ada01c0e6ef0b5bcce92e578523f502b6d70c8f0544ab8f489807a8af4365285b5d2ac9405c6cc413adcb8d0f332a38b28adf81276762d27760b68af527104b0bcfb12c5\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                      "Index Name": "integer_encrypted_1000000_ore_index",
                      "Node Type": "Bitmap Index Scan",
                      "Parallel Aware": false,
                      "Parent Relationship": "Outer",
                      "Plan Rows": 5000,
                      "Plan Width": 0,
                      "Startup Cost": 0.0,
                      "Total Cost": 1598.3
                    }
                  ],
                  "Recheck Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af3965ab3800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78839e5375b93148d2c7258c7c5b10da6baaf6161425134bf7a220b78fc411dc6f7124f9da3212acff12515e2eed9100ed1bee67c3b98cb9923dd9cef23cf55584f226e3a79eb62b8613be6f582b0317a9e3ae5f2daa0b521732942028a34be14dd378699b686093617a625a235dcbf24b80386eeb12cfdd69516f276801b3fe1e407ecde76ec2a150503f37b82e184b69aa7f53ad67bf3a1ff155f17d92c59af076f897b200534b25aef12149f5493e8461a88084528a5d68abaa1b909e1123e5ccd90884a5e275f560bc74a5707fb45480cb2b36bea4f2551a126d492cc6080f9669a1ce78b14c3569d308bd3e41cd260cb2fc4babebb899dd5d600eb1ada01c0e6ef0b5bcce92e578523f502b6d70c8f0544ab8f489807a8af4365285b5d2ac9405c6cc413adcb8d0f332a38b28adf81276762d27760b68af527104b0bcfb12c5\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                  "Relation Name": "integer_encrypted_1000000",
                  "Startup Cost": 1599.55,
                  "Total Cost": 19587.07
                }
              ],
              "Startup Cost": 19592.28,
              "Strategy": "Plain",
              "Total Cost": 19592.29
            }
          ],
          "Single Copy": false,
          "Startup Cost": 20592.28,
          "Total Cost": 20592.49,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 20592.49,
      "Strategy": "Plain",
      "Total Cost": 20592.5
    }
  }
]
```

**10,000,000 rows**

```
Aggregate
  Gather
    Aggregate
      Bitmap Heap Scan on integer_encrypted_10000000
        Bitmap Index Scan using integer_encrypted_10000000_ore_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "JIT": {
      "Functions": 7,
      "Options": {
        "Deforming": true,
        "Expressions": true,
        "Inlining": false,
        "Optimization": false
      }
    },
    "Plan": {
      "Async Capable": false,
      "Node Type": "Aggregate",
      "Parallel Aware": false,
      "Partial Mode": "Finalize",
      "Plan Rows": 1,
      "Plan Width": 8,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Gather",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 2,
          "Plan Width": 8,
          "Plans": [
            {
              "Async Capable": false,
              "Node Type": "Aggregate",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Partial Mode": "Partial",
              "Plan Rows": 1,
              "Plan Width": 8,
              "Plans": [
                {
                  "Alias": "integer_encrypted_10000000",
                  "Async Capable": false,
                  "Node Type": "Bitmap Heap Scan",
                  "Parallel Aware": true,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 20833,
                  "Plan Width": 0,
                  "Plans": [
                    {
                      "Async Capable": false,
                      "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c54c4cd50f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12bbb0bd496b07fcd3fb5f6023b99f693a88b0a0e8c51a1c10c2af20a0e7cfc0a7e40c25d147a0ac4821b8eae3d8d762b9c22306bb84f2ee781c0cb8f7d53003060948ab2ed3082927e8862e763fa2342a844ff626ab7fe272b282a43d4e0c68d8c0d6a3874b55845ede4a7703ac1cba89ef0a4218b287cc7c21458d042a0664196c4eb69769dbaa42e692a0a14a23523453cc5186247a554401d396c8659a21420c6d22ea18e03dc07820a4dcf1c15045cefd19f78505615b38ea28d297f05fdda6da1ea2af15e5e09a5adb5f364a00ef6f72370675bb96f4c0ac07a8f7e56a5319f20543b1978620990d71c1089600ce66d6121202d28663652ad230098e1da5856d5874f2438c3a87f3c5cac4d906885bc95325fca5794b131cb8590ff5081e67ec424fb85f94a5c243d8589487a0aa86f4fc09859e72eeed9ad139336d9112\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                      "Index Name": "integer_encrypted_10000000_ore_index",
                      "Node Type": "Bitmap Index Scan",
                      "Parallel Aware": false,
                      "Parent Relationship": "Outer",
                      "Plan Rows": 50000,
                      "Plan Width": 0,
                      "Startup Cost": 0.0,
                      "Total Cost": 15963.93
                    }
                  ],
                  "Recheck Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c54c4cd50f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12bbb0bd496b07fcd3fb5f6023b99f693a88b0a0e8c51a1c10c2af20a0e7cfc0a7e40c25d147a0ac4821b8eae3d8d762b9c22306bb84f2ee781c0cb8f7d53003060948ab2ed3082927e8862e763fa2342a844ff626ab7fe272b282a43d4e0c68d8c0d6a3874b55845ede4a7703ac1cba89ef0a4218b287cc7c21458d042a0664196c4eb69769dbaa42e692a0a14a23523453cc5186247a554401d396c8659a21420c6d22ea18e03dc07820a4dcf1c15045cefd19f78505615b38ea28d297f05fdda6da1ea2af15e5e09a5adb5f364a00ef6f72370675bb96f4c0ac07a8f7e56a5319f20543b1978620990d71c1089600ce66d6121202d28663652ad230098e1da5856d5874f2438c3a87f3c5cac4d906885bc95325fca5794b131cb8590ff5081e67ec424fb85f94a5c243d8589487a0aa86f4fc09859e72eeed9ad139336d9112\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                  "Relation Name": "integer_encrypted_10000000",
                  "Startup Cost": 15976.43,
                  "Total Cost": 195823.06
                }
              ],
              "Startup Cost": 195875.15,
              "Strategy": "Plain",
              "Total Cost": 195875.16
            }
          ],
          "Single Copy": false,
          "Startup Cost": 196875.15,
          "Total Cost": 196875.36,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 196875.36,
      "Strategy": "Plain",
      "Total Cost": 196875.37
    }
  }
]
```

</details>

![Query Performance - ORE/range_selective_gt_count](query_ore_range_selective_gt_count_chart.png)

