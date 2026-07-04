# COMBO Queries

[← Back to overview](./BENCHMARK_REPORT.md)

Per-tier query performance. Each scenario lists its SQL, the indexes available on the target table, the indexes the planner actually picked per tier, the timing table, and the full EXPLAIN plan in a collapsed block.

## bloom_ore_order_limit

**Description:** Composite predicate: filter by name pattern (bloom), order by age (ORE), limit 10

**SQL Query:**
```sql
SELECT id FROM {TABLE} WHERE name LIKE $1 ORDER BY eql_v2.ore_block_u64_8_256(age) LIMIT 10
```

**Parameter:** `Bob`

**Table: `combo_encrypted_{rows}` with three encrypted columns — `name` (match + hmac), `age` (ORE), `category` (hmac). Indexes: functional GIN on `eql_v2.bloom_filter(name)`, functional btree on `eql_v2.ore_block_u64_8_256(age)`, functional hash on `eql_v2.hmac_256(category)`. **The bloom GIN index engages for the LIKE predicate**, narrowing the input to ~0.01–0.1% of rows; the planner then sorts the small filtered set by `eql_v2.ore_block_u64_8_256(age)` and returns the top 10. The ORE btree doesn't engage here — PostgreSQL can't merge two unrelated indexes on different columns (bloom on `name`, btree on `age`), so the ORDER BY is satisfied by a Sort node above the Bitmap Heap Scan. With the bloom narrowing so aggressively, that Sort is cheap; the cost is dominated by the bloom + heap fetch.**

**Indexes available on the table:**
```sql
CREATE INDEX
combo_encrypted_10000_name_gin_index
ON combo_encrypted_10000 USING GIN (
    eql_v2.bloom_filter(name)
);

CREATE INDEX
combo_encrypted_10000_age_ore_index
ON combo_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(age)
);

CREATE INDEX
combo_encrypted_10000_category_hash_index
ON combo_encrypted_10000 USING hash (
    eql_v2.hmac_256(category)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `combo_encrypted_v3_10000_name_match_gin_index`
- 100,000: `combo_encrypted_v3_100000_name_match_gin_index`
- 1,000,000: `combo_encrypted_v3_1000000_name_match_gin_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 4 | 451.00μs | N/A |
| 100,000 | 10 | 2.13ms | N/A |
| 1,000,000 | 10 | 14.14ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Sort
    Bitmap Heap Scan on combo_encrypted_v3_10000
      Bitmap Index Scan using combo_encrypted_v3_10000_name_match_gin_index
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
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Alias": "combo_encrypted_v3_10000",
              "Async Capable": false,
              "Node Type": "Bitmap Heap Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 36,
              "Plans": [
                {
                  "Async Capable": false,
                  "Index Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbL>skz&>@ZwHR>sj!<V%}236(jdb_+=HrU3gQjEgg#q?Xpclr(wh(VBF#a?MDyDPvRvd2q<=k5C2T@QHSlG`cgyxz?AYzq;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [567, 943, 1673, 1076, 1471, 1346], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c32944ef4c23761fbfebce84279b510ec9d61424533b13324333fe203e628548d7c32d933211b7d1ccbe80b6f159cf35bcb72e227ed7cf9d6d98d34d0caa63532faf2cb1bc7fab22e2fa62153c47ef16a2059a84f0365683dcc9e988b6d8cc0c9af7b898dd0cf94780449205893c437ca52736531299dbe8da84a6ac71583973131df8464b7cc2b20c49dccadd71f326d23b040b2ead89b04136219a72c3393eb8aa67dd4c92aef974ea8bf765423fbd2bcf843b8da7c0644d9ca91730071c5724b9c98c9c0d1e820878318186beaf8bbc6e5d9d4f089231f0c1a8bf9ddfc55cd07d23d62fb489b9dd83ba9624f103bf313a53c868c4311faaabc7318014f8a146db15ad3e5b9eda879a516824fc1b3804\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
                  "Index Name": "combo_encrypted_v3_10000_name_match_gin_index",
                  "Node Type": "Bitmap Index Scan",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 1,
                  "Plan Width": 0,
                  "Startup Cost": 0.0,
                  "Total Cost": 56.47
                }
              ],
              "Recheck Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbL>skz&>@ZwHR>sj!<V%}236(jdb_+=HrU3gQjEgg#q?Xpclr(wh(VBF#a?MDyDPvRvd2q<=k5C2T@QHSlG`cgyxz?AYzq;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [567, 943, 1673, 1076, 1471, 1346], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c32944ef4c23761fbfebce84279b510ec9d61424533b13324333fe203e628548d7c32d933211b7d1ccbe80b6f159cf35bcb72e227ed7cf9d6d98d34d0caa63532faf2cb1bc7fab22e2fa62153c47ef16a2059a84f0365683dcc9e988b6d8cc0c9af7b898dd0cf94780449205893c437ca52736531299dbe8da84a6ac71583973131df8464b7cc2b20c49dccadd71f326d23b040b2ead89b04136219a72c3393eb8aa67dd4c92aef974ea8bf765423fbd2bcf843b8da7c0644d9ca91730071c5724b9c98c9c0d1e820878318186beaf8bbc6e5d9d4f089231f0c1a8bf9ddfc55cd07d23d62fb489b9dd83ba9624f103bf313a53c868c4311faaabc7318014f8a146db15ad3e5b9eda879a516824fc1b3804\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Relation Name": "combo_encrypted_v3_10000",
              "Startup Cost": 56.47,
              "Total Cost": 61.24
            }
          ],
          "Sort Key": [
            "(eql_v3_internal.ore_block_256((age)::jsonb))"
          ],
          "Startup Cost": 61.25,
          "Total Cost": 61.25
        }
      ],
      "Startup Cost": 61.25,
      "Total Cost": 61.25
    }
  }
]
```

**100,000 rows**

```
Limit
  Sort
    Bitmap Heap Scan on combo_encrypted_v3_100000
      Bitmap Index Scan using combo_encrypted_v3_100000_name_match_gin_index
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
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Alias": "combo_encrypted_v3_100000",
              "Async Capable": false,
              "Node Type": "Bitmap Heap Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 36,
              "Plans": [
                {
                  "Async Capable": false,
                  "Index Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbJtpEKZ-1_Rlb9xuzbSS7Z^6}IlL&FPAi!hQM-qlov%c*O9uu5-j7QD)aGa`MU~ga<Vb8it2(52)d0lfDpsj|T$OpW(huq;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [1076, 1673, 943, 1471, 1346, 567], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c38b2943f88091c46111f2f44d5d6e3caabac269f34844f3f474c4816a17450355e7ff3d2ccfa4d9b48f15fa192bae9f39505ad26db4ad2a6b6bfac71a7de7de97aba2acdb8060f7d5852dcf77141273f733cf3f5b20b52de11fa0c3d557aca03d8f23e393b74c273835190dbc125783bbb6de70f8807a44340892760f6d9abaeaa4c5e7858ce9f8b4ace4258d755c08e1388180228c4fb2a346a023777f6851e73cbf549add3f450c0acda40a406dffe9733fdf14d88cb967ed332174db2fe03a1ae6bda43fb8095dba07cc735af572156f049ec155703cfd7d0fcf29bf039b7fb304b67978068b4cb7b41d8412b42960d5b2bbd8e333dc50823b90903ac7f53cd97128ab7d6c49582a4f8ae89ff6aab1\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
                  "Index Name": "combo_encrypted_v3_100000_name_match_gin_index",
                  "Node Type": "Bitmap Index Scan",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 1,
                  "Plan Width": 0,
                  "Startup Cost": 0.0,
                  "Total Cost": 93.6
                }
              ],
              "Recheck Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbJtpEKZ-1_Rlb9xuzbSS7Z^6}IlL&FPAi!hQM-qlov%c*O9uu5-j7QD)aGa`MU~ga<Vb8it2(52)d0lfDpsj|T$OpW(huq;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [1076, 1673, 943, 1471, 1346, 567], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c38b2943f88091c46111f2f44d5d6e3caabac269f34844f3f474c4816a17450355e7ff3d2ccfa4d9b48f15fa192bae9f39505ad26db4ad2a6b6bfac71a7de7de97aba2acdb8060f7d5852dcf77141273f733cf3f5b20b52de11fa0c3d557aca03d8f23e393b74c273835190dbc125783bbb6de70f8807a44340892760f6d9abaeaa4c5e7858ce9f8b4ace4258d755c08e1388180228c4fb2a346a023777f6851e73cbf549add3f450c0acda40a406dffe9733fdf14d88cb967ed332174db2fe03a1ae6bda43fb8095dba07cc735af572156f049ec155703cfd7d0fcf29bf039b7fb304b67978068b4cb7b41d8412b42960d5b2bbd8e333dc50823b90903ac7f53cd97128ab7d6c49582a4f8ae89ff6aab1\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Relation Name": "combo_encrypted_v3_100000",
              "Startup Cost": 93.6,
              "Total Cost": 98.36
            }
          ],
          "Sort Key": [
            "(eql_v3_internal.ore_block_256((age)::jsonb))"
          ],
          "Startup Cost": 98.37,
          "Total Cost": 98.38
        }
      ],
      "Startup Cost": 98.37,
      "Total Cost": 98.38
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Sort
    Bitmap Heap Scan on combo_encrypted_v3_1000000
      Bitmap Index Scan using combo_encrypted_v3_1000000_name_match_gin_index
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
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Alias": "combo_encrypted_v3_1000000",
              "Async Capable": false,
              "Node Type": "Bitmap Heap Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 36,
              "Plans": [
                {
                  "Async Capable": false,
                  "Index Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbK!3*9Wj3Ebjn%k4>m4}Mm}6`(v@<Bd}Lk-<0w)wxuYz+o_vI99|UY>xTT!hH9xjbl!rG=xu{D9cc=Q<Vp?Q7~B}zHg8Tq;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_1000000\"}, \"v\": 3, \"bf\": [567, 1471, 1346, 943, 1673, 1076], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c3f46c4366be9263b48e2afe0f27583eb6e552622125d4aa23fdbb7957ef81237e21f4e83da01a612e3b694c1e2712775e807bf09a080deddf91f56fe608fb890617acdf628508e82178e5d065b8b8ff6c80c9a5ea3e90b61220ed785e48e4055e2ab8467e14688718391f29ebb7c1b7f50fd0f269f4a776b9d64eb14cf8b065fa5115f6a6dac4db7f0aaa037e303aea1e619fac9d31245a8b09010a2f111d8fc7f204541aec8304cd717815c720010f57772e217393b84236a36d151fe428d34402fb657078afaec288cfd770b6fc9fc151bf20cfc9141b30f38ea888115bbec786fb0498ee7173d379a4d91ec4284c0218047ebc50195952508246d4880ef27379675f731b7416c2fd0a483b13f0599d\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
                  "Index Name": "combo_encrypted_v3_1000000_name_match_gin_index",
                  "Node Type": "Bitmap Index Scan",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 1,
                  "Plan Width": 0,
                  "Startup Cost": 0.0,
                  "Total Cost": 290.85
                }
              ],
              "Recheck Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbK!3*9Wj3Ebjn%k4>m4}Mm}6`(v@<Bd}Lk-<0w)wxuYz+o_vI99|UY>xTT!hH9xjbl!rG=xu{D9cc=Q<Vp?Q7~B}zHg8Tq;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_1000000\"}, \"v\": 3, \"bf\": [567, 1471, 1346, 943, 1673, 1076], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c3f46c4366be9263b48e2afe0f27583eb6e552622125d4aa23fdbb7957ef81237e21f4e83da01a612e3b694c1e2712775e807bf09a080deddf91f56fe608fb890617acdf628508e82178e5d065b8b8ff6c80c9a5ea3e90b61220ed785e48e4055e2ab8467e14688718391f29ebb7c1b7f50fd0f269f4a776b9d64eb14cf8b065fa5115f6a6dac4db7f0aaa037e303aea1e619fac9d31245a8b09010a2f111d8fc7f204541aec8304cd717815c720010f57772e217393b84236a36d151fe428d34402fb657078afaec288cfd770b6fc9fc151bf20cfc9141b30f38ea888115bbec786fb0498ee7173d379a4d91ec4284c0218047ebc50195952508246d4880ef27379675f731b7416c2fd0a483b13f0599d\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Relation Name": "combo_encrypted_v3_1000000",
              "Startup Cost": 290.85,
              "Total Cost": 295.61
            }
          ],
          "Sort Key": [
            "(eql_v3_internal.ore_block_256((age)::jsonb))"
          ],
          "Startup Cost": 295.62,
          "Total Cost": 295.63
        }
      ],
      "Startup Cost": 295.62,
      "Total Cost": 295.63
    }
  }
]
```

</details>

![Query Performance - COMBO/bloom_ore_order_limit](query_combo_bloom_ore_order_limit_chart.png)

## filtered_group_by

**Description:** Composite predicate: filter by name pattern, GROUP BY category

**SQL Query:**
```sql
SELECT eql_v2.hmac_256(category), count(*) FROM {TABLE} WHERE name LIKE $1 GROUP BY 1
```

**Parameter:** `Bob`

**Table: `combo_encrypted_{rows}`. Query: `SELECT eql_v2.hmac_256(category), count(*) FROM tbl WHERE name LIKE $1 GROUP BY 1`. Bloom filter on `name` filters the input set; HashAggregate then groups the small post-filter set by the 32-byte category HMAC. With ~0.01-0.1% of names matching a typical bloom pattern and 250 category buckets, the aggregate stage is essentially free — the cost is bloom filter scan plus per-matching-row HMAC.**

**Indexes available on the table:**
```sql
CREATE INDEX
combo_encrypted_10000_name_gin_index
ON combo_encrypted_10000 USING GIN (
    eql_v2.bloom_filter(name)
);

CREATE INDEX
combo_encrypted_10000_age_ore_index
ON combo_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(age)
);

CREATE INDEX
combo_encrypted_10000_category_hash_index
ON combo_encrypted_10000 USING hash (
    eql_v2.hmac_256(category)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `combo_encrypted_v3_10000_name_match_gin_index`
- 100,000: `combo_encrypted_v3_100000_name_match_gin_index`
- 1,000,000: `combo_encrypted_v3_1000000_name_match_gin_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 4 | 233.91μs | N/A |
| 100,000 | 51 | 788.22μs | N/A |
| 1,000,000 | 237 | 5.36ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Aggregate (Sorted)
  Sort
    Bitmap Heap Scan on combo_encrypted_v3_10000
      Bitmap Index Scan using combo_encrypted_v3_10000_name_match_gin_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Group Key": [
        "((((category)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
      ],
      "Node Type": "Aggregate",
      "Parallel Aware": false,
      "Partial Mode": "Simple",
      "Plan Rows": 1,
      "Plan Width": 40,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 32,
          "Plans": [
            {
              "Alias": "combo_encrypted_v3_10000",
              "Async Capable": false,
              "Node Type": "Bitmap Heap Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 32,
              "Plans": [
                {
                  "Async Capable": false,
                  "Index Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbJUhfrb`JEtIr&txLSDer*96|pI`;UA|QJV{i$g$y<EU?sjguY|-PvnXDMli))^aiIwTC3QLk=nv&oAd3jZ!Qg-eW6MA}q;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [1673, 1471, 943, 567, 1076, 1346], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c3aecf3452b74675acef3f5613e934679e03b79198df0812554b870b2093c07662e3fbd988db90b2d4319a61787838414b58014bd29dbc9d057b7f7ae534bbe5dfa27a243d63f5186f5a65b4b7611b59bd51d644dd4e00aa61d394070789fe164f1de6b63fe02d3b2823d8d35c980094a906ef097621ac99ed96cc8e8d0173e91a43b632830a5e6f0de27c1dc664b58f48db8bf2972938604cd222ff487bf862985dd011663d85548cc287887591f290356736ec0e0912397e10f96ab9ebc925a7545ebc2196052aae2ed52aa8bb1fafca6002b9f06d67298225e17dc3763e9447d777f79c2d974a4768f5246e31b2201685cab1cbdf20056f0cab9072d443ea88570b5838aebd518ac4cf5a93d4f2db00\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
                  "Index Name": "combo_encrypted_v3_10000_name_match_gin_index",
                  "Node Type": "Bitmap Index Scan",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 1,
                  "Plan Width": 0,
                  "Startup Cost": 0.0,
                  "Total Cost": 56.47
                }
              ],
              "Recheck Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbJUhfrb`JEtIr&txLSDer*96|pI`;UA|QJV{i$g$y<EU?sjguY|-PvnXDMli))^aiIwTC3QLk=nv&oAd3jZ!Qg-eW6MA}q;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [1673, 1471, 943, 567, 1076, 1346], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c3aecf3452b74675acef3f5613e934679e03b79198df0812554b870b2093c07662e3fbd988db90b2d4319a61787838414b58014bd29dbc9d057b7f7ae534bbe5dfa27a243d63f5186f5a65b4b7611b59bd51d644dd4e00aa61d394070789fe164f1de6b63fe02d3b2823d8d35c980094a906ef097621ac99ed96cc8e8d0173e91a43b632830a5e6f0de27c1dc664b58f48db8bf2972938604cd222ff487bf862985dd011663d85548cc287887591f290356736ec0e0912397e10f96ab9ebc925a7545ebc2196052aae2ed52aa8bb1fafca6002b9f06d67298225e17dc3763e9447d777f79c2d974a4768f5246e31b2201685cab1cbdf20056f0cab9072d443ea88570b5838aebd518ac4cf5a93d4f2db00\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Relation Name": "combo_encrypted_v3_10000",
              "Startup Cost": 56.47,
              "Total Cost": 60.99
            }
          ],
          "Sort Key": [
            "((((category)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
          ],
          "Startup Cost": 61.0,
          "Total Cost": 61.01
        }
      ],
      "Startup Cost": 61.0,
      "Strategy": "Sorted",
      "Total Cost": 61.02
    }
  }
]
```

**100,000 rows**

```
Aggregate (Sorted)
  Sort
    Bitmap Heap Scan on combo_encrypted_v3_100000
      Bitmap Index Scan using combo_encrypted_v3_100000_name_match_gin_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Group Key": [
        "((((category)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
      ],
      "Node Type": "Aggregate",
      "Parallel Aware": false,
      "Partial Mode": "Simple",
      "Plan Rows": 1,
      "Plan Width": 40,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 32,
          "Plans": [
            {
              "Alias": "combo_encrypted_v3_100000",
              "Async Capable": false,
              "Node Type": "Bitmap Heap Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 32,
              "Plans": [
                {
                  "Async Capable": false,
                  "Index Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbLH8|5ql4M2Q_Pp`vyaAZWp6+X-lgSi|Z5MHiX>KE^)(=s*NOLxQ|R!;$YnKh-wL)>^x9Zn|24Dt^$rm)!b)|?E^*5Z;+q;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [1471, 943, 1346, 1076, 567, 1673], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c35bc10e47437489ceccd697f83c193e402f589e059b223b6367059997df64e15305798f235df5f24135f52eceeed140c773b69166ff84df05ef43cd4979584f524f55026fa0e987397b59cfec41626e7fcd430dd7d4d72a501f0d3e20c3f57e7e6846351d12010108c409522fd5076798228081070549dd640d1a0092d002d9f9b62805158abae3850b48b25f7ba2424196b1224ecf056dd4afce29f352a480118f4c9f2c0fe7331f862cbddff09432f8d19dc48cfcb784e01ba365e32137e3eaea6c4c2acd4d218973067a199c1a9bd37e4047c766fb9a357e39931fa21c24725455138a6d75799b92be48db6550ef6d18539fafae82d416eccded84d87789f48d9353709ec734ac6b535f73c7c86e43\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
                  "Index Name": "combo_encrypted_v3_100000_name_match_gin_index",
                  "Node Type": "Bitmap Index Scan",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 1,
                  "Plan Width": 0,
                  "Startup Cost": 0.0,
                  "Total Cost": 93.6
                }
              ],
              "Recheck Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbLH8|5ql4M2Q_Pp`vyaAZWp6+X-lgSi|Z5MHiX>KE^)(=s*NOLxQ|R!;$YnKh-wL)>^x9Zn|24Dt^$rm)!b)|?E^*5Z;+q;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [1471, 943, 1346, 1076, 567, 1673], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c35bc10e47437489ceccd697f83c193e402f589e059b223b6367059997df64e15305798f235df5f24135f52eceeed140c773b69166ff84df05ef43cd4979584f524f55026fa0e987397b59cfec41626e7fcd430dd7d4d72a501f0d3e20c3f57e7e6846351d12010108c409522fd5076798228081070549dd640d1a0092d002d9f9b62805158abae3850b48b25f7ba2424196b1224ecf056dd4afce29f352a480118f4c9f2c0fe7331f862cbddff09432f8d19dc48cfcb784e01ba365e32137e3eaea6c4c2acd4d218973067a199c1a9bd37e4047c766fb9a357e39931fa21c24725455138a6d75799b92be48db6550ef6d18539fafae82d416eccded84d87789f48d9353709ec734ac6b535f73c7c86e43\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Relation Name": "combo_encrypted_v3_100000",
              "Startup Cost": 93.6,
              "Total Cost": 98.12
            }
          ],
          "Sort Key": [
            "((((category)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
          ],
          "Startup Cost": 98.13,
          "Total Cost": 98.13
        }
      ],
      "Startup Cost": 98.13,
      "Strategy": "Sorted",
      "Total Cost": 98.15
    }
  }
]
```

**1,000,000 rows**

```
Aggregate (Sorted)
  Sort
    Bitmap Heap Scan on combo_encrypted_v3_1000000
      Bitmap Index Scan using combo_encrypted_v3_1000000_name_match_gin_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Group Key": [
        "((((category)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
      ],
      "Node Type": "Aggregate",
      "Parallel Aware": false,
      "Partial Mode": "Simple",
      "Plan Rows": 1,
      "Plan Width": 40,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 32,
          "Plans": [
            {
              "Alias": "combo_encrypted_v3_1000000",
              "Async Capable": false,
              "Node Type": "Bitmap Heap Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 32,
              "Plans": [
                {
                  "Async Capable": false,
                  "Index Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbKd&X$zB8z`*yRf!732|Aj@6>qjSB^1sS$Vw%h{e>9i#x~6z*1*IdoEQeVkw1nOVCi8ZO{!0oK#}5CWZZ8~N4Rlnr$L7wq;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_1000000\"}, \"v\": 3, \"bf\": [567, 1076, 1471, 1346, 1673, 943], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c37ac803e7e33616e24d657cb0c13195c32451b0380519d765dbe59a22428dd5e37ad4c87ec1aa1e6dcf9b85af628b3b57d34aa882d23a7d616f346ee8f14d649151d95ccfa6f3aed289823153566a3c0440a315174426e0d07a15733bab9c973e5b7078ae0534ae9bcac756810b2785517f28dcb03722cc94af137415fa1c041bb1821a44875c8bbb0c7f8a1da06c77042bcd5bfd799ca1cd8333d31ed01bfddcfe0a9fadca1ba048a46f9a02ce82f68e78e1c9829fa4552dd71a205a30bca22be8b23ca2c83290983ac3ba5a9f73c0f502b267e1f462149417e3e28b29fd96fafc934c1ca29a9b78d9fbec0deaa25fd5d4b7186e680c64d2136efb47ff8f27364acb539da1a849148f74aed73200c62e\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
                  "Index Name": "combo_encrypted_v3_1000000_name_match_gin_index",
                  "Node Type": "Bitmap Index Scan",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 1,
                  "Plan Width": 0,
                  "Startup Cost": 0.0,
                  "Total Cost": 290.85
                }
              ],
              "Recheck Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbKd&X$zB8z`*yRf!732|Aj@6>qjSB^1sS$Vw%h{e>9i#x~6z*1*IdoEQeVkw1nOVCi8ZO{!0oK#}5CWZZ8~N4Rlnr$L7wq;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_1000000\"}, \"v\": 3, \"bf\": [567, 1076, 1471, 1346, 1673, 943], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c37ac803e7e33616e24d657cb0c13195c32451b0380519d765dbe59a22428dd5e37ad4c87ec1aa1e6dcf9b85af628b3b57d34aa882d23a7d616f346ee8f14d649151d95ccfa6f3aed289823153566a3c0440a315174426e0d07a15733bab9c973e5b7078ae0534ae9bcac756810b2785517f28dcb03722cc94af137415fa1c041bb1821a44875c8bbb0c7f8a1da06c77042bcd5bfd799ca1cd8333d31ed01bfddcfe0a9fadca1ba048a46f9a02ce82f68e78e1c9829fa4552dd71a205a30bca22be8b23ca2c83290983ac3ba5a9f73c0f502b267e1f462149417e3e28b29fd96fafc934c1ca29a9b78d9fbec0deaa25fd5d4b7186e680c64d2136efb47ff8f27364acb539da1a849148f74aed73200c62e\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Relation Name": "combo_encrypted_v3_1000000",
              "Startup Cost": 290.85,
              "Total Cost": 295.37
            }
          ],
          "Sort Key": [
            "((((category)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
          ],
          "Startup Cost": 295.38,
          "Total Cost": 295.38
        }
      ],
      "Startup Cost": 295.38,
      "Strategy": "Sorted",
      "Total Cost": 295.4
    }
  }
]
```

</details>

![Query Performance - COMBO/filtered_group_by](query_combo_filtered_group_by_chart.png)

## top_n_filtered_group_by

**Description:** Dashboard analytic: top 10 categories for customers matching a name pattern

**SQL Query:**
```sql
SELECT eql_v2.hmac_256(category), count(*) FROM {TABLE} WHERE name LIKE $1 GROUP BY 1 ORDER BY count(*) DESC LIMIT 10
```

**Parameter:** `Bob`

**Table: `combo_encrypted_{rows}`. Query: `SELECT eql_v2.hmac_256(category), count(*) FROM tbl WHERE name LIKE $1 GROUP BY 1 ORDER BY count(*) DESC LIMIT 10`. Same shape as `filtered_group_by` with an outer Top-N sort + LIMIT 10. Realistic analytics shape for surfacing the categories that contain the most customers matching a filter, without revealing the underlying names or category labels.**

**Indexes available on the table:**
```sql
CREATE INDEX
combo_encrypted_10000_name_gin_index
ON combo_encrypted_10000 USING GIN (
    eql_v2.bloom_filter(name)
);

CREATE INDEX
combo_encrypted_10000_age_ore_index
ON combo_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(age)
);

CREATE INDEX
combo_encrypted_10000_category_hash_index
ON combo_encrypted_10000 USING hash (
    eql_v2.hmac_256(category)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `combo_encrypted_v3_10000_name_match_gin_index`
- 100,000: `combo_encrypted_v3_100000_name_match_gin_index`
- 1,000,000: `combo_encrypted_v3_1000000_name_match_gin_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 4 | 245.09μs | N/A |
| 100,000 | 10 | 786.23μs | N/A |
| 1,000,000 | 10 | 5.25ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Sort
    Aggregate (Sorted)
      Sort
        Bitmap Heap Scan on combo_encrypted_v3_10000
          Bitmap Index Scan using combo_encrypted_v3_10000_name_match_gin_index
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
      "Plan Width": 40,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 40,
          "Plans": [
            {
              "Async Capable": false,
              "Group Key": [
                "((((category)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
              ],
              "Node Type": "Aggregate",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Partial Mode": "Simple",
              "Plan Rows": 1,
              "Plan Width": 40,
              "Plans": [
                {
                  "Async Capable": false,
                  "Node Type": "Sort",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 1,
                  "Plan Width": 32,
                  "Plans": [
                    {
                      "Alias": "combo_encrypted_v3_10000",
                      "Async Capable": false,
                      "Node Type": "Bitmap Heap Scan",
                      "Parallel Aware": false,
                      "Parent Relationship": "Outer",
                      "Plan Rows": 1,
                      "Plan Width": 32,
                      "Plans": [
                        {
                          "Async Capable": false,
                          "Index Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbLJ@{d+gEr3m|9$?(a<5Y^o6;FpZkwW+twq$9c1+3#Hbqm5C`c1?j6<LFe8Y00&w<0)o2VU}1Ju_|dNZZEd|8>pob<4Idq;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [1471, 1346, 567, 1076, 943, 1673], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c357a2e1e684606d053a7d2ea49601d879065c9cfcdcca0cdbfaa451191dabbd50cd556c39b6153a2254d357110460b9b07e97eb7704c3bd2e41190846ea00770cd425c4de22b019e2ef89a44d4a82e106f6a6698cb3d099040376ead796b31d400d66583dc98d18d8c6c5ad0ae5c3b73e572f13127872afbd811fc4f37a0ffd8d15db4add60cf7d6c9f8fee87e7b40214764f859f3177b8a9a80be27d1fe4b04eed3392a810b4f3da263205f3ce3563350be0e9783e3b33fb5a357c727fce4fb325f3074064e90b18dc875a200d888b6028afbf2cbf08e35e171e15b32fa6bd35b57a9e32347ea5cd91aaa200fa1059e614ee03e093ce7e49b2ca66bed0db7265e13f46ba7a55d4acdefa9d4f9cd88145\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
                          "Index Name": "combo_encrypted_v3_10000_name_match_gin_index",
                          "Node Type": "Bitmap Index Scan",
                          "Parallel Aware": false,
                          "Parent Relationship": "Outer",
                          "Plan Rows": 1,
                          "Plan Width": 0,
                          "Startup Cost": 0.0,
                          "Total Cost": 56.47
                        }
                      ],
                      "Recheck Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbLJ@{d+gEr3m|9$?(a<5Y^o6;FpZkwW+twq$9c1+3#Hbqm5C`c1?j6<LFe8Y00&w<0)o2VU}1Ju_|dNZZEd|8>pob<4Idq;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [1471, 1346, 567, 1076, 943, 1673], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c357a2e1e684606d053a7d2ea49601d879065c9cfcdcca0cdbfaa451191dabbd50cd556c39b6153a2254d357110460b9b07e97eb7704c3bd2e41190846ea00770cd425c4de22b019e2ef89a44d4a82e106f6a6698cb3d099040376ead796b31d400d66583dc98d18d8c6c5ad0ae5c3b73e572f13127872afbd811fc4f37a0ffd8d15db4add60cf7d6c9f8fee87e7b40214764f859f3177b8a9a80be27d1fe4b04eed3392a810b4f3da263205f3ce3563350be0e9783e3b33fb5a357c727fce4fb325f3074064e90b18dc875a200d888b6028afbf2cbf08e35e171e15b32fa6bd35b57a9e32347ea5cd91aaa200fa1059e614ee03e093ce7e49b2ca66bed0db7265e13f46ba7a55d4acdefa9d4f9cd88145\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
                      "Relation Name": "combo_encrypted_v3_10000",
                      "Startup Cost": 56.47,
                      "Total Cost": 60.99
                    }
                  ],
                  "Sort Key": [
                    "((((category)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
                  ],
                  "Startup Cost": 61.0,
                  "Total Cost": 61.01
                }
              ],
              "Startup Cost": 61.0,
              "Strategy": "Sorted",
              "Total Cost": 61.02
            }
          ],
          "Sort Key": [
            "(count(*)) DESC"
          ],
          "Startup Cost": 61.03,
          "Total Cost": 61.04
        }
      ],
      "Startup Cost": 61.03,
      "Total Cost": 61.04
    }
  }
]
```

**100,000 rows**

```
Limit
  Sort
    Aggregate (Sorted)
      Sort
        Bitmap Heap Scan on combo_encrypted_v3_100000
          Bitmap Index Scan using combo_encrypted_v3_100000_name_match_gin_index
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
      "Plan Width": 40,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 40,
          "Plans": [
            {
              "Async Capable": false,
              "Group Key": [
                "((((category)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
              ],
              "Node Type": "Aggregate",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Partial Mode": "Simple",
              "Plan Rows": 1,
              "Plan Width": 40,
              "Plans": [
                {
                  "Async Capable": false,
                  "Node Type": "Sort",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 1,
                  "Plan Width": 32,
                  "Plans": [
                    {
                      "Alias": "combo_encrypted_v3_100000",
                      "Async Capable": false,
                      "Node Type": "Bitmap Heap Scan",
                      "Parallel Aware": false,
                      "Parent Relationship": "Outer",
                      "Plan Rows": 1,
                      "Plan Width": 32,
                      "Plans": [
                        {
                          "Async Capable": false,
                          "Index Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbM6EwWN2VRz*I3DdDB#Fmo86+?kTUDQ)3C~$YCfpIWtH3*?`N1MbTs_dsnSw}Szu~U`$sq7vNWOvE$ptPi!+(lKfM!NLJq;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [567, 943, 1673, 1346, 1076, 1471], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c34318ae765150f77b3a53a263bc7d2993112769788be44c691b6bb7dd0d108287ccf62da1c7b66cce8a289f4b0952dda6525ef989cd7dc0234274f4210812bc6c6cf77842215f4ee0069d5d53a44978ba9afa08fc508dfb429c9ae59fe09b1273671429c50943643b8d06a953cefefaff93a57437960fae6da97d06605a455e9d42ea642245ab451b32f59c3377a7d22e6cdaf03ea0db72f487cc24ec8cffa2f93bee652ef90dd93ef1a753a3c6b9ec87c1685b79444cdeeff126163034a7b0250223b500e6739e976a0feac92b54cde7c0249b106c1ceb8617407131b893b688bdfaba895f2345ae848dac6078c107cdce8aa0b27658eb23c31483eaa2189f56b7cda0d2d5b068905f006440337a0ae6\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
                          "Index Name": "combo_encrypted_v3_100000_name_match_gin_index",
                          "Node Type": "Bitmap Index Scan",
                          "Parallel Aware": false,
                          "Parent Relationship": "Outer",
                          "Plan Rows": 1,
                          "Plan Width": 0,
                          "Startup Cost": 0.0,
                          "Total Cost": 93.6
                        }
                      ],
                      "Recheck Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbM6EwWN2VRz*I3DdDB#Fmo86+?kTUDQ)3C~$YCfpIWtH3*?`N1MbTs_dsnSw}Szu~U`$sq7vNWOvE$ptPi!+(lKfM!NLJq;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [567, 943, 1673, 1346, 1076, 1471], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c34318ae765150f77b3a53a263bc7d2993112769788be44c691b6bb7dd0d108287ccf62da1c7b66cce8a289f4b0952dda6525ef989cd7dc0234274f4210812bc6c6cf77842215f4ee0069d5d53a44978ba9afa08fc508dfb429c9ae59fe09b1273671429c50943643b8d06a953cefefaff93a57437960fae6da97d06605a455e9d42ea642245ab451b32f59c3377a7d22e6cdaf03ea0db72f487cc24ec8cffa2f93bee652ef90dd93ef1a753a3c6b9ec87c1685b79444cdeeff126163034a7b0250223b500e6739e976a0feac92b54cde7c0249b106c1ceb8617407131b893b688bdfaba895f2345ae848dac6078c107cdce8aa0b27658eb23c31483eaa2189f56b7cda0d2d5b068905f006440337a0ae6\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
                      "Relation Name": "combo_encrypted_v3_100000",
                      "Startup Cost": 93.6,
                      "Total Cost": 98.12
                    }
                  ],
                  "Sort Key": [
                    "((((category)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
                  ],
                  "Startup Cost": 98.13,
                  "Total Cost": 98.13
                }
              ],
              "Startup Cost": 98.13,
              "Strategy": "Sorted",
              "Total Cost": 98.15
            }
          ],
          "Sort Key": [
            "(count(*)) DESC"
          ],
          "Startup Cost": 98.16,
          "Total Cost": 98.16
        }
      ],
      "Startup Cost": 98.16,
      "Total Cost": 98.16
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Sort
    Aggregate (Sorted)
      Sort
        Bitmap Heap Scan on combo_encrypted_v3_1000000
          Bitmap Index Scan using combo_encrypted_v3_1000000_name_match_gin_index
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
      "Plan Width": 40,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 40,
          "Plans": [
            {
              "Async Capable": false,
              "Group Key": [
                "((((category)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
              ],
              "Node Type": "Aggregate",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Partial Mode": "Simple",
              "Plan Rows": 1,
              "Plan Width": 40,
              "Plans": [
                {
                  "Async Capable": false,
                  "Node Type": "Sort",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 1,
                  "Plan Width": 32,
                  "Plans": [
                    {
                      "Alias": "combo_encrypted_v3_1000000",
                      "Async Capable": false,
                      "Node Type": "Bitmap Heap Scan",
                      "Parallel Aware": false,
                      "Parent Relationship": "Outer",
                      "Plan Rows": 1,
                      "Plan Width": 32,
                      "Plans": [
                        {
                          "Async Capable": false,
                          "Index Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbK?n#yHO8S<9OhZ5wwj!b#P6_t|pj)n8K?Zo~dhrsIBju0m)@$tkUG0hZ}l)P{ufwd!^FOb85Lx~{q0-yfvT$p$FPm6Rsq;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_1000000\"}, \"v\": 3, \"bf\": [1471, 567, 943, 1076, 1673, 1346], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c3f5daee23452cdc6846ef2bd879e525ea569d3c31dbe369d74c759d578f4c5a5a97a9c8f407a7058b6995f1df3f07cbdf9f7fc96dfc1dfd8ac223d5e8d50a008f3b9d6cded770d4fa28534887f6e9af5130cf7ca08321dc3572af25a063d72618ecdaef2ab55bbb159abea62804432d8d289f0f9aa78adcfda13aa8ca7269e4e34e74bf5ca2045f5b174bdfb58337b8e0ddd6824dc6d6cb1ec3553fd88a2b1d259c33ab069a5d564e2a981077324f60431a8294d8cae96f5b8f120b164c5a2341a34e3242bd4102eedefc38665e5990703d9ed7bbd22c2b7bdb905ed14ed67e6b1a87b6a1544d58e1a69d45d11ddd19630d374848465eefee3327cfedc631f70230eadbbf80547c17dac2524a297e6720\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
                          "Index Name": "combo_encrypted_v3_1000000_name_match_gin_index",
                          "Node Type": "Bitmap Index Scan",
                          "Parallel Aware": false,
                          "Parent Relationship": "Outer",
                          "Plan Rows": 1,
                          "Plan Width": 0,
                          "Startup Cost": 0.0,
                          "Total Cost": 290.85
                        }
                      ],
                      "Recheck Cond": "((eql_v3_internal.bloom_filter((name)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbK?n#yHO8S<9OhZ5wwj!b#P6_t|pj)n8K?Zo~dhrsIBju0m)@$tkUG0hZ}l)P{ufwd!^FOb85Lx~{q0-yfvT$p$FPm6Rsq;6qtWyBEWG)cPL_Dk-y$tP`SR?}h\", \"i\": {\"c\": \"name\", \"t\": \"combo_encrypted_v3_1000000\"}, \"v\": 3, \"bf\": [1471, 567, 943, 1076, 1673, 1346], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c3f5daee23452cdc6846ef2bd879e525ea569d3c31dbe369d74c759d578f4c5a5a97a9c8f407a7058b6995f1df3f07cbdf9f7fc96dfc1dfd8ac223d5e8d50a008f3b9d6cded770d4fa28534887f6e9af5130cf7ca08321dc3572af25a063d72618ecdaef2ab55bbb159abea62804432d8d289f0f9aa78adcfda13aa8ca7269e4e34e74bf5ca2045f5b174bdfb58337b8e0ddd6824dc6d6cb1ec3553fd88a2b1d259c33ab069a5d564e2a981077324f60431a8294d8cae96f5b8f120b164c5a2341a34e3242bd4102eedefc38665e5990703d9ed7bbd22c2b7bdb905ed14ed67e6b1a87b6a1544d58e1a69d45d11ddd19630d374848465eefee3327cfedc631f70230eadbbf80547c17dac2524a297e6720\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
                      "Relation Name": "combo_encrypted_v3_1000000",
                      "Startup Cost": 290.85,
                      "Total Cost": 295.37
                    }
                  ],
                  "Sort Key": [
                    "((((category)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
                  ],
                  "Startup Cost": 295.38,
                  "Total Cost": 295.38
                }
              ],
              "Startup Cost": 295.38,
              "Strategy": "Sorted",
              "Total Cost": 295.4
            }
          ],
          "Sort Key": [
            "(count(*)) DESC"
          ],
          "Startup Cost": 295.41,
          "Total Cost": 295.41
        }
      ],
      "Startup Cost": 295.41,
      "Total Cost": 295.41
    }
  }
]
```

</details>

![Query Performance - COMBO/top_n_filtered_group_by](query_combo_top_n_filtered_group_by_chart.png)

