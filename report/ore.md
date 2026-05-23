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
| 10,000 | 10 | 673.19μs | 25.67ms |
| 100,000 | 10 | 646.97μs | 26.08ms |
| 1,000,000 | 10 | 745.90μs | 28.85ms |
| 10,000,000 | 10 | 684.58μs | 25.71ms |

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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb281fcbb2148847646358ebf64f5c300bf943c9521f0872d0c37afa56c2a6694326cd58fd510f7d70396fd405fd2c5b46ccc2b6c8a7782d85a2788ce9a2ee9b931635b472def8b68bd444bb07226669d1ab381ee59d9cfa845ed4a99e26f0a9ef632ed033fe2ecf801ce3b89daa8d852209b6a3ea29046a6df0ff5e2d25ed1dd02bd89de4b0ed6e1c8a5d2babbb6b3b58f03af54d2edac710996d6f5939b1f20ecc8a6cac9af57947bb39bab2902d8d420ccc09dae0e7dd42d7ef27556980da5c6978bd64de9fac2e739070d6852b8a67cd18d0c26a95bad4c31553e8f253a54d13a86acfe52190e4149d53ffae25127be7720e0e759228a0edf15c39dd0fd82a392f430bce5478f91378b5ab4a03d9e24a\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5050,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 7842.5
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 15.53
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28ec053174beedc51fa1f08ec19a719469a3008186fd2a3e3b3f3d28688454883d0ae8fff672d4bfaacb21ac296b2c0edfd2b3558eca3a747cc5681362f706329486b387c5f15d3a3a6b12e4ac3d1ad486e62491a8891d0287493d2c840865d6c663700e0e4160c250c0a249963540cc97ddf1e0526f54504312178c67f5e98fd2c841837052015a6dc97e8c0961191d11c9b8d3f2cc8541de4b017be874311a910249ef7a5c6aab744d88dabb64e4c4960fe9435cd42f8db0ce9671144cc1a386a1f0735398af99b71fa6b9a3e9098033bc47feafb1fb0dfe522f8df966ef8b340530eeb9c7db5a054d2347b6dfd64d1917450b56f60ba42f9104d325b9773305f62657a1ff53b6df7c2688f354e49edd\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50500,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77969.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 15.44
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb283d9c30243752b82ecfdf5acc3b0b6b4f810b693a7c93365af6b21ecf76a1c180c06d95a025b20af59aa6050de275f7596d1d1cf6dcaac66a94b0e4a359d3c6c1fa5aa01ca97ede41cef814025baaae134132c307d2b05748452c49c459b224543aeac77f27b247245b28d361b642bbae0cbcb7331736308bc69ee5d50970073dbb1163da19ebe39d2b4be259f642fa62238f322fc2675c834367bff640003262692e6201ae2db6d36ad2acda8260f3c47ccbaa01cf24c0efac7fda64531d33c7c8ec847f981f3ea0cd14f06bb6de18a1c3a4fbae38b5f1a5822ded8e6016e74f5af02f797fc942158133ad64df6ec1a3dce9b72c5a114103bdad9213b3a45ac167e25fa2e929e559dca6040aab22bed8\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 505003,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 779173.81
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb2897ac1542b4b99862268255ebda5e8065d38b89404eb47ff3a393dd7b66f3db65c6882a06fbe3cd72c23b45cf58c8a9eaf7e668da5d15c77245c88f33d1b5871c2f9041c7f540ad85a5908bf2a45585c1390c194e7ebef036f8314882c659eaa820c3f2e3bf662c27111f879ef425fd615bc45d4d78a6e5a4347954339ce2f920bc6b13c0a33968633420d03d9483fa52de1710be58ed2de53c93e56f03f4aeb52be8550531aa749765ea2d81c48e7940c0789ea188be7a2bc76b1f15def68591eda587eb68f41365c8ace088db35d2bfc4826744f6da7118054206d22576116de50fcd9579e4c4ca3353eb28fd438beb690f5c589e866753b279a344cdb29668784540fda6ebf2d5f6467f1b24bb3e2e\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5050158,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7791314.62
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
| 10,000 | 100 | 4.11ms | 42.95ms |
| 100,000 | 100 | 4.23ms | 41.51ms |
| 1,000,000 | 100 | 4.19ms | 45.50ms |
| 10,000,000 | 100 | 4.21ms | 38.58ms |

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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28ffb1ad63b61819bf176c607d457d8bd5d427c83cb41cc943c363dc801b57a8276bc2006cafe5b1d7e74b8e0a95b744cae9c810bdb184cda4c87f3f16a994ace406df29d0b54b76c10797379ddcd2c453217fb0a11ae85a1b2e926fb158b9fef4c9611538d331d5b20bb0c71fe00e218b89cebe1de59c9579cdb8bdde5d6be1cd47e002332afd62216a06dc106fad912f6ad6d24143ce9c10f4f0b925fb7323df488a4a6cf91c9b3701ddf180874ee2b37c11dfc4bc33335576731c09eff61b6f3b130c8c8e2d38154a915b18a6d49ff8f9a4a5dc98e96beb762abf1dc48d9bdcb918aaaf7a357e09a6d5f0292bb696601182dca537843f69db5bf2e607e9132cb5fd511e046a64d1287f34f8771f99c1\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5050,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 7842.5
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 155.3
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28ab8784d94e9c98d431dc8b3df3aedf3877824a2b9567b21858fb3713b0587de942af584242a56c627422d611e9633b98743697e17606f8a58f63d48bee461f6b41d3b43e989446d4b558eba8b13e0b4d4eb124a2c595db2a22408d52103d4e532098ce5b5145622b34b1952d493fa38478dd05c11d47c3f4a7e809972afa67bb9713c728bbb76d2eae28e329f77deed48194cbc4d00eebecdfe4e70d2dfff1e0152eba69f8306fb99818605aaecdacf47b9bacc779c140631baf03fd70f552e53189f297154ad87488200d8deadf583f657ca1282e055b2ea133fef7130654f28e0c725396f150defb098dafbd08911e78eca450bc49144f61e629c4d6e873c4754f39a5aed63aacd2d8dbed4afcda09\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50500,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77969.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 154.39
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb285d503fed50ce404d6a9da5e3ee2ea29bae68c45d4f1f20659c61cdb8025c62b0d36fc260c7bfc048124bdc6df0e0cb2732754a54e2c705e78a45d02adc345d6dc5c557fff037661fd4c8b73b62fb374dea84fe05cab1d480e06312965938500269bc21c41b084791ca7fdb7edbc6d7dedd81a620e15bc36f51716db9b0b41e7130d2bbf4fda8e6f63b8fe197e7be75949868cdad32bbf31f00efeeb5a1e48fd7618b29d8e193af9e55e092d7067eaefa7501099fb4661b0dfdefd793ad84d503498f5592bad6eff6aae8cbb58aea5ad06819efa6dea56ce071fbaa4ff9bf5236a3e4f4fe3180d10c898169ce95adf254633323db5a45551c1b39ac20c740045d6ff8abcf6bde90d5e834854c43c472a8\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 505003,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 779173.81
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 154.29
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28bc1bb073be7d6c84aad2e54619145ded1fa384ce72b9f1b79c8e03f8b4c21dd65406f2167cae5a95ae387e2d4d334c6c07c781195603920e739a48fd85333aa7d17f8a92f6642f247033d9e28479976a8fbc0e6ef9f32dcc4d3a379a671facadf823406aac2dc84f0d808f78a7604d805af81fe0c41493ce4f7a2530f04e2801ba247dfc2e936718afd633697f0bd653a8e5da7e6cdbb87d64719f130328ab70b53f76a0c6485041b121ee218eb743f0f7d1089055f45385e7d6f0854df89cfbd26362362db8001f5f5c4eedd897ee2629a32cbe4281a4b10853c549aec99db7b657eab0a03b5bd5206fe6e36ae7aaf657832e307b545a20e408e509d2648e3147769f3c452abe9117d7db110784ccdf\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5050158,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7791314.62
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
| 10,000 | 10 | 630.29μs | 25.39ms |
| 100,000 | 10 | 719.61μs | 27.00ms |
| 1,000,000 | 10 | 595.42μs | 25.89ms |
| 10,000,000 | 10 | 583.89μs | 25.86ms |

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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28dd49b24f014e20192f8068ce4d63f6cd341e9bccff0443aca6a863d7e2ff09dc62950ef0f8bfbda74c4ee2e0832d4d0e525ccb9338e2bd616c1f8e7bb9da04a4697ff0a0e706724b9eda052733ec856a2d31150e1da842e7d01e5f9f74ad0c779edfe3ecd31f3d16f7785972955d584ca4341ab487273299af3e8105618c282c762a7a1f179e7a10d23fb1aa96cb6994cde43f0644eb29a025e9f2111ced7d91aead3d424fd85137831c26fbcfa7a4f5e0c62b9eeda0419f42d64ea4f9a1cba842f652a1fd5bcca1cceb3039a6c5952e8f0add1efe0350ca6a8220b74d581c943e7d8bf84b480c0c0a3f405c384644b9f443189f48f28405d3151eff5e3994d5e36d2876a180a5dbd06a3ad839c0a87e\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4949,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 7817.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 15.8
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb284f32e07d9ab75770e561b8e009f205e5163d630328d710d99f2639261d4c6c5a6ddd86001873563a73ea587a514bcaebdfe9ff97bdfa082ddb10bccbdb37bdb04fda6a66efb116f2bc39b92f98b838094c1fc27a7c700577d073e5e1bcf8b31dcfa2d4147caeae87a0e58c85808ef4d3e0279dc8bdaca29164fcd3e6a0ad6f50094f85b024f5703c242ed35acfe9016a46725006ecb836f652c0f8c9c09750abea5e5e3ac16c0a749217f10d1533302bf5023841aa5188f2d7e5298fbcea3a5e706b9faaf41abd8aae92bf8a8c0d8890a63581ac75634cc3c1ed149a3714cd8a53d119004772a0b47592bb3bcd91a4f971f7927f339366bf41b489bfec7a42ae60c501e0454a4800f93f56db653b1dd2\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49499,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77718.75
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 15.7
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28c940b28f49d11f5b2aa7fd5b9cb4f9a6fb28d53173db721edb4b7fae6cb11bc00d121f2ab923526d4861c91dc65ee44e71c98b5977b347e44dd11d423b6d151f0071077e92311efc3518c3f53ae846a2b290e849ac35aa2aab4cb4fa6ffb519f2811efb7f1b2791a88911a2777b78ca48283a7cf3f07fba1204e99b90e467330e70c9b92fe2dcaa664652959ba09a7d0b0ade8f991730309ba46b63a60b41d7fc969ded6e2c596eca28299bf1ca1c036b156ce158b4f1438dfd9ab11c3676652dcef9c55fd63f20d4dcc27c855c6e3081abcccebc9491b0a79b617447339ef2e451f296b4182a25e31ed8651134cd7f2c620ea3da55104983f9b60da6f39a8a8ef39ba52a552ccbf38f1dcfa9376476f\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 495002,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 776673.56
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28ec6af8a0b27946056adc69ed431e7f5347ce9e2d7f74b866edf60153862ed41da292461448a70447f817ab1fa021194e2eff4ba7328a9431048e532806440b515b9e0aa8e6d4c2b0f7d2f651592b7a8a0cc26f312b330c25542542e11077773d3971ac4d2093ad7b1ab2774f1c83c372c2d9c668524491fc76a956c2d6f81cbb329c97f8c681081d4251f15a992edc925c0ed4826c21e5aaaeceb609c14bb4009aed920ddac42ab176060bccb4359b0538187c52eb4e365d9e1d5cc227293f121da17374840a1ddbf481901ae7cfa8c2fd17bd745d3fd3cc461a10e9a9b53725f347e0be86cec8c25aee7fa0449c516c46a18a14668fd618bf7212c0d9cc259d6f0c559204a64a320917daf8175d80f3\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4950153,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7766313.37
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
| 10,000 | 100 | 4.14ms | 38.28ms |
| 100,000 | 100 | 4.01ms | 40.35ms |
| 1,000,000 | 100 | 4.15ms | 40.70ms |
| 10,000,000 | 100 | 4.17ms | 39.31ms |

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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb284d1ea35f9fd7d8853fdf86bb63035b368b840f7515a50a9db38177fed40b10fff37b953b902680237a453d02acec8456f6081150c19b6caa545d5504938955bbc5d331e003969b3e1c989544cdbe58505e76c819edcb62d58700bd5124782944c03423de8c487729c61f7cf2143c1935c223a7ee90a5d56ab6641dfa94e97ee6772a31ad3dae1173a11d051d5612babb2ad944d3e1ae302e4548840b8c99d6d6562ced046bb281c83ed92e55ba0dd549adc9baa489b9e8d09ee5a0cb753a1d38d36c501327bcdc71b87dc625778862cd6c2730c566f00863335d06b7e858bcbddc3082d867c92905f20c0f5c0492dca724b0f9712ee7d525d26076540bfe6dca7adc9941a7899089c5c9f0f0861f6567\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4949,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 7817.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 157.96
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb288de7dd5249e47fe7e232e0514f243c468274f0cab0f7dcc47bb9faf02794b4bd8e39afb9874dfc9f2c27f6ccdb426d1d2e1aa4bc4f0f31a3110353518eb91a5b1168551c0cc6f28a66adbae8395e013de491e5a65d6b2c7759ed9981940cae97356f90e90062ffd648ab2890c0fb94c205840c24345000709d85cb317c2c9975619fee5ff0bd67273556ad80b989ff07cedd550459060483a840e1ed2abf1994f5dfe103c15b0af1bcb07fe301bfb5de2ef1feeeb784ae7c88d93f500a73328d43fefc4dc96352e12844d966517f116e4e9eec73eb24c1dd231907c7ada7f421279f9609d57bcd251080de32f884e1b9cfdd1b1b88ca471ffbfc44aa11c3c1ccda9ea857f3ea83943e71746024c1154b\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49499,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77718.75
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 157.01
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28b661a01108dcd1418bf2dded2cc7dba72398030b31e283336ef51ccf9bc5768a50602315fbe02d1dd399c616dd243ac64b01e40dc9a3e23f13afd22c0595ee928d72b5e41463918e6f51eab217309048ae1d1c0e26546644f43648f56906f277f671cae08af426f2c3d5d81826dd854611072c79fc2bebbb379b268b05b2658c97b55b52590a4407cd0e111388cebd7bacb8862df21fe00129d7ad03e3ff09ab0fa7271177ec1af829d409feee061873afdf1f53a09fe859d35a9d9028f678b9c7c9a948216b2329d9f47361d6036722e44226e8fa0c2d91054a7835c6079fd88c122072ba81e0632ab65c0fe32723017769522b79c06bae411308e8fa9326325ab5a940f851f1e5927492cadf3f0237\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 495002,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 776673.56
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 156.9
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28e693a8bcfcad2caa016b4b0620ea1628643c0fd45029f00223aa20192dd242dbfcb9bd6133bfda6addeadd925afd9d7c7e62263b005c14f1e991295b085b4163519f8f9dfaac335c35737745a73befbda18f8a07d6cd8a2efeae84e688063e5df9d7c364605ef9a48793a80ca6e90098eb6d2b0f40f2ed0333498a3a8f71e781ef3ae1b879c8048cb2dddd13961c1bc1722bc1d3c061984efaaa02418814cd6131c09166b2732af6d05b0190ccc53f16bfc18a53af516d9e4631fc6954871ac9f8e368e8316cb960cdb3cf965082ed1a2299b3b9bf5a7603b622ddbc7daa284b4c4a406680a1b88d9ef36fc7617153d7c855e8ef0bd34dec3b7a197af820b6d6b1d799006c356ae2faa864a66f6e8ae9\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4950153,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7766313.37
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
| 10,000 | 10 | 544.12μs | 28.26ms |
| 100,000 | 10 | 537.12μs | 26.04ms |
| 1,000,000 | 10 | 508.27μs | 26.22ms |
| 10,000,000 | 10 | 528.17μs | 25.26ms |

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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb2813900e49296eb048a5fae4410a653f0ea50522e037d65106b129f31bc4c49cb226ef833caa560ac58fb61b7dc2c1ea1e1731e5ffd4aad1383c104f34f18ec9c17d2606b09a3424f4f0011da19be75ab6e3914f60da84176d14991329cea8708ae7e6ad8bd7cb72b015b6c11569f1bdf9bee577176ac2bb7947d65cbff320250ab3199c97a1947fc94e5d67799f968236ed7733ba7356d4255f74a925a305cb5d9017278dbd77bb46d838070c78501e75ee2a7bef0d128407c06dada7e25d1a246059dcbc2059cd365a71ce039a52ac0fed401a12f5281e1da22feaa6681673a7aff89c4e04abae27e5bb35d5e1f7c80e5e26604ddb38e57d7a6f59b0088697d0e87946dccfbb983c8cbc5041944d7f15\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_10000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4949,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 10033.63
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 20.81
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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28f9f6ee44511990d3590aae516351dbe589eac5ef4092456f6f23abc24fdec8f35f88547c01bed0b7145834d838c98f485f2a60d917938945476101781a084006b7b141446ef6d4f3ef24a7f9c86fe9bfe6716a7ee1eca681b54c2648e7242529c7d54dec3f9572ea7536958d4307d97317c8b22849c1ffe0708c44002544bd43dc0fdba97035ce6bb86b9929fc7fc3c1ddf49173564a96409feb7db56cf71120465d9f67056273de471890c786726fa635d5af4595776971feef02224209cb897ce77628945d30a69425e23d24eea5a036e413ab2746a82683b44e0cdc54eae9ebf616b9044afb1ff4da63c077efc5a5b321dae4b4f9d1013488b116c109f0002087ef0664cefd724746cc3bd351e0de\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_100000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49499,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.67,
          "Total Cost": 98427.07
        }
      ],
      "Startup Cost": 0.67,
      "Total Cost": 20.55
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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28e3a86a576e6b4a6634a05c01df4d5efd75eb75a63083aece0905238bd5635a3da8d89db7bf11e8bae7039ffc7e09e1fcad00f3e3ca99a3946550707fa538c3eb2cc0df1aba34755885f70f4222ce882f02b9b632bf14114ecf14747e613157c759c7715325ff4a733ed4dc1346ac066060fa5dbbe1e9ae0daf9d7364c89bf5089acd6a02b090302da19b621887929f0d67b9450467a9241692dd3964d3d3a4638e82e4fd15974af0f61522d5d7dcb8e7dd4eefcad298c05276f27c2bbd19423225badd6f8660d68992a73870258650797a973e363b0503255a212564834cb2695fe4885c83ec88e023c92b4c15fa5d43d56bdbdf6b3bf53e0536c8a56d298bafea7bd48aee9449402f1c5ba395a0edd6\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_1000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 495002,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.8,
          "Total Cost": 982139.12
        }
      ],
      "Startup Cost": 0.8,
      "Total Cost": 20.64
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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb281de1bab518993d4171905f68c4b3209fbe5cdb4cdb40b97efd9e587f46f62b87dde0452bc17e8c49003a30422e7ccb219296961a9b0bce8cc190a9840e7b9771ad55a38ebf72f13dabc5e28d86bc5587a1f97084f3e23e11ea817e95b533d7d7bf90efe7858af0bd7ae44a4e07550991eb7f344fd340c0a72afd026ad4658a4da31500374b86e89a872316c8f7ea3a31ad29221726f5d277bb2f7c1d0d0f546117f7d6872fc22341f497f89cc9d38adb3053c9fdd1e01f70e78bfa98e683819ac3290f3eb16fe46b552d9c876359e5f2b39b9d08d301469484739c0fc9d862c9ccb973c9803c154e224bd79e5e41b09924452cb1a2f347b1b65d3f6a1e555886c2ad19cf44dc2c0477b7e421136b804a\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_10000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4950153,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.94,
          "Total Cost": 9818767.58
        }
      ],
      "Startup Cost": 0.94,
      "Total Cost": 20.77
    }
  }
]
```

</details>

![Query Performance - ORE/range_lt_hybrid_ordered_10](query_ore_range_lt_hybrid_ordered_10_chart.png)

## range_lt_natural_ordered_10

**Description:** Ordered range query (natural form: column in ORDER BY)

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 ORDER BY value LIMIT 10
```

**Parameter:** `5000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value < 5000 ORDER BY value LIMIT 10. The sort key doesn't match the index expression, so the plan keeps a residual Top-N Sort over the bitmap-scan output. Post-EQL #218 each comparison in the sort is the inlined ORE-term path (no plpgsql dispatch per row), but the Sort cost still scales with the size of the post-WHERE set. Companion to `range_lt_hybrid_ordered_10`; the cost delta is the price of the §4 sort-key shortcut.**

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

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 11.89ms | 36.69ms |
| 100,000 | 10 | ⚠️ 197.64ms | ⚠️ 116.95ms |
| 1,000,000 | 10 | ⚠️ 4.969s | ⚠️ 909.61ms |
| 10,000,000 | 10 | ⚠️ 61.696s | ⚠️ 26.193s |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Gather Merge
    Sort
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
          "Async Capable": false,
          "Node Type": "Gather Merge",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 2911,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Node Type": "Sort",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 2911,
              "Plan Width": 36,
              "Plans": [
                {
                  "Alias": "integer_encrypted_10000",
                  "Async Capable": false,
                  "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb281327270ba0aaad814507c2a26347b7dd804ff341b605ac8b1bdd4f2834e8f47d8d4e71e159d9a31b02c9aba5641ed95a6eb1af75cce2339b5d8995543a9a162529152d21eb6a44915470f7915bb72e6cd170a41a714cca54407418883bdf735ac31d95aa7caba2427e647f5fd2de4d2af8964f5723c5ec5a70cbf55356b78424da95f580792580d7e8564cfd0e17b94eb13fe5378821f2e00ae008a8fa55b08dd9a335a4b4af60eea8709a067de084fd51561fc2a68d7c8f5bce7a4c98c8519b3821ec17e5b4cb3060a5c3831d15620496ad43a732c97b274239481612e3895812bbb2d719af221d7c033b483b0cc98bcabf8e673758bc61f35f595d00467ba08caeac0a0aef21890769cd24e0e0c426\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                  "Node Type": "Seq Scan",
                  "Parallel Aware": true,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 2911,
                  "Plan Width": 36,
                  "Relation Name": "integer_encrypted_10000",
                  "Startup Cost": 0.0,
                  "Total Cost": 5207.75
                }
              ],
              "Sort Key": [
                "((value)::jsonb)"
              ],
              "Startup Cost": 5270.66,
              "Total Cost": 5277.93
            }
          ],
          "Startup Cost": 6270.67,
          "Total Cost": 6605.43,
          "Workers Planned": 1
        }
      ],
      "Startup Cost": 6270.67,
      "Total Cost": 6271.82
    }
  }
]
```

**100,000 rows**

```
Limit
  Gather Merge
    Sort
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
          "Async Capable": false,
          "Node Type": "Gather Merge",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 41250,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Node Type": "Sort",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 20625,
              "Plan Width": 36,
              "Plans": [
                {
                  "Alias": "integer_encrypted_100000",
                  "Async Capable": false,
                  "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28c7d451c1383c3544400dfdf3be3494d1953899b0b0c16813d68db621d711f68a26c45973ba7962313f451c3d07aaa0857a8a3af3aa9acd3d1b62d036cc3d7f6db2e491e0aef8cab7a700a8efb6dfdd85b82285b1bdc57e9ac11f6e7ccf00ed1210b59265b46b3e4fb10340d349b3220323095d05bc271522a007e3c6b1a0a81e0b07aa910466a227849cbf32a41a8d5b9cf1272989e5eabaf877bd27b2865be8a8160f6a039d7ddf9bef3a28a35b4e5d37af3c13677fbf3e6761fba52c37e6885a1e5b77b65a0bef04b1649c00a384f469cf54e0b06f80d4d1f2f0ae171ec814f852dc800e06627ccc4ec164282dec106b94f87658bf1b286b10e88b123065bdd42f0a146157d1c87cbc3f0f4a760fda\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                  "Node Type": "Seq Scan",
                  "Parallel Aware": true,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 20625,
                  "Plan Width": 36,
                  "Relation Name": "integer_encrypted_100000",
                  "Startup Cost": 0.0,
                  "Total Cost": 40750.25
                }
              ],
              "Sort Key": [
                "((value)::jsonb)"
              ],
              "Startup Cost": 41195.95,
              "Total Cost": 41247.51
            }
          ],
          "Startup Cost": 42195.97,
          "Total Cost": 47008.81,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 42195.97,
      "Total Cost": 42197.14
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Gather Merge
    Sort
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
          "Async Capable": false,
          "Node Type": "Gather Merge",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 412502,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Node Type": "Sort",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 206251,
              "Plan Width": 36,
              "Plans": [
                {
                  "Alias": "integer_encrypted_1000000",
                  "Async Capable": false,
                  "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb284296f95efea3e839d79d517545354d7acefb9b74f6fa620dcb72b967343d540fad29d03145a35a14ac094decfefa40930be9c523677d8fa8c3491d42ca746ed4c66ef914c2b1e6203f1e8c52f497ee221529b5497ce961419d6f868d19ed317b7f6137600793b2b577904a56346d87bb1aec198071dee715ced77eb7e913ea6b8dda2c7481d5f759a541c6a0d829c6367eaf1d501d95a320937ec0bc28177721bcf068d49f6e8d8c91e8749cfe0818201b46d6fb7f4ce24f4705abb7b3741e7d0782ba7c21a9497cc1894c996cdbe3beb2f6ff6ce9988508f9f49b5d28babecfef8bb1240d1e8c11f742e413df6ef2a738b83b4b6935e9d2bc74702ec11e62221eb6ce3b6239e83e1a1caf66eeb58024\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                  "Node Type": "Seq Scan",
                  "Parallel Aware": true,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 206251,
                  "Plan Width": 36,
                  "Relation Name": "integer_encrypted_1000000",
                  "Startup Cost": 0.0,
                  "Total Cost": 406984.03
                }
              ],
              "Sort Key": [
                "((value)::jsonb)"
              ],
              "Startup Cost": 411441.03,
              "Total Cost": 411956.66
            }
          ],
          "Startup Cost": 412441.06,
          "Total Cost": 460569.65,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 412441.06,
      "Total Cost": 412442.23
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Gather Merge
    Sort
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
          "Async Capable": false,
          "Node Type": "Gather Merge",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4125128,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Node Type": "Sort",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 2062564,
              "Plan Width": 36,
              "Plans": [
                {
                  "Alias": "integer_encrypted_10000000",
                  "Async Capable": false,
                  "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28f9953ade6fa1ef69804a6fc900432212545f66c27abb4a5793c2ed8f214486d34c0c2784a0d87fecf71791a808d13a76e785bbced191cd1dda5b939b9e2b6336e42a2bf306e33a728a6ecbb330a08f65b612aedff8535a7cfd9fc11f8e93bc5860d8a4df7deb96528cfb84dab5121e2661f3cb9e7a97397d1b3638b5818fe1dcc6e2231e0899639c34a0495f128bb24811b2abd3cbc0d7df3903a2a9e37722c3ab08903098e0aba9d133a5da385a3649bbe2ea872869a8c375c6338b41f0cb8020cf30e9c27357f36373a6e6aa44415ce3553289feac555e3e0c673158356c6f48399b8a08ad5e575c6412bb3ab4b6fa23bae286ca573a4becf4e4ca4a7680e01f7103e02f6cfd18ab9c284c26718a5a\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                  "Node Type": "Seq Scan",
                  "Parallel Aware": true,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 2062564,
                  "Plan Width": 36,
                  "Relation Name": "integer_encrypted_10000000",
                  "Startup Cost": 0.0,
                  "Total Cost": 4069323.3
                }
              ],
              "Sort Key": [
                "((value)::jsonb)"
              ],
              "Startup Cost": 4113894.57,
              "Total Cost": 4119050.98
            }
          ],
          "Startup Cost": 4114894.59,
          "Total Cost": 4596193.13,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 4114894.59,
      "Total Cost": 4114895.76
    }
  }
]
```

</details>

![Query Performance - ORE/range_lt_natural_ordered_10](query_ore_range_lt_natural_ordered_10_chart.png)

