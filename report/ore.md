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
| 10,000 | 10 | 1.09ms | 26.92ms |
| 100,000 | 10 | 1.98ms | 26.79ms |
| 1,000,000 | 10 | 1.76ms | 27.49ms |
| 10,000,000 | 10 | 1.42ms | 29.93ms |

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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bce6505e5d0db03c5dd653e9ad5bbbf37b01663a79c310597f2cca4feea88f7f863fdf7e0a642334bc851ef546007bab4c16748773788325e7693c2983f64e84420fefe66ebd55fba175369ff1cb035880cace69ada40eaa9c4e562258c34290667bbe9f9e089e37dee0a7fa68a1d6ecbb0e227b1711af6f0c662e7b9f665d2460a0460e1c7c84171e8115895dca66cfde0a3d5c59e35674c512b243ce1e9d6a2ec34d88a8d54d49cb3ed640b680b20d58dd07744ee54fcccb4e383b625ffacc120fe18629047851013df66e79cccf101ba839bb63c98bfa01121a41bf239c283e288df5d9730e3f463e905c464c9f7be623a47c2b4f9f948270cd132c67cbb3ef0e32320e83d411a6b4427aed3f657e7d\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28e2671b48a9d8f1fe2c256ecc704da27879e81717c0048018f01696a37c14708728c94c3620f79d265b167cfc507937e622e93cc7ec2755556b834bf39d39ff7142e5fe866333c1e9b52d93effebdc64b04b7f196aea01125337cfd78d442a76136fae950c4890b817e6ab0cfa4c4ed3cd5a96198f2b18772cf9484bc68fb8c92bf0b7d9a034e72d0f4bd9f9c7ffde1f849f77dae81a29b091e3d33258a6a83ba0eccc3a02748e84f2478871e4ebfe877f435efbbfe28d82ee6bdd8ad788ccb708fcbd2aa3d0887540a8b4fcb17c5e95ea70ed699f50f230a57a453e216661f14d3e04bbd2ec1ef136c16d082cf05c0215b486ad5b0de66af4eccb38ee58279f0afc65c3a3a00336f4f31d0470ea91342\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 51485,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 78157.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 15.18
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28afa19a7b131232f83db66d5b128966cf8f9cb18e7ae55e0224d59a0f74f1ecb5769fcc576297b696a78eac82310e7a47423ef8264a9193653649e45c728829b117de6e25ce0cc736efb808be3a37cfd121085fd2d9fde019ea95c2ac08bee411a3e1e3ebb26884963fee56bc6fc74d3b3cdac88a0d69899afd82a402eec163433fe67b338e86103a848d412071f1c8512ceb153e1e1640212e00d38b8279720bc6aea290dc02a7a4c73898f65da2f66a4ba8e15e6855eaee93990f5fe67002656c0b4aff86b85f32eee100230e1d9587c3254e10a7c1b38e040e8469965a0190aea860592cb719a5143fb8a4b550fd52cecd4a1402f5a02ce23adc40b8ed044dd62226fd35c00501b523a1e76a53150d\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 475250,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 771673.56
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 16.24
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
| 10,000 | 100 | 6.49ms | 42.04ms |
| 100,000 | 100 | 8.21ms | 43.75ms |
| 1,000,000 | 100 | 8.54ms | 47.74ms |
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bcf77977b33152775a0849e0fb9e90d1b5724a136449d77f85f9623e8a6ee61f10e0d8f485ea13f3e7522cfed28560e0a59a0c03c4b547b09f9da863d6889b7453b1c0c0e5b54222a6259b22ed2b98f752603ce6628b08da4e993c6165b3d7fdaf8ae72965268d0e524c2f0603d44d47e9c0fb410e16e2b1a4ee3cdd465a2c541aad8cc6273d11c66daf989bbd1b1536b816041cb55ab273f1882f078247272f15ccd5e7a1293dcfc299aa809bb595a54eb89afe5f91948671bf96d684828b23d22f6ce70c119dadf107c6882a2c9c7fc478dc285f3c3268ceb2fea1f52e991aae7182e4422c8e41c3e1b07844a705c7687118ea35ab9c896c944fdf4298ad5825005f80ecbaaad437e4bd489ff0d1441e\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb283743a5850c808795d0c54d2483bb2f6186a9c1d4f8b379942682dc12d9e70df90e01d8b73dcf31f313f9c3da1068cddf6de6eab5b7b8b3d4fb730f42bf12e30c1c4dea1c1590bcf1994e31df6ee1cfdd98e19e40df815125c3002dd3e296a019bb0eb12eb125c05fe8235768d936535d99cab06c9244a212e855f5960e17fe67ba5faf645acf5e8e0545394c7d686872800b256b09d664f2a4cdb6275b5ee61cc7e9612402671157022d9b5c2c32b629c2a5c5d26973b386381359f9b87292f6a22caff1dd0ad8fe9641841b330b8909a1b82eff2e4c556a370fad9d84c25c1435c64872ec30bd3593864f2c1dfe23fbf117e6bf05e707b278e3f349a14b6c7320191fa970ec6052d2f46e2efe254560\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 51485,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 78157.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 151.81
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb283b37cab0d68bb43dd50843749d396b5d4c7613e3be06e6c1cc3769636862de15ef236b2dc3e362f8f771656d47924ff21fc7bed7c0ba1becffafbf3a4c267597434b935c85300d0cdfb0636d5b98e7def86520137edbe337626933a69c85e0e5f18bd89bc95f40be020a5de0154dde4228f1d2addbf68f29439e58eacc7f5d2e6081bbc0acd77a8b7278c2444364025fc9176ca4cd233efddc5e2215af9c11182eabb831cb672b6b9c575fd991579a29519621789c424cdf8e6c82b340f7521403990c62d5e779bf885b9449708bbd62b9e7ce2f08dbccba54518df9019803565400930842de43cb9cb12bcf431d126343a7eb0d0183df6c8fd216cb8e045c9140f5ce92bb647dd43e7ce6800838598a\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 475250,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 771673.56
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 162.37
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
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `integer_encrypted_10000_ore_index`
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: `integer_encrypted_10000000_ore_index`

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 3 | 1.43ms | 29.65ms |
| 100,000 | 10 | ⚠️ 3.043s | ⚠️ 2.946s |
| 1,000,000 | 10 | ⚠️ 3.490s | ⚠️ 3.141s |
| 10,000,000 | 10 | ⚠️ 3.157s | ⚠️ 3.605s |

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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c74696450f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12000c1ce2da49187443be7a630b7bba4e4c788d889b7eef636659fb4d6b64dbab777c3973caa72a76ec3c372074623e3670b0657354ad4a7421ca41d370e370ff590a6b30fe2af4027fa71292f0e45ac799af17fb03cafe6be194889d475340b068b3fae2b2c12943b5c72e09c2e8299fc9549f3d90e06beb4b7e115ff92c5d982130fceb631a012c1322f858a5426a8765e9803d47a570d7d4b3bc9c311122d9c8dd1235314b4b423d38be0ca0203f990b9610edfe06e1f081c5a36c43316ef6394f9e76b4460031fcb6f0e20a2c90b0801c50a2663bf5f442d47d59266e3279dfaf34b249b4e60518b671af379d156f42f6311b475dbaae20f9db0a1db11a29231de62e70bf65889b9395cb4b00fc9056dfc254cea86d313be589ed5a46fe921ad46fb991ff6bc3b6784eef66189b75e6d0e00661387099c3b52495247cc159\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
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
  Gather
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
          "Node Type": "Gather",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10,
          "Plan Width": 36,
          "Plans": [
            {
              "Alias": "integer_encrypted_100000",
              "Async Capable": false,
              "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af391252a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78832bb9ab1c62d6173375d24578f37a3a1894af5fffa9bb07282f385b6a950f871d007ea7bab7ad6a555492b307b4323b6ac4abb3d91c22578cb4f9fd46f9473722e53e0330391e98dbf1b3d460c028e6eec77d801f5f2450615c0441d29bbe73ea7b4027e013814ed3ab830cdd1a9b017e5c88d0b527ad83dae629f121fa09bb1e527431a4d16cab224aee7722bf9bf597dc5c2a0f4efbe36b8df9e55726b92b7bfa8f096e5561f9fdfa3b262f309c88987d8507b8761e8fe6a155f423798122447b6c0de34014344006f55adc8368292beca2fa0a3297a2de156e0a2cdba8eb80966bfdc28041bcf08a366ac086ab832615302e1efca54191bec367769f0e828cad1b3511be3aa15907e7ed5441cf1901474ba8f605bcc6389ced850cc1b597c857b9bef829c99c00e19c977c82312f9db19fec76bfa3edb482ebbaaffcc73ff8\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
              "Node Type": "Seq Scan",
              "Parallel Aware": true,
              "Parent Relationship": "Outer",
              "Plan Rows": 4,
              "Plan Width": 36,
              "Relation Name": "integer_encrypted_100000",
              "Startup Cost": 0.0,
              "Total Cost": 35537.0
            }
          ],
          "Single Copy": false,
          "Startup Cost": 1000.0,
          "Total Cost": 36538.0,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 1000.0,
      "Total Cost": 36538.0
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Gather
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
          "Node Type": "Gather",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 100,
          "Plan Width": 36,
          "Plans": [
            {
              "Alias": "integer_encrypted_1000000",
              "Async Capable": false,
              "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af391252a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78832bb9ab1c62d6173375d24578f37a3a1894af5fffa9bb07282f385b6a950f871d007ea7bab7ad6a555492b307b4323b6a348c11c2ef3cbc0c8036e15656f648fdc713ffe98b21935f8caa88ef015db4536ccbea41ded9cc2b4af82868ef01559e4efdb111131647c9aab26899c0ec062a3da7ab0367615ac7ac8c50be0b49cfabf8abf674ff0f1d7be01958b8dc491936d5b8e6cd5399dd0c75955e2d769259b629b4074e123b90d0565093fff7594935d01af78cb6a20af55ba61452dc5cc4f07cc061acc243daefaa908c85e9e7015aeb8b083bad0b43c02d6b96434e7cad50d8f37546e888de3fdb39cacd1acc25af6342013ab6314878aa8d3ba6050fe17cefb387f1cd31f6dc27e398a187a68adf70e4d2c57fac65fef318ee6719e44a018169056f6245800b48c38355cc574e9e7a1b618e9f7c9cd0c85ab838f11ed0fe\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
              "Node Type": "Seq Scan",
              "Parallel Aware": true,
              "Parent Relationship": "Outer",
              "Plan Rows": 42,
              "Plan Width": 36,
              "Relation Name": "integer_encrypted_1000000",
              "Startup Cost": 0.0,
              "Total Cost": 355369.78
            }
          ],
          "Single Copy": false,
          "Startup Cost": 1000.0,
          "Total Cost": 356379.78,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 1000.0,
      "Total Cost": 36537.98
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
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `integer_encrypted_10000_ore_index`
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: `integer_encrypted_10000000_ore_index`

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 1 | 850.69μs | N/A |
| 100,000 | 1 | ⚠️ 828.83ms | N/A |
| 1,000,000 | 1 | ⚠️ 8.577s | N/A |
| 10,000,000 | 1 | 19.96ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Aggregate
  Bitmap Heap Scan on integer_encrypted_10000
    Bitmap Index Scan using integer_encrypted_10000_ore_index
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
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50,
          "Plan Width": 0,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c74696450f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12000c1ce2da49187443be7a630b7bba4e4c788d889b7eef636659fb4d6b64dbab777c3973caa72a76ec3c372074623e36ca01feb59b53a7fd3ec6c294403e0cae2f48e415c42c793a57bca9c9401973eb8ec69c36c2666090eec127850a13fa5739a8b2a4355604b79b3cfedb0230770970707cb62140e7e842ebfd292cfe957f0fa584e76cb7234fdaa826b1f97a98f10afbbc2cae030607a9450893cbd7b464ed9d61c8beeb67a0bc3e125d8911d9091e2b3525dd5ceae3cb99153363aebed8d9184296d198d25b68c82ba42bf1e193d97bbd2f3c58c359d48efd3aa1468f83b720cfba85742b14e70818e39a881303cf1e38f2a3b5c86b7380485e0fb0a79de67098e2c9aa8a7f3f587b86d4f87b2931f5a69d65ebd8d01fc3c65496cbc73d02888eb69744d8c06f28f1a6ffb93d854a252ba483dbf1d79cc7ba4a6829b3c4\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
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
          "Recheck Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c74696450f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12000c1ce2da49187443be7a630b7bba4e4c788d889b7eef636659fb4d6b64dbab777c3973caa72a76ec3c372074623e36ca01feb59b53a7fd3ec6c294403e0cae2f48e415c42c793a57bca9c9401973eb8ec69c36c2666090eec127850a13fa5739a8b2a4355604b79b3cfedb0230770970707cb62140e7e842ebfd292cfe957f0fa584e76cb7234fdaa826b1f97a98f10afbbc2cae030607a9450893cbd7b464ed9d61c8beeb67a0bc3e125d8911d9091e2b3525dd5ceae3cb99153363aebed8d9184296d198d25b68c82ba42bf1e193d97bbd2f3c58c359d48efd3aa1468f83b720cfba85742b14e70818e39a881303cf1e38f2a3b5c86b7380485e0fb0a79de67098e2c9aa8a7f3f587b86d4f87b2931f5a69d65ebd8d01fc3c65496cbc73d02888eb69744d8c06f28f1a6ffb93d854a252ba483dbf1d79cc7ba4a6829b3c4\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 16.92,
          "Total Cost": 214.36
        }
      ],
      "Startup Cost": 214.49,
      "Strategy": "Plain",
      "Total Cost": 214.5
    }
  }
]
```

**100,000 rows**

```
Aggregate
  Gather
    Aggregate
      Seq Scan on integer_encrypted_100000
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
                  "Alias": "integer_encrypted_100000",
                  "Async Capable": false,
                  "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af391252a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78832bb9ab1c62d6173375d24578f37a3a1894af5fffa9bb07282f385b6a950f871d007ea7bab7ad6a555492b307b4323b6a7480c162da74028088cb430a11926abd92f9a46a686812c71b80f50ef11a5cf69ddf11d63e13ba795016e2027468b25d7c2676f6537bbf2ee8438e5c1c4fea0ae42565ff91989145c9bb58091d86eb0730b1cd7ddce251e73a2263e326b28a4ce2fdc614a2f69ba07801a301af6d5db23a38f27bbeca487c1a03dfb751affe6ca0ac7be1553086984f17ae7450c710dc778d462632585b6c91dbf5d096a03acb5dcb5ba27d8f891dedf1544a9cfa0700eede3f055547142ba7f82f6e1f98450220500a26048a676ab6247e996f9f99b23202f8840bf8871ff8e1395dec96e53fe9197c9d3c2edbad34de460d5ef43d113bc138b04f93e9412689a1a9ae07be9f3e1b2351cefaf310b54558397674189c\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                  "Node Type": "Seq Scan",
                  "Parallel Aware": true,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 4,
                  "Plan Width": 0,
                  "Relation Name": "integer_encrypted_100000",
                  "Startup Cost": 0.0,
                  "Total Cost": 35536.0
                }
              ],
              "Startup Cost": 35536.01,
              "Strategy": "Plain",
              "Total Cost": 35536.02
            }
          ],
          "Single Copy": false,
          "Startup Cost": 36536.01,
          "Total Cost": 36536.22,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 36536.22,
      "Strategy": "Plain",
      "Total Cost": 36536.24
    }
  }
]
```

**1,000,000 rows**

```
Aggregate
  Gather
    Aggregate
      Seq Scan on integer_encrypted_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "JIT": {
      "Functions": 6,
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
                  "Alias": "integer_encrypted_1000000",
                  "Async Capable": false,
                  "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af391252a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78832bb9ab1c62d6173375d24578f37a3a1894af5fffa9bb07282f385b6a950f871d007ea7bab7ad6a555492b307b4323b6a7798ff4821f16b57ea21aa76ce2481db975f5ac8e4cfd42f20348eb83a44d6e8e03ee1f5ad91ce27b7083446244fb38dd6ecf846ff9630d425e12db5a2e83ad9d28c59d131bbb9f89e90ebd6e145cb843e58d66667dab6d5ff237085aaf58146a1bd3bce7fe0d02a4c7e27c35897e04c4734e0c2d7187d982fb5deb8705a892257c4e1878f9d6d5ea7fce29c1bf79e24f7b304d2d5d2876f9b8b9b927925039ba773f308dc8a2f377b6d10ce745c5c94537ea6e06e61229e9f10e7f07e2d8f16d76bccb310123b21aa19b00c3bc2ebc3cbd6e4ef2dbbbf592f8508aab521c989453a63fc58d1ee6512ab09d6f938e7ed8c89100b751417ab4a8eea2d271013dc33cb17233a01c92f032e99ccb8ebe290\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                  "Node Type": "Seq Scan",
                  "Parallel Aware": true,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 42,
                  "Plan Width": 0,
                  "Relation Name": "integer_encrypted_1000000",
                  "Startup Cost": 0.0,
                  "Total Cost": 355359.28
                }
              ],
              "Startup Cost": 355359.38,
              "Strategy": "Plain",
              "Total Cost": 355359.39
            }
          ],
          "Single Copy": false,
          "Startup Cost": 356359.38,
          "Total Cost": 356359.59,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 356359.6,
      "Strategy": "Plain",
      "Total Cost": 356359.61
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
| 10,000 | 10 | 1.64ms | 28.29ms |
| 100,000 | 10 | 2.56ms | 27.50ms |
| 1,000,000 | 10 | 3.36ms | 25.65ms |
| 10,000,000 | 10 | 1.72ms | 30.15ms |

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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bceda16bf25f19b2ba0c94d71aae8910336edbe8f825a9f1be66459c95f7d5ce637ffd968733af28c55772302718b1537642462eae63269ca3e1b1806a4285ea492b8b37638b391569cf41a9958e7c039252ea8645bcdb8b1104998e7eec88bb77ca2eca6e5c67125bff1dbb77ba8545eec1a301162fef42843b650e8f058ba0c1a91e9cb871057791d996b8de16fcb07d261a98f909f2be733592615197af1670d7d9d0691d9615721cd2a76f48db165128a311ba268e1cf2c26039f0b2c48bcaca64d340d0dbd03b7cbe0c44cdf32e9deb4fc11ceea9c799693678136e8a8934a696410c4dd99dc339af41270c2b5fa709e60ba4768d1440d612f7895df7724840a5c6f493f382091b10f8aadc17d266\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28d88b01741e85cb372190f087f70494921a644229c8a4db07cc3d112dc7cd3f73190d779efdf614086f82924e8792b225d905791f8b23f3a9d5793dd37ee234c191b1fd93879d6de84c0e08f19e2a0853510e6d40b3dbff6b0e595c74467945e3bad8adb657a483a7193c1aeb55b1dec52d672b829b97afb16aa64f002dbf852ce879be45ca48c7c00f744a20bf91e726f5e4182bfef8c3c279a891425372a465bb482cd2472c1ffae03e6324f655aa87a4d7d8eb355dab8342b757a73523ff59bf6f1ba1dfb462e7d5106509f09c12119f600918f5068fb6b55363b59658684f44274207476ad5dc69b10675aef7d6d9aa293123ce51c952547238c0982d43cceac6e758cbaa3df1d80fc85e67b4fadb\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 48515,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77414.75
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 15.96
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28cb13b7c13eb6527ebb255a7701d9dede2bac556fffdab6d334ac87915b1ca7efd17bef735722161fbe5d1c0de24f922afa1ab349feb6f6a0f0ab0397341b91fd496b9e26b6a672e219246e92595fcbe3c0832e9fa7983f510c827e9c526ea70cd246c65b2e566d1811b78bcbe8389f31459600eb0c3c19c68d2a91e041b0475aa2839e977787b932fbd84bc435db38f0d9ce063d06b76e0e92aa1e5fdcf2754158be34962368fbcbb605c863ae123e70e662f5dba3cf3f5116d049912e0b141c6113baff20dce0e306f0cb9b4fbc4aa7a17c793ce5f9cd35a2da9b225bb95c8d88ef75de14772b285062f3aaa98f7c3853cebe18dcf9bf45c57daed21d45554d690af153451ebb88fcf92d285de924c9\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 524756,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 784050.06
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 14.94
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
| 10,000 | 100 | 7.13ms | 46.12ms |
| 100,000 | 100 | 7.54ms | 43.38ms |
| 1,000,000 | 100 | 7.73ms | 47.67ms |
| 10,000,000 | 100 | 7.58ms | 45.56ms |

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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bc981f0239ae333c99d66e5d01c963193aaa470df62b2cf1d8fadc9dacfefbf678db1d341af31e204af6bedb5687884f5926aecccff21542589f17ccd5d92f58039d7cb9bc26558dcbe00ad147970e8efd3c8dfe3c269ecd91a9eebd4169d2509d9c9593ce79991fa754adee5102cb0f6bd53e197d022d06f31e56e1a7c964b87fd1e2de5c0e3a5c7bc2bf8dde1b96b79bde395c2bae3528d2b8f03bbe012db7858b2fcdfd76140c593eefbfbcb6f096f7e267094ebcbfcf2b97e1d63355f7402900e78841ae4d014f529dc4ac6726ceefd7f53b331dac99225d26122bc018fb3ef32ef60e18632be85cce7c73801c2094c8eb1f0c966755c54c626568cf3c01bd6f9968d16011a17308379262242d3faf\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb288640da67c52edc4a4c4e99fed5324b0a5c3bc8e13c9522375ea8780b15e098b530e34943f7758f837ff38b76a454a3ab086cf44e7cab968981a7adca9aff1a455eb4ab762929b2353dd3a72ea4425e020a3cf5dfd7639b4c22fce81de2970875534861a789175bc515018232b8224e7e6ad495d893955b19f159061149d91d8a99039391c950bb9d1b851992d03162589dc7e6835009ec076f33a04cb0b61b0c76f3cf2928b3cd09a1fc71b99857a0df46e13e3455dd64e18fe5d86db4b5e8787ac433cb79977faf056dfa1e29879cbeaf9dcf6baa318c50ca02cf2aba47346aaf395c3a172c2a2f273de26863b84103c9bcb9e3abf3d8181781c68c326d5ab257f83011ad8fd15a783f557a81ce327a\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 48515,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 77414.75
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 159.57
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb289c142dd6e77da790833f93c082c060e7f0e5692d69b6c195df9bd7d062cae95e7d43996d801281b5fd333dee6d68b9ca336fb14c5f9b9f66410f4c593484da1f9edb02196666b76ba59c8b8aa0e9ad9e5c27257045fd1a3e70c4676feb8231756c06815a26c8770882be3091bd0785ea45cb4b8f2d420b9107d77bdefad99fa6b1d95b284c0eea8a4ac9e1dbfebb8c4c4c2256b7a55db13bf025e5d9265d34f53c291bc2af76d69a91ee2161a0bbf56a885b00559ce4dcaaf8569b33d3f6315e04c46472cb9d53721136c898fd6f7f383a1f05785943eef2168336935a3783bbd54b10d0d3012dc8705ecb050c41d9e4a702a02809d2895417f64c9abead9166d639f90503e80227d6cb50d1042c7230\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 524756,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 784050.06
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 149.41
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
| 10,000 | 10 | 1.96ms | 30.28ms |
| 100,000 | 10 | 2.21ms | 27.24ms |
| 1,000,000 | 10 | 2.48ms | 28.54ms |
| 10,000,000 | 10 | 1.16ms | 29.07ms |

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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181813d3ba05650f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f48144b2318d70dbf219e5ccc1d1871c3b4249148392b5dab99ef56f446d7e2b2ee6e9f2875635d6b2b1b5c167a713e05cee6d388100bab0f678d763ff3cbe16bc14347b7005145baed64bf5fd2b8c091f589680a0075cc7b0ffc370b8403d88e39ef58e2e11513beaeb537b61e17653181c5098ad15b1b4938d2295de2d7182221083e77a080cf8d5811fcbbbb8915865d1c1ca4f1ee713737a75b21e2874b003d9a5e9ec064049a77b210642d44390b830b7ed1001be4cdeed88ba168bc01d5714ddac2c8ba033bd00da6aa4eea70669b3ca6214f798d3abca3debbbdc0aac1bd8110a55015cd6dc522f8e1fe161cb6de2b1f60456d5041488cd50dc15f9ddec1076b2202a0291c2a34a28459ffc061ef95890d8829788296cb6a4c54c8ed3e6c886fce7a37ed38c84c42f83f79eb0581b6592858e80879d01d855bcb646e76612af9987ffdf73d0d65127b8409d5388\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb2808a633a1c6ca153839ac231a8b07016c10ea1796c7241ec4d8a1d8d14ef2223d66424b6e3393e46d5b8d876a9863e94f256a73f286c4748996d2ecb0bda163d21295342a2a892f53f98bc48b3817c1d71c1aabeea11293343f764400f550ba7a5825eae4d03d06e63e3d40e426f06aba985249a352ea0d5200c637074028fa101ed961122c442cd7381c8af1614644c178599e30794cae679ea83697aef06c8d42ea04fdd133a14b122fdc940d6da5115db29999f1794dc3a6578d52e1aaffcf1ae48b346ad562a2d31de8bd1f12d0fde88c4a92eeb8cfd937c5f990bf2eef60ed72957c3ac80e0997daef8cf77b0aa68bbef1accb9c4df0e579ce5ddf793bd02044a2efd5a0d092e4ef562812c59813\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_100000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 48515,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.67,
          "Total Cost": 164086.09
        }
      ],
      "Startup Cost": 0.67,
      "Total Cost": 34.49
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28d6362ddfc670a383f6b7461d82c61bb5791757eecf7679c123f9cda394ca5708fa745b4a799d705fc2f3ac679cf7b9b0a8bca6dbfb38961a2280c34c2a7ca8a00f8af1ec371663583a49cbc35ef2890400fd6b5b7819fc6d397cc71655917bb4c827adfbef26f2395e2dd20dd0f7b111c88fc2e005e0b5edf0d6b01c20ad529f461c70bd267eaaad8e3105c5e7ee7ce49cec9ec92d42b12528b1de766c3f39bbc84fb5b83afaaaf0653e41b728c8331dfb82b3e0054b9eb57fd35edcaca7e7fb160ccf2f33467f710c5fd242b87d3e0953b7cd22f2d72c22dba31778d58836e98ea8573f013dfa8ee81c35a320f7580a8241f708529d51efd306890c6ea61e5c6f6f9128d69a9c9c94cfcd34b79b69e3\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_1000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 524756,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.8,
          "Total Cost": 1660514.06
        }
      ],
      "Startup Cost": 0.8,
      "Total Cost": 32.44
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
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `integer_encrypted_10000_ore_index`
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: `integer_encrypted_10000000_ore_index`

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 21 | 1.21ms | 32.00ms |
| 100,000 | 100 | ⚠️ 833.04ms | ⚠️ 994.06ms |
| 1,000,000 | 100 | ⚠️ 1.199s | ⚠️ 1.681s |
| 10,000,000 | 100 | ⚠️ 2.092s | ⚠️ 2.262s |

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
              "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c54c4cd50f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12bbb0bd496b07fcd3fb5f6023b99f693a88b0a0e8c51a1c10c2af20a0e7cfc0a7e40c25d147a0ac4821b8eae3d8d762b91dae460933df85e4969c1af944c2e28978bc4808c2e4f10131b732b607dac7eb48f29e66b0599ad7182328009dd2b61f89ba48fd10657e90a6c1f16fd41e97fbaf99a11818257f963c1bc6eecde30663acdc615d65a0653b21704d00ff0a3173de2d878f31fcfdc9c34c77ebc71b41f1e4f0dd617395e5797ae691fc2e5864765f567790a9db72d781467b7f6e1b83cef0fb5a34b54983d0a195334289bf80c6f03ea0c7a1011e4c0177758c82b54eb4b5cd429895f5f79bba741349e330d04b4c5fea589b5765cc5a8584e55c86728e0cc19ef5c808bec946522e12d6c97d690da317f5f493c51d0eb8ee79a859e2864d057a50c5cfc93cc0807e18b9aaa251e349afc80ff526bcbf63b9dc41ffc567\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
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
          "Recheck Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c54c4cd50f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12bbb0bd496b07fcd3fb5f6023b99f693a88b0a0e8c51a1c10c2af20a0e7cfc0a7e40c25d147a0ac4821b8eae3d8d762b91dae460933df85e4969c1af944c2e28978bc4808c2e4f10131b732b607dac7eb48f29e66b0599ad7182328009dd2b61f89ba48fd10657e90a6c1f16fd41e97fbaf99a11818257f963c1bc6eecde30663acdc615d65a0653b21704d00ff0a3173de2d878f31fcfdc9c34c77ebc71b41f1e4f0dd617395e5797ae691fc2e5864765f567790a9db72d781467b7f6e1b83cef0fb5a34b54983d0a195334289bf80c6f03ea0c7a1011e4c0177758c82b54eb4b5cd429895f5f79bba741349e330d04b4c5fea589b5765cc5a8584e55c86728e0cc19ef5c808bec946522e12d6c97d690da317f5f493c51d0eb8ee79a859e2864d057a50c5cfc93cc0807e18b9aaa251e349afc80ff526bcbf63b9dc41ffc567\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
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
  Gather
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
          "Async Capable": false,
          "Node Type": "Gather",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 990,
          "Plan Width": 36,
          "Plans": [
            {
              "Alias": "integer_encrypted_100000",
              "Async Capable": false,
              "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af3965ab3800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78839e5375b93148d2c7258c7c5b10da6baaf6161425134bf7a220b78fc411dc6f7124f9da3212acff12515e2eed9100ed1b843eb3b32c1dadf98c34f506685300f960712dd336e881a644c1b8b4c8893aa146359f82906a2a38a10d7c30aaa265448be04b6c650c045f59e4fe59e20004b38d862333b2fdc0e5088a3041f1a6f971d5bbd5f9e5db2f1370fdbc5109d5e3cfd5bcae1c842f276e1f803e4c6abf305ad987222c99bb899f54025cdfec37ca8e07bd69c180abf0dd032b4f99a4c2b64bf7f2bff5acf3e598f1db0aca0ac6133e000e4abbab4161e8d9c21b8041758574ebdc3b45bc6a856e3e68fdc03a27dfcdb45f8844fdad2a59bc5431a9aca0035ae09e65800082fab6ee640557cd52b64a2acab41556ba7a42dbefa176fa8bddbffffa5a856b9126076f88c5ce19ed2bc28d1f9bc3d3a3d560d068f02040d4cbc1\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
              "Node Type": "Seq Scan",
              "Parallel Aware": true,
              "Parent Relationship": "Outer",
              "Plan Rows": 412,
              "Plan Width": 36,
              "Relation Name": "integer_encrypted_100000",
              "Startup Cost": 0.0,
              "Total Cost": 35639.0
            }
          ],
          "Single Copy": false,
          "Startup Cost": 1000.0,
          "Total Cost": 36738.0,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 1000.0,
      "Total Cost": 4609.9
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Gather
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
          "Async Capable": false,
          "Node Type": "Gather",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 9901,
          "Plan Width": 36,
          "Plans": [
            {
              "Alias": "integer_encrypted_1000000",
              "Async Capable": false,
              "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af3965ab3800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78839e5375b93148d2c7258c7c5b10da6baaf6161425134bf7a220b78fc411dc6f7124f9da3212acff12515e2eed9100ed1bf89f0399a0572fb28ab4ae5513437976f9e4bfcde5a041c86602f37fdb8649e663a80f9496d5db24270960f6e29fad213e202d481dfa01fbaff07f68ab093d688250ecbacd511687a571ecf5017f6ce083f1347ae91fc1d8b427fa63ee10746c6217f3f54ba1658c9223e4d28eb8edb3c0e308ea58bc1b20a432903c58d66324bc105f72e7531eef97bdeb8c584a13769251314f13956856696e632b09e6696d4a3e5a462c9ec80ebcfe68ee4b635559fdf74c5008118e97ac56eb481f479ac3f9324031a4fdfa40296a84815eee1dee98bc68a3cee489f8304bf9fa72d48bd8ead75b0076a1a572319573de12ca1746185edaaf08b4575bf0e2b4a42039ff51b5fc8b6629ca2b8a64da422b146c9581\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
              "Node Type": "Seq Scan",
              "Parallel Aware": true,
              "Parent Relationship": "Outer",
              "Plan Rows": 4125,
              "Plan Width": 36,
              "Relation Name": "integer_encrypted_1000000",
              "Startup Cost": 0.0,
              "Total Cost": 356390.53
            }
          ],
          "Single Copy": false,
          "Startup Cost": 1000.0,
          "Total Cost": 358380.62,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 1000.0,
      "Total Cost": 4609.54
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
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `integer_encrypted_10000_ore_index`
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: `integer_encrypted_10000000_ore_index`

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 1 | 958.35μs | N/A |
| 100,000 | 1 | ⚠️ 834.39ms | N/A |
| 1,000,000 | 1 | ⚠️ 8.744s | N/A |
| 10,000,000 | 1 | 77.18ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Aggregate
  Bitmap Heap Scan on integer_encrypted_10000
    Bitmap Index Scan using integer_encrypted_10000_ore_index
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
          "Alias": "integer_encrypted_10000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50,
          "Plan Width": 0,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c54c4cd50f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12bbb0bd496b07fcd3fb5f6023b99f693a88b0a0e8c51a1c10c2af20a0e7cfc0a7e40c25d147a0ac4821b8eae3d8d762b91a80367663852bdfcbc395253c06c9c19d4bb4234c922426de81be8473af8ab59a928fbff4b7f6f70d275c6b00206fb739b00718ed29aeebbbad1fa73bd2ed0789d978867c805e75a42d2517e204e9a807d88bd71a40c8c975cf9bafef627ffefdd245ebb16b7959f956a5916e058dc6f757866458c67afb9f17f60aa1fa4366d91f9ed39cc0d642c80329c3d762ed4a70d7818ac373604b2a15f1efcdb7af4605006becd7e953169cf2c0a1ef5c970875ce3c033f1d15e9ae7c04ef0e0674774397cddbaedbb588d1ad491ea5e1d74bb13f55290e2e9ea95cce66ee2f7e36963557a39358de046dd56a7d263254b0abde73b81fcab424245c1a5aaaf4c6d29ea997cecf2ccc24eeb8fa1181ee17c050\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
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
          "Recheck Cond": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x818181819c54c4cd50f0e2e50041a4028946d3b901f2227b9f40ca2d24ae4855610af3b93ab3fb03fb06a17df7471b16a573428a9f4c92b3cc79a164992485a18ab87494b45830f40b0aaa51599711df6ba2dcc070bb9c12bbb0bd496b07fcd3fb5f6023b99f693a88b0a0e8c51a1c10c2af20a0e7cfc0a7e40c25d147a0ac4821b8eae3d8d762b91a80367663852bdfcbc395253c06c9c19d4bb4234c922426de81be8473af8ab59a928fbff4b7f6f70d275c6b00206fb739b00718ed29aeebbbad1fa73bd2ed0789d978867c805e75a42d2517e204e9a807d88bd71a40c8c975cf9bafef627ffefdd245ebb16b7959f956a5916e058dc6f757866458c67afb9f17f60aa1fa4366d91f9ed39cc0d642c80329c3d762ed4a70d7818ac373604b2a15f1efcdb7af4605006becd7e953169cf2c0a1ef5c970875ce3c033f1d15e9ae7c04ef0e0674774397cddbaedbb588d1ad491ea5e1d74bb13f55290e2e9ea95cce66ee2f7e36963557a39358de046dd56a7d263254b0abde73b81fcab424245c1a5aaaf4c6d29ea997cecf2ccc24eeb8fa1181ee17c050\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 16.92,
          "Total Cost": 214.36
        }
      ],
      "Startup Cost": 214.49,
      "Strategy": "Plain",
      "Total Cost": 214.5
    }
  }
]
```

**100,000 rows**

```
Aggregate
  Gather
    Aggregate
      Seq Scan on integer_encrypted_100000
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
                  "Alias": "integer_encrypted_100000",
                  "Async Capable": false,
                  "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af3965ab3800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78839e5375b93148d2c7258c7c5b10da6baaf6161425134bf7a220b78fc411dc6f7124f9da3212acff12515e2eed9100ed1bbae6689e26a003980e4687449d2e03f1052526231528d9a073f135675f81c4e19afcc1663b0423ad0549b51d758d66bb057d50d0729cf956b2d19748d721bf75ae2c2605169544f9282fbfea064bd56a0456aaa72707900a1b5d8477ebb3c0baed8e68e963c3cddb5ef5f038c3ed96c7a0d4961be7bb28e286dba8c94293896ed96895d55f6be01c33d7b3bced3676343bb359fa2ca4b73d7ebb7aad50ca0f0bfa7d78d8bf83e01711d6186e61b781f1bc69714b830a6edcf9e10e90b8daef4e5c56486c2acc3edf1b166ac4718eb82023ae1d6b456418719fdd062ce609d3ce4773ddb466239f32b3ba3a008dfd7a3ada7c1d9aec920f5ff70019e77e932a5a6c619a16393adde58620cf2a04390cb0\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                  "Node Type": "Seq Scan",
                  "Parallel Aware": true,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 412,
                  "Plan Width": 0,
                  "Relation Name": "integer_encrypted_100000",
                  "Startup Cost": 0.0,
                  "Total Cost": 35536.0
                }
              ],
              "Startup Cost": 35537.03,
              "Strategy": "Plain",
              "Total Cost": 35537.04
            }
          ],
          "Single Copy": false,
          "Startup Cost": 36537.03,
          "Total Cost": 36537.24,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 36537.24,
      "Strategy": "Plain",
      "Total Cost": 36537.25
    }
  }
]
```

**1,000,000 rows**

```
Aggregate
  Gather
    Aggregate
      Seq Scan on integer_encrypted_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "JIT": {
      "Functions": 6,
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
                  "Alias": "integer_encrypted_1000000",
                  "Async Capable": false,
                  "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4af3965ab3800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7009e9d384e0a1948ba24bd23106f78839e5375b93148d2c7258c7c5b10da6baaf6161425134bf7a220b78fc411dc6f7124f9da3212acff12515e2eed9100ed1bbfea15acf117b76fbfebfd35ffb619ea6e1e7b69520e2ec3ce12eec6343751fb9c837856ff7dae84b210f6d5c729bd9cbe736313defdebe0905dca87d72a0d0d4951e688229330aae942f68fc85c5cca812190bf11215e2ddb3a749aa1595e4c637367897119fbdc749109a71492bb49e305796c580e0f49cbaaba95386880f866f447b0af234a99497a12daafca50722bb2c284feb7ecea2c18d401314eb01364eada580b2a439848e179124f2d663cfc41906424d1f03b04de64966a1bb6eaf520f24821df7f494572d1b17dc3fcd7acb6c2bc7229bf0e460928f2f0c447ea1e74b1ebec4bf0ec7d2e6c36ca3f9f7d18c5780afdf101b364534d3e3892f08d911085bc04d95dfad81263d236153c1a\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
                  "Node Type": "Seq Scan",
                  "Parallel Aware": true,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 4125,
                  "Plan Width": 0,
                  "Relation Name": "integer_encrypted_1000000",
                  "Startup Cost": 0.0,
                  "Total Cost": 355359.28
                }
              ],
              "Startup Cost": 355369.59,
              "Strategy": "Plain",
              "Total Cost": 355369.6
            }
          ],
          "Single Copy": false,
          "Startup Cost": 356369.59,
          "Total Cost": 356369.8,
          "Workers Planned": 2
        }
      ],
      "Startup Cost": 356369.8,
      "Strategy": "Plain",
      "Total Cost": 356369.81
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

