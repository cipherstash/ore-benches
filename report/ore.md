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
| 10,000 | 10 | 629.63μs | 31.20ms |
| 100,000 | 10 | 568.25μs | 25.67ms |
| 1,000,000 | 10 | 699.44μs | 26.64ms |
| 10,000,000 | 10 | 605.03μs | 25.46ms |

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
      "Plan Width": 1096,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb2895058ceacf03e52bb407fe49acfa3b16b0ac37b95cd2d7c2de61fe1fe5c0653f7403fbfabc9e15ba1280c66d5cfbe9271bceba052138a29b6b9af48605878bec30891ce20d6c30dc13b435e8b698b06f1f10e88cf6bfc3d374fe394c575c12e87b18bacad46d3397f9ac10b82ec8becf12be4d8c253c2690718699d1e9d9125887e6d2413f2f4fff0e8ce299a3435f7adbe089c5749adda2b589569bcaf8b8c0e589eeae06a9565a4dfbd92d03c3efe15129d6b4f0a9503fd37f84516d96d67e313189e1dbfa5c6c969b299fedb83ad31b7c075888ea9a0e5b8b97d631aeaa16efaf41bf9eb702562c8e57d7f2e2e180d59b1179249bdb33c845146c1a9649cb7058859d3bf96bbefa66227e87acb5ab\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5050,
          "Plan Width": 1096,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 6580.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 13.03
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
      "Plan Width": 1100,
      "Plans": [
        {
          "Alias": "integer_encrypted_100000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28cdffed476d8949ad004e8503eee2d1ede277e72680cb4f49336cfc2d27d4d3d87c0d2b188c5667d06c772cd8943097e6bc09e35e82cd7eac79ff5f3601f84c13822ff353bf058a28c334a9152c678ab9f603654b31bbf1a670be0f43714bc2fb662770684b98854b4eee3e4d97b02ea06fd7daf2239bea2e3cbcf9522a5c452bae4b25f21a91080615a53f71d1b59525dd08f87cb9bcd6cad1a6eaad71c123101d4d9483ece39053b5515249f666037d3edaf4197fb129095640f52b1b0e981fa00af3f0ba265e155bccb963fc0afe38b99146b9e7a63813c896bbdbb795e99038271c6047475734c28a135e2aa1b72c282fd257cc7f45412f70e476f63bad5adb65cb1461b84caec0f8bfc07d92acd0\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49500,
          "Plan Width": 1100,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 65344.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 13.2
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
      "Plan Width": 1100,
      "Plans": [
        {
          "Alias": "integer_encrypted_1000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb282f84fb8bea947267022c54168884f13b299cb13e75fd502a59e9eeeb2841560bc59c69a676482a044359edbc2138c8f381eb13e4b765294ab3eaafa4751639312ead32fde25cb182fbfdcabb32af14a7e3a45dc406285c52e6e3a0ac1ed955d59fb786dd85b6c92608eab5a6f556e3aa9f401ef39d3453e149b01af85327bb8aa02bd78f191f969f1474d04917ec8360c9aa15a7d62181d1f0d29c5b90c6272c8668287f38405b6832c2807a362201945ec2100cdc5d4fda682f40eb0b891c39a3c2accf1d8fd685da9a5853e1c168e6f9c921acbc19ce840aa8f093228183d84c20f72c67758110a35bb1c9d048e04a8c9b476452b8784a4db734f50f0c72d735fd69e8f16f262ce66fe0815c8cdcbb\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 505003,
          "Plan Width": 1100,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 652923.06
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 12.93
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
      "Plan Width": 1100,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28ab3526a1f71312dd4b4012a56a854f8b6d50e0865458819e6ddb7e5a26c521c2351887d2b69273ee56010754f2e8e30383a63df6b667c659cd59c6e8fc5925653beafc5f1898c4d0f149f230b11ae10083b8316ff49fb1be201bab7464f3695804ce459307906047dd100a885fc13b467ebae0fbecffc31db1f56cf8e177b9ddc24bac3871f14f8475c446c4d0ea2b13d6e687dacf1d6f2647ed35d2e0f009f8ed3d843452ca2df663d840bd0d2dd90a11fadd0b32300a245b4ad2daa679186eea23a0a5fe20f789827344692613214cdc5d901715d26293a64a2ef96d0010dd7136981dc63ff16764bd1c6f5b223970f1ba23dc3855b0e2eea32d484436e2dba077bc5da5537f4fb5d03c23d011184f\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5050158,
          "Plan Width": 1100,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 6528775.12
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 12.93
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
| 10,000 | 100 | 4.08ms | 46.72ms |
| 100,000 | 100 | 4.19ms | 42.46ms |
| 1,000,000 | 100 | 4.11ms | 46.26ms |
| 10,000,000 | 100 | 4.06ms | 45.98ms |

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
      "Plan Width": 1096,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28516c2cd5946c69af275e35b4d463d045361a1cf588bd7ce00b458793690af709d02b69d2e6752be608e71d0d74b4c86de79bffbb1a2eedb0d48d31c5e25f35c3d7b0b6d0ebb351be2de001a2e98897471c2de5ccef4c643f000cc2aa4a42b34d9ac46e80fb5e73cdd60cd305f2f65e3a825c3520a9f6700f3b3f332eb1162f56e805f3e43fcc54b611217b120bbf618dc909865ff5ae4724123e3f73465b786fcd127477cf4e0559e500c4fa8e0ff35081ba73c3d5a34a5da3137c37450ddc9fec6050a46b1a3876200fd83a21b7b84f683d2ae6cb0a1e648a227a800e26e7f7071d0a9851bc3c5eca490ef0a51102eaa215147b4f10bef5a9a88475c3fa128476ac2d4061d2f29e48ba5bdcfc772539\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5050,
          "Plan Width": 1096,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 6580.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 130.3
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
      "Plan Width": 1100,
      "Plans": [
        {
          "Alias": "integer_encrypted_100000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28fa32673493f0be9a0aeb3ba6c3df4f09b3aeab46f298065b776269777c3cf9a42c654c6745db2642d462699cb93ca8b476e2f4662fb7cf9056f5c67c34eb6fd472e5f3a395a4625a56c286821eebcfa6324006b3abdf6f21cd1dcb4708ceb9680f2363fd312604c638f72e3db772996c73df4b980caddfba6941d3a5944928755ff7cae1ba838a7ebd4fabe18f44a243d6f45bb097f77be87c981b3dd3a517ad71f9bbb0ebdf929e8161b5d0c4f7567e6d60ddf06e5516047d814f873fa339e4b4e3b641085e826fbd534cb79e0aadc6b3b96d25533fb166e88205cb32171bad7648bb4e5eb7dcc55b6b73beffa33d5caefe093e86744da6b82947761a9ac38f51439bcdb1f936439295d266b560fe08\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49500,
          "Plan Width": 1100,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 65344.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 132.01
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
      "Plan Width": 1100,
      "Plans": [
        {
          "Alias": "integer_encrypted_1000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28e35161ead6beffd7663345c5318f66d1601b178b8b64bd03b9cb417932b68f4923776a54650ab7182e808668cfaa90543e34e7e4cf6d8dfdb524d45e3bf38f06e99980370109eff79345e4b9389ddb30026d6686c9ec5e1d36452c4b8b48dd5f6c4fe91054ef5518e2ceeac0d04695f32b16b4b5e093386bcacf6e0735402f482eb60618f0fc29f602147257485621aa8ac80a875e6597f48fb66a1895a989ca665ea38d46fccd031aa9230d997a4e6cea51831b3fddbeee566978d81f29c3732d68408cefb521676823a67cf770b733033941594d74afafc37ad96fa7be7d862ff77227be89b16f1f651eadbb2c3c3a5a034a098b9f595c07209177e3c4acae06718f5cbd99d07c4103a7b463d3f6b9\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 505003,
          "Plan Width": 1100,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 652923.06
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 129.29
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
      "Plan Width": 1100,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb289ce2631903d6108b8bf2bdf62a911654684dc048f7060858c44bc617b98d87fda06546b08c80283edeb5488db636347b2b0cf949c93dc9e038f247774433cafbcd63bf91d129b31c52a56e4db0361d4f527df7c29e15380b9063b1e04a1e43b74fc095b98b17bb6551448320c938c455cd0ced12447e4c0fc8c05d9bb62593442d628e0386ee98a9abaac940b361b10249c58bbba2726fc2475a423aa5a5e3efe320e3374ba6d247d1864bffd741ae8b3e50734c2e31558d31396840e8d90455d0b2798c225048af2de3201c962b2e8df2edd69d2e172220aedcb05bedfcbe6e3beaa69920c9900f5bec3a2fddf9d589d4e6c6558df08bd9b8f3aecca4566c9b7da26f2dda2bd1eddb0a372119661a3e\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5050158,
          "Plan Width": 1100,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 6528775.12
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 129.28
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
| 10,000 | 10 | 593.75μs | 28.77ms |
| 100,000 | 10 | 654.88μs | 29.26ms |
| 1,000,000 | 10 | 587.45μs | 26.92ms |
| 10,000,000 | 10 | 765.42μs | 28.62ms |

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
      "Plan Width": 1096,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28281514a62219c9e223b1b98519673056b9a5be40434e07d5bb2f217fc5e9d32fbee4b30897d67c09230375ac6ca91c9de55ce861c231520461bad34d8c6b4dc3f153de14b91b711829818e7037c964b2359ea8a038c5822c23bf0d35a0e68635c59c82dc31771fb6dc55614f583c2730b47cc153459a890943b9c4ce5d74e636b1293882943db6023ccdc3460d1a1647dce4a0861af21d65f33cec0528496cd813c2de31ba42d27e125ed3732c01a628229a7a3f7bc91e2f62bb333e92f4f3249478a190c117a45a03c00a1a4ea3b1b9942cd7566ee3052a2abd75b903bdab00476e90b9643e4758d203cd0e86edcf0f9ddab24394d50304f84b884d435ba096b6ed4d201d0a4315a4790152bcfd8743\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4949,
          "Plan Width": 1096,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 6580.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 13.3
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
      "Plan Width": 1100,
      "Plans": [
        {
          "Alias": "integer_encrypted_100000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb2818082cdcde08b3ec83a009cc965e2b3feb49c2fd9f35398709b0928121eb28e004ce488d915e1e277d5aaad38670a26d9f03fc6e160ad1aff6b665825d60af3b6b258ff799d72e7a9cc740da7654c2184e3044c78a660f91a093ceb87645f29e93f1b298d14d5a717d4b4aedeed621f2b4b40486787924e2eca5cc8711857806d9b570fcd8cca93af62e401c9af35f2458f83b251587508641cbe02fb3bf90c27a3e4c68ec890dcc2a8eddcfc2bf7999619122104e09064b3c12042914e24ad8fcb74fffc9268c06cc6d7093a545e14ea482fb8d85ac048edb6b882ad41d25c8c6f3e48b6f33df6dd86d3909846307fb8497181666303bfb821890299fdfbba60d9cf8ca0ed7104d78140514cdec9b21\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50499,
          "Plan Width": 1100,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 65344.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 12.94
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
      "Plan Width": 1100,
      "Plans": [
        {
          "Alias": "integer_encrypted_1000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb282e91d2b8748554723eec099dc035833f94094f04a27dd8ed5f63bd409a8139d10d2b11e57772a0e591746c4668e8dc002068a91bf70ebdb4209b1692fe59c767f676f24a9fda124826331b1a715518b9f84fa85009b12429e59c9781aaf6bb4362435caaba5dce276cab29415bada9b340e36e96a2377ded7519e8eb30527761902980d7b087bc6336365e238c478989279982e9b7d808305f489af26cf8f8512d2b736935502391c6d2551e50e5c699dd1378be1b8d2ee6b0603658cf9b7ed58b7fc21bec0493cf001ed97b549b9b76dd4157d365f26bbfb019a99d4df555e3f4fc96bc79f254b55353df22614ea788caf528980fda2831cd483fd31f30b291d40f3d8b8dca1b4716a6eaae0c6ff57e\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 495002,
          "Plan Width": 1100,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 652923.06
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 13.19
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
      "Plan Width": 1100,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28f593f1586d1251937b77bd76c4a36fc688c54b1b1760abb18eda2851fccc74552ed80c0a772f803e41a02ec7ab03c4857e53354c7c07aa846e6aa04319816826eba5a69558565a14711bd9dbe930e3ed263b3c6b370ac260c1e12699539c78cdace31999339368554c76fd77a3a1b5a1497ef2fd3d92c730057add2cb55ee475e8fd91eaadb75ed02e7cccbe261730822a2c1afa46b057c79b2201186499e7f11e88644c7bd3616379938706ce2b9d82e8be8a761ce70ec2cfe28e31a4fb50d439755da13c318c2841ac2af4743137f8398a5d9d244be89b6a38777ea4dd01a7048e5733945a991d249e95f2b7d616e763f54923b3e40770d5b7fe6eea162e19c51366bdbcc0bf674f63af3f882e473a\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4950153,
          "Plan Width": 1100,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 6528775.12
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 13.19
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
| 10,000 | 100 | 3.95ms | 45.93ms |
| 100,000 | 100 | 3.91ms | 47.31ms |
| 1,000,000 | 100 | 4.07ms | 44.32ms |
| 10,000,000 | 100 | 4.12ms | 46.31ms |

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
      "Plan Width": 1096,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb2842b46e630529ee0ad9f9da1dc9679eb9b63125d7c154536158c4bf3612c27decbdc3b89e5e239608510c120ec9fa347dd6e6e9e880ee6e45170a62e35c5f658d698b507dea059565113422ceb693ea2a38a1db6da4e6e1ba71cf3a804e364395be09331d90d897c75286796865ba0088c31dc4ad592ae1c7b577cc0bcb4b1f63cfa7d1946c25be95132827f5763c5237df5077e36fcc28d52dd7047b22f7554736e18d62a83d4a963ff23be234576d0fef6e5952c461d37f234597df886ff57f2db07497a038574bdb9949c48ea54d02c68c438ac86231763b1e7f92dfe39608aa2937b56d8bd076beddff41ee82f7c65fdc1aaa47c91e7c75a09795a50bf0bcaa5e8875702cfba1be4646f82affe75d\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4949,
          "Plan Width": 1096,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 6580.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 132.96
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
      "Plan Width": 1100,
      "Plans": [
        {
          "Alias": "integer_encrypted_100000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb2890665bb1d27a5a79962d6606881dd9ab6a557fcda289c68cf30f1028fc14e63b6f55a97b9c37995eac10b7122224fb62889debe5c644da1d1e45f75d084b47096442f2b1aac1d0a180d4312b6d817c27d94edcbe1d2903cf91993971e19111ac7c7f7080a68df266b2697699d2e68168492356a64bcbed646de191f4172bc37bd2eb4091a723621e30afd761a205d42bfe3e025ac3240fa6a4480156ececf1f07d67ba3794609a149cf502ada96dc86100fa4c3534837ce8017741778834cb1803236971bb7bcf083e88e7b0f369d6ccc32ea5f15f499b354ea7ada520807d6d687867350ccafe99955b647ebc72b4efb0157e9359419d56532864699db3ab733ae60a3f5ee31ecd52ce5c11b8671942\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50499,
          "Plan Width": 1100,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 65344.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 129.4
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
      "Plan Width": 1100,
      "Plans": [
        {
          "Alias": "integer_encrypted_1000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb280f0a220b44bf138b1eeeff29fdf93ea9b1c0992b21396f165eee66f3c630a2f747674e1ec9ac0d28832db9749c921c9fc7ca8d65c423df4a3baa3ff31e48c7f7845f720da4bfb0a30bbf533be3327d1a78135d0fc446ad691c2212fc9f99221c31d2e39bde6b6440b5805915db927fbfa77c599d926f7d2ea09bf8a91abe3143c4882cd81fc268401a57f4be401b94341bc5ec2c143cc54e465c44a1d59a6cba892262c96fd9b5a8f2d5d4b6cd606a8b6623096b570b0d3a8b4813e507f56123ce438d1a4931ad76fbcb17930c20ddc8784c234aea0de7986520080708b50cb91ee2bc3970bc071195854660b3d374b85dcc3e2f3085f0d5d77a8d2e3df905ca5e9bf6aff650c799ee2fa7f67a5883cc\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 495002,
          "Plan Width": 1100,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 652923.06
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 131.9
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
      "Plan Width": 1100,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000000",
          "Async Capable": false,
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb288760bfdc5090d4008746ffbbc9564c0557b9a0e0f6b3e8e756c45d0b080540b9871bbab03a4d65bc35ccdb62515535cfdcc6e8eaed2cf489c23a23985916b1c73381a4dd970edaac9111b97e40cc5de333d40f0db178d987d6b1343e6c2464e5ca2f06ec4203ca875b7ec69be9d25822bd2d9d9c9964afabf74d9c6e2ade4d6a5dac18936c675fd79c54af9570f7efbb01d85100a6f2970c956d2d15de7c7a617e54347296aca56c0b58a1154d8a20c77f5da0f2a28b473e2d7d3437593d1d787ee78884647dc976123b8e1c728227dd1fafd530c5422a758660d30785a5b19f4f35be342c8ade262968e31eb07fb05fea210d5e3baa46c251be7e8c75e42c788ae4e3d0a784e7f0161b2016c16cdee0\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4950153,
          "Plan Width": 1100,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 6528775.12
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 131.89
    }
  }
]
```

</details>

![Query Performance - ORE/range_lt_100](query_ore_range_lt_100_chart.png)

## range_lt_ordered_10

**Description:** Unknown query

****

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
| 10,000 | 10 | 477.00μs | 28.85ms |
| 100,000 | 10 | 508.19μs | 28.80ms |
| 1,000,000 | 10 | 513.33μs | 27.10ms |
| 10,000,000 | 10 | 519.51μs | 26.54ms |

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
      "Plan Width": 1128,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb280e44626ed9555695a2cb4526032945fc0d94a21cfafd74507cf7f504d833e4819e5d2559755ed1dd33bb71349ef14cdaa984cdf81c1f9b45f9760429854a86d97e7b52eae8ce7e76b219a1ceefa8f145e7851dea9d92bca84caed4fc577f0b3248e8ffd00c6619cdbe25fd2a49dd3a81b514a128eac4d6458804c8e0c22807f7fa4513ef1ad9e4e716dc2534ba73178d710434d7f5d907116b7fc5816dc094e3ee443a5f106fedcbe7ffe37144ef757ca1a0efacac1de17b6ac06e4117df56d4564636004948f8d680a8573dfef9203bd349d89894380eff94f63759eb0aa34f3c78e8638327bfc60a02da53257846f5028e714cac8febfeb0ca58d5eb1f5ef4a52a981c694138d1fe0ef1aecf582290\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_10000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4949,
          "Plan Width": 1128,
          "Relation Name": "integer_encrypted_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 8796.38
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 18.31
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
      "Plan Width": 1132,
      "Plans": [
        {
          "Alias": "integer_encrypted_100000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28a04e1538aef13d7dd0274a9106e45bb7e18744c1d268a8ae635ab442f5b76e9b9e10d695c7879972bf817d8a33a9fab2cf79b718dec6b47366f1105c61315524ccf91773e719f53f72a98fe07d2476e14c4752c0a2f105826c66db0a731df6a563ba7f884a4cd390f816b7262b80858ef95cf204af0b7fd73d221e3c665bd0e4fdc8dd741f97e9ff2781272426a781392d476744643d2388572fae7e50ca46b5d6d67b3e56d8ff13375add7d0696a818d5c802719e46c3ca40965aabdefb7e7e22d68cc57ca67e4e7707cc4ccbcd4e333d71bdfd77a6c02a7a18852cac31842590781ca501d08b6f524a18b0f70f60bfba0c3c895a617e603375c8dbc2d0a1611d2fccd785f508e78172c74d34bc204f\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_100000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50499,
          "Plan Width": 1132,
          "Relation Name": "integer_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.67,
          "Total Cost": 86629.25
        }
      ],
      "Startup Cost": 0.67,
      "Total Cost": 17.82
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
      "Plan Width": 1132,
      "Plans": [
        {
          "Alias": "integer_encrypted_1000000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28c42a974dd413be8197f39a39743b9753fb03ac7a0090988ceed596fc5f2eba8b3a4abd0bfc406e9574de6dfa828633cba8a23a99e5e94c4b769193fd0650114e95a993e451581068267bbb24ce7a639c28974d095366345a81fbe99d57aa9bd954e3e3ffc3c7ee1f18d58cad41b6b32fbec55da6048406616f338a39631e2e84690816e56858ac0d7e517c256131aeb42295fec3a55af4071c5f2dfa0e7ceed54cd4a61153f53806d6e2abaf9f5d36a2c80237b1d7e5baf94dbbb9ba979a269609e5f053f738bbcbc89fbca1a83dc642ef6a578ae5a6d419780fc64837551b1659f91170b8922aa92f8f51d970d45172736424fa44542400d359bfa102f701ad05b10414e6eca20bda1f2964ca7f1322\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_1000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 495002,
          "Plan Width": 1132,
          "Relation Name": "integer_encrypted_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.8,
          "Total Cost": 858388.62
        }
      ],
      "Startup Cost": 0.8,
      "Total Cost": 18.14
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
      "Plan Width": 1132,
      "Plans": [
        {
          "Alias": "integer_encrypted_10000000",
          "Async Capable": false,
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb280c255538e575a2753d81ac25925e6e969bb0185a5dc30563df8af424efb304dac7cabc859d0694466c688bf93c2a2442b293763a5ad34a2d6536df4985eb7f0db964c73be7605c18917aec9b6cc58c756c6bfa3c147164f40778499953e816ffd650afae1d4687b519c846d8ce849f2c40d787edb645b20459ad945b1aa86183198c4ab4ebe63c0c0bc89979cf088e025ab91d061f1dce453c8358034893ad2fbe6422dfc34d1b590abe1d6ac9aa223505cc8dd058eb4a4d672e53363c7cfda6d15aa600642077855f590e0df12e0f2ef1cf5c49e56a88affc372b8cfcf5f6a8b866763620d68333ffb15668e72fca5df89a3410446dcfc39a0194d9584518d484a816f719b684bea68e4f6aa80eaed5\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_10000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4950153,
          "Plan Width": 1132,
          "Relation Name": "integer_encrypted_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.94,
          "Total Cost": 8581229.33
        }
      ],
      "Startup Cost": 0.94,
      "Total Cost": 18.27
    }
  }
]
```

</details>

![Query Performance - ORE/range_lt_ordered_10](query_ore_range_lt_ordered_10_chart.png)

