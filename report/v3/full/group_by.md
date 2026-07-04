# GROUP_BY Queries

[← Back to overview](./BENCHMARK_REPORT.md)

Per-tier query performance. Each scenario lists its SQL, the indexes available on the target table, the indexes the planner actually picked per tier, the timing table, and the full EXPLAIN plan in a collapsed block.

## low_cardinality_groups_encrypted

**Description:** Low-cardinality GROUP BY (~250 buckets) on `eql_v2.hmac_256(value)`, wrapped in `count(*)` to isolate aggregation cost from emit cost

**SQL Query:**
```sql
SELECT count(*) FROM (SELECT 1 FROM {TABLE} GROUP BY eql_v2.hmac_256(value)) g
```

**Table: `category_encrypted_{rows}` with encrypted categorical values (`CAT_001`..`CAT_250`, uniform random — ~250 distinct buckets). The encrypted value carries an `hm` HMAC term via the `unique` search index. **Index: hash index on `eql_v2.hmac_256(value)`, but `GROUP BY` doesn't engage it directly** — the planner picks `HashAggregate`, building an in-memory hash table keyed on the 32-byte HMAC. With only 250 distinct keys the hash table fits comfortably in default `work_mem`. The outer `count(*)` keeps the result-set emission at exactly one row, so wall-clock time tracks aggregation cost. The companion `low_cardinality_groups_plaintext` scenario runs the same query shape against an unindexed TEXT column for a baseline.**

**Indexes available on the table:**
```sql
CREATE INDEX
category_encrypted_10000_hash_index
ON category_encrypted_10000 using hash (
    eql_v2.hmac_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 1 | 2.17ms | N/A |
| 100,000 | 1 | 19.31ms | N/A |
| 1,000,000 | 1 | 83.25ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Aggregate
  Aggregate (Hashed)
    Seq Scan on category_encrypted_v3_10000
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
          "Async Capable": false,
          "Group Key": [
            "(((category_encrypted_v3_10000.value)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256"
          ],
          "Node Type": "Aggregate",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Partial Mode": "Simple",
          "Plan Rows": 250,
          "Plan Width": 36,
          "Planned Partitions": 0,
          "Plans": [
            {
              "Alias": "category_encrypted_v3_10000",
              "Async Capable": false,
              "Node Type": "Seq Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 10000,
              "Plan Width": 32,
              "Relation Name": "category_encrypted_v3_10000",
              "Startup Cost": 0.0,
              "Total Cost": 542.0
            }
          ],
          "Startup Cost": 567.0,
          "Strategy": "Hashed",
          "Total Cost": 570.12
        }
      ],
      "Startup Cost": 573.25,
      "Strategy": "Plain",
      "Total Cost": 573.26
    }
  }
]
```

**100,000 rows**

```
Aggregate
  Aggregate (Hashed)
    Seq Scan on category_encrypted_v3_100000
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
          "Async Capable": false,
          "Group Key": [
            "(((category_encrypted_v3_100000.value)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256"
          ],
          "Node Type": "Aggregate",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Partial Mode": "Simple",
          "Plan Rows": 250,
          "Plan Width": 36,
          "Planned Partitions": 0,
          "Plans": [
            {
              "Alias": "category_encrypted_v3_100000",
              "Async Capable": false,
              "Node Type": "Seq Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 100000,
              "Plan Width": 32,
              "Relation Name": "category_encrypted_v3_100000",
              "Startup Cost": 0.0,
              "Total Cost": 5417.0
            }
          ],
          "Startup Cost": 5667.0,
          "Strategy": "Hashed",
          "Total Cost": 5670.12
        }
      ],
      "Startup Cost": 5673.25,
      "Strategy": "Plain",
      "Total Cost": 5673.26
    }
  }
]
```

**1,000,000 rows**

```
Aggregate
  Group
    Gather Merge
      Sort
        Aggregate (Hashed)
          Seq Scan on category_encrypted_v3_1000000
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
          "Async Capable": false,
          "Group Key": [
            "((((category_encrypted_v3_1000000.value)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
          ],
          "Node Type": "Group",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 250,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Node Type": "Gather Merge",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 500,
              "Plan Width": 32,
              "Plans": [
                {
                  "Async Capable": false,
                  "Node Type": "Sort",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 250,
                  "Plan Width": 32,
                  "Plans": [
                    {
                      "Async Capable": false,
                      "Group Key": [
                        "(((category_encrypted_v3_1000000.value)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256"
                      ],
                      "Node Type": "Aggregate",
                      "Parallel Aware": false,
                      "Parent Relationship": "Outer",
                      "Partial Mode": "Partial",
                      "Plan Rows": 250,
                      "Plan Width": 32,
                      "Planned Partitions": 0,
                      "Plans": [
                        {
                          "Alias": "category_encrypted_v3_1000000",
                          "Async Capable": false,
                          "Node Type": "Seq Scan",
                          "Parallel Aware": true,
                          "Parent Relationship": "Outer",
                          "Plan Rows": 416665,
                          "Plan Width": 32,
                          "Relation Name": "category_encrypted_v3_1000000",
                          "Startup Cost": 0.0,
                          "Total Cost": 46875.32
                        }
                      ],
                      "Startup Cost": 47916.98,
                      "Strategy": "Hashed",
                      "Total Cost": 47920.1
                    }
                  ],
                  "Sort Key": [
                    "((((category_encrypted_v3_1000000.value)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
                  ],
                  "Startup Cost": 47930.06,
                  "Total Cost": 47930.69
                }
              ],
              "Startup Cost": 48930.09,
              "Total Cost": 48988.42,
              "Workers Planned": 2
            }
          ],
          "Startup Cost": 48930.09,
          "Total Cost": 48990.3
        }
      ],
      "Startup Cost": 48993.42,
      "Strategy": "Plain",
      "Total Cost": 48993.43
    }
  }
]
```

</details>

![Query Performance - GROUP_BY/low_cardinality_groups_encrypted](query_group_by_low_cardinality_groups_encrypted_chart.png)

## low_cardinality_groups_plaintext

**Description:** Plaintext baseline: low-cardinality GROUP BY on a plain TEXT column, same query shape as the encrypted scenario

**SQL Query:**
```sql
SELECT count(*) FROM (SELECT 1 FROM {TABLE} GROUP BY value) g
```

**Table: `category_plaintext_{rows}` with the same `CAT_001`..`CAT_250` distribution (uniform random, populated by SQL via `mise run prepare:category_plaintext` — no encryption-client dependency). Index: none. The wall-clock delta between this and `low_cardinality_groups_encrypted` is the EQL recipe's overhead relative to a bare-PG aggregate at the same row count and cardinality.**

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 1 | 1.21ms | N/A |
| 100,000 | 1 | 9.60ms | N/A |
| 1,000,000 | 1 | 37.13ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Aggregate
  Aggregate (Hashed)
    Seq Scan on category_plaintext_10000
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
          "Async Capable": false,
          "Group Key": [
            "category_plaintext_10000.value"
          ],
          "Node Type": "Aggregate",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Partial Mode": "Simple",
          "Plan Rows": 250,
          "Plan Width": 12,
          "Planned Partitions": 0,
          "Plans": [
            {
              "Alias": "category_plaintext_10000",
              "Async Capable": false,
              "Node Type": "Seq Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 10000,
              "Plan Width": 8,
              "Relation Name": "category_plaintext_10000",
              "Startup Cost": 0.0,
              "Total Cost": 155.0
            }
          ],
          "Startup Cost": 180.0,
          "Strategy": "Hashed",
          "Total Cost": 182.5
        }
      ],
      "Startup Cost": 185.62,
      "Strategy": "Plain",
      "Total Cost": 185.63
    }
  }
]
```

**100,000 rows**

```
Aggregate
  Aggregate (Hashed)
    Seq Scan on category_plaintext_100000
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
          "Async Capable": false,
          "Group Key": [
            "category_plaintext_100000.value"
          ],
          "Node Type": "Aggregate",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Partial Mode": "Simple",
          "Plan Rows": 250,
          "Plan Width": 12,
          "Planned Partitions": 0,
          "Plans": [
            {
              "Alias": "category_plaintext_100000",
              "Async Capable": false,
              "Node Type": "Seq Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 100000,
              "Plan Width": 8,
              "Relation Name": "category_plaintext_100000",
              "Startup Cost": 0.0,
              "Total Cost": 1541.0
            }
          ],
          "Startup Cost": 1791.0,
          "Strategy": "Hashed",
          "Total Cost": 1793.5
        }
      ],
      "Startup Cost": 1796.62,
      "Strategy": "Plain",
      "Total Cost": 1796.63
    }
  }
]
```

**1,000,000 rows**

```
Aggregate
  Group
    Gather Merge
      Sort
        Aggregate (Hashed)
          Seq Scan on category_plaintext_1000000
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
          "Async Capable": false,
          "Group Key": [
            "category_plaintext_1000000.value"
          ],
          "Node Type": "Group",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 250,
          "Plan Width": 12,
          "Plans": [
            {
              "Async Capable": false,
              "Node Type": "Gather Merge",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 500,
              "Plan Width": 8,
              "Plans": [
                {
                  "Async Capable": false,
                  "Node Type": "Sort",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 250,
                  "Plan Width": 8,
                  "Plans": [
                    {
                      "Async Capable": false,
                      "Group Key": [
                        "category_plaintext_1000000.value"
                      ],
                      "Node Type": "Aggregate",
                      "Parallel Aware": false,
                      "Parent Relationship": "Outer",
                      "Partial Mode": "Partial",
                      "Plan Rows": 250,
                      "Plan Width": 8,
                      "Planned Partitions": 0,
                      "Plans": [
                        {
                          "Alias": "category_plaintext_1000000",
                          "Async Capable": false,
                          "Node Type": "Seq Scan",
                          "Parallel Aware": true,
                          "Parent Relationship": "Outer",
                          "Plan Rows": 416667,
                          "Plan Width": 8,
                          "Relation Name": "category_plaintext_1000000",
                          "Startup Cost": 0.0,
                          "Total Cost": 9572.67
                        }
                      ],
                      "Startup Cost": 10614.33,
                      "Strategy": "Hashed",
                      "Total Cost": 10616.83
                    }
                  ],
                  "Sort Key": [
                    "category_plaintext_1000000.value"
                  ],
                  "Startup Cost": 10626.79,
                  "Total Cost": 10627.42
                }
              ],
              "Startup Cost": 11626.82,
              "Total Cost": 11685.15,
              "Workers Planned": 2
            }
          ],
          "Startup Cost": 11626.82,
          "Total Cost": 11686.4
        }
      ],
      "Startup Cost": 11689.53,
      "Strategy": "Plain",
      "Total Cost": 11689.54
    }
  }
]
```

</details>

![Query Performance - GROUP_BY/low_cardinality_groups_plaintext](query_group_by_low_cardinality_groups_plaintext_chart.png)

## top_n_groups_encrypted

**Description:** Dashboard analytic: top 10 categories by frequency, EQL recipe form

**SQL Query:**
```sql
SELECT eql_v2.hmac_256(value), count(*) FROM {TABLE} GROUP BY 1 ORDER BY count(*) DESC LIMIT 10
```

**Table: `category_encrypted_{rows}` (same data as the `low_cardinality_*` scenarios above). Query: `SELECT eql_v2.hmac_256(value), count(*) FROM tbl GROUP BY 1 ORDER BY count(*) DESC LIMIT 10`. The bench always emits 10 rows regardless of input size, so the cost is dominated by the inner HashAggregate (per-row HMAC + hash-table insert) plus a tiny sort over the 250 group entries. Realistic shape for analytics queries that surface the most common categories in an encrypted dataset.**

**Indexes available on the table:**
```sql
CREATE INDEX
category_encrypted_10000_hash_index
ON category_encrypted_10000 using hash (
    eql_v2.hmac_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 2.11ms | N/A |
| 100,000 | 10 | 19.99ms | N/A |
| 1,000,000 | 10 | 88.02ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Sort
    Aggregate (Hashed)
      Seq Scan on category_encrypted_v3_10000
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
      "Plan Width": 40,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 250,
          "Plan Width": 40,
          "Plans": [
            {
              "Async Capable": false,
              "Group Key": [
                "(((value)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256"
              ],
              "Node Type": "Aggregate",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Partial Mode": "Simple",
              "Plan Rows": 250,
              "Plan Width": 40,
              "Planned Partitions": 0,
              "Plans": [
                {
                  "Alias": "category_encrypted_v3_10000",
                  "Async Capable": false,
                  "Node Type": "Seq Scan",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 10000,
                  "Plan Width": 32,
                  "Relation Name": "category_encrypted_v3_10000",
                  "Startup Cost": 0.0,
                  "Total Cost": 542.0
                }
              ],
              "Startup Cost": 592.0,
              "Strategy": "Hashed",
              "Total Cost": 595.12
            }
          ],
          "Sort Key": [
            "(count(*)) DESC"
          ],
          "Startup Cost": 600.53,
          "Total Cost": 601.15
        }
      ],
      "Startup Cost": 600.53,
      "Total Cost": 600.55
    }
  }
]
```

**100,000 rows**

```
Limit
  Sort
    Aggregate (Hashed)
      Seq Scan on category_encrypted_v3_100000
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
      "Plan Width": 40,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 250,
          "Plan Width": 40,
          "Plans": [
            {
              "Async Capable": false,
              "Group Key": [
                "(((value)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256"
              ],
              "Node Type": "Aggregate",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Partial Mode": "Simple",
              "Plan Rows": 250,
              "Plan Width": 40,
              "Planned Partitions": 0,
              "Plans": [
                {
                  "Alias": "category_encrypted_v3_100000",
                  "Async Capable": false,
                  "Node Type": "Seq Scan",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 100000,
                  "Plan Width": 32,
                  "Relation Name": "category_encrypted_v3_100000",
                  "Startup Cost": 0.0,
                  "Total Cost": 5417.0
                }
              ],
              "Startup Cost": 5917.0,
              "Strategy": "Hashed",
              "Total Cost": 5920.12
            }
          ],
          "Sort Key": [
            "(count(*)) DESC"
          ],
          "Startup Cost": 5925.53,
          "Total Cost": 5926.15
        }
      ],
      "Startup Cost": 5925.53,
      "Total Cost": 5925.55
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Sort
    Aggregate (Sorted)
      Gather Merge
        Sort
          Aggregate (Hashed)
            Seq Scan on category_encrypted_v3_1000000
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
      "Plan Width": 40,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 250,
          "Plan Width": 40,
          "Plans": [
            {
              "Async Capable": false,
              "Group Key": [
                "((((value)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
              ],
              "Node Type": "Aggregate",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Partial Mode": "Finalize",
              "Plan Rows": 250,
              "Plan Width": 40,
              "Plans": [
                {
                  "Async Capable": false,
                  "Node Type": "Gather Merge",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 500,
                  "Plan Width": 40,
                  "Plans": [
                    {
                      "Async Capable": false,
                      "Node Type": "Sort",
                      "Parallel Aware": false,
                      "Parent Relationship": "Outer",
                      "Plan Rows": 250,
                      "Plan Width": 40,
                      "Plans": [
                        {
                          "Async Capable": false,
                          "Group Key": [
                            "(((value)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256"
                          ],
                          "Node Type": "Aggregate",
                          "Parallel Aware": false,
                          "Parent Relationship": "Outer",
                          "Partial Mode": "Partial",
                          "Plan Rows": 250,
                          "Plan Width": 40,
                          "Planned Partitions": 0,
                          "Plans": [
                            {
                              "Alias": "category_encrypted_v3_1000000",
                              "Async Capable": false,
                              "Node Type": "Seq Scan",
                              "Parallel Aware": true,
                              "Parent Relationship": "Outer",
                              "Plan Rows": 416665,
                              "Plan Width": 32,
                              "Relation Name": "category_encrypted_v3_1000000",
                              "Startup Cost": 0.0,
                              "Total Cost": 46875.32
                            }
                          ],
                          "Startup Cost": 48958.64,
                          "Strategy": "Hashed",
                          "Total Cost": 48961.77
                        }
                      ],
                      "Sort Key": [
                        "((((value)::jsonb ->> 'hm'::text))::eql_v3_internal.hmac_256)"
                      ],
                      "Startup Cost": 48971.72,
                      "Total Cost": 48972.35
                    }
                  ],
                  "Startup Cost": 49971.75,
                  "Total Cost": 50030.09,
                  "Workers Planned": 2
                }
              ],
              "Startup Cost": 49971.75,
              "Strategy": "Sorted",
              "Total Cost": 50035.71
            }
          ],
          "Sort Key": [
            "(count(*)) DESC"
          ],
          "Startup Cost": 50041.11,
          "Total Cost": 50041.74
        }
      ],
      "Startup Cost": 50041.11,
      "Total Cost": 50041.14
    }
  }
]
```

</details>

![Query Performance - GROUP_BY/top_n_groups_encrypted](query_group_by_top_n_groups_encrypted_chart.png)

## top_n_groups_plaintext

**Description:** Plaintext baseline: top 10 categories by frequency on a plain TEXT column

**SQL Query:**
```sql
SELECT value, count(*) FROM {TABLE} GROUP BY 1 ORDER BY count(*) DESC LIMIT 10
```

**Table: `category_plaintext_{rows}`. Same query shape as the encrypted top-N scenario; the delta is the EQL recipe's overhead for the same shape on the same cardinality data.**

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 1.24ms | N/A |
| 100,000 | 10 | 10.07ms | N/A |
| 1,000,000 | 10 | 38.70ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Sort
    Aggregate (Hashed)
      Seq Scan on category_plaintext_10000
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
      "Plan Width": 16,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 250,
          "Plan Width": 16,
          "Plans": [
            {
              "Async Capable": false,
              "Group Key": [
                "value"
              ],
              "Node Type": "Aggregate",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Partial Mode": "Simple",
              "Plan Rows": 250,
              "Plan Width": 16,
              "Planned Partitions": 0,
              "Plans": [
                {
                  "Alias": "category_plaintext_10000",
                  "Async Capable": false,
                  "Node Type": "Seq Scan",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 10000,
                  "Plan Width": 8,
                  "Relation Name": "category_plaintext_10000",
                  "Startup Cost": 0.0,
                  "Total Cost": 155.0
                }
              ],
              "Startup Cost": 205.0,
              "Strategy": "Hashed",
              "Total Cost": 207.5
            }
          ],
          "Sort Key": [
            "(count(*)) DESC"
          ],
          "Startup Cost": 212.9,
          "Total Cost": 213.53
        }
      ],
      "Startup Cost": 212.9,
      "Total Cost": 212.93
    }
  }
]
```

**100,000 rows**

```
Limit
  Sort
    Aggregate (Hashed)
      Seq Scan on category_plaintext_100000
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
      "Plan Width": 16,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 250,
          "Plan Width": 16,
          "Plans": [
            {
              "Async Capable": false,
              "Group Key": [
                "value"
              ],
              "Node Type": "Aggregate",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Partial Mode": "Simple",
              "Plan Rows": 250,
              "Plan Width": 16,
              "Planned Partitions": 0,
              "Plans": [
                {
                  "Alias": "category_plaintext_100000",
                  "Async Capable": false,
                  "Node Type": "Seq Scan",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 100000,
                  "Plan Width": 8,
                  "Relation Name": "category_plaintext_100000",
                  "Startup Cost": 0.0,
                  "Total Cost": 1541.0
                }
              ],
              "Startup Cost": 2041.0,
              "Strategy": "Hashed",
              "Total Cost": 2043.5
            }
          ],
          "Sort Key": [
            "(count(*)) DESC"
          ],
          "Startup Cost": 2048.9,
          "Total Cost": 2049.53
        }
      ],
      "Startup Cost": 2048.9,
      "Total Cost": 2048.93
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Sort
    Aggregate (Sorted)
      Gather Merge
        Sort
          Aggregate (Hashed)
            Seq Scan on category_plaintext_1000000
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
      "Plan Width": 16,
      "Plans": [
        {
          "Async Capable": false,
          "Node Type": "Sort",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 250,
          "Plan Width": 16,
          "Plans": [
            {
              "Async Capable": false,
              "Group Key": [
                "value"
              ],
              "Node Type": "Aggregate",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Partial Mode": "Finalize",
              "Plan Rows": 250,
              "Plan Width": 16,
              "Plans": [
                {
                  "Async Capable": false,
                  "Node Type": "Gather Merge",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 500,
                  "Plan Width": 16,
                  "Plans": [
                    {
                      "Async Capable": false,
                      "Node Type": "Sort",
                      "Parallel Aware": false,
                      "Parent Relationship": "Outer",
                      "Plan Rows": 250,
                      "Plan Width": 16,
                      "Plans": [
                        {
                          "Async Capable": false,
                          "Group Key": [
                            "value"
                          ],
                          "Node Type": "Aggregate",
                          "Parallel Aware": false,
                          "Parent Relationship": "Outer",
                          "Partial Mode": "Partial",
                          "Plan Rows": 250,
                          "Plan Width": 16,
                          "Planned Partitions": 0,
                          "Plans": [
                            {
                              "Alias": "category_plaintext_1000000",
                              "Async Capable": false,
                              "Node Type": "Seq Scan",
                              "Parallel Aware": true,
                              "Parent Relationship": "Outer",
                              "Plan Rows": 416667,
                              "Plan Width": 8,
                              "Relation Name": "category_plaintext_1000000",
                              "Startup Cost": 0.0,
                              "Total Cost": 9572.67
                            }
                          ],
                          "Startup Cost": 11656.0,
                          "Strategy": "Hashed",
                          "Total Cost": 11658.5
                        }
                      ],
                      "Sort Key": [
                        "value"
                      ],
                      "Startup Cost": 11668.46,
                      "Total Cost": 11669.08
                    }
                  ],
                  "Startup Cost": 12668.48,
                  "Total Cost": 12726.82,
                  "Workers Planned": 2
                }
              ],
              "Startup Cost": 12668.48,
              "Strategy": "Sorted",
              "Total Cost": 12731.82
            }
          ],
          "Sort Key": [
            "(count(*)) DESC"
          ],
          "Startup Cost": 12737.22,
          "Total Cost": 12737.85
        }
      ],
      "Startup Cost": 12737.22,
      "Total Cost": 12737.25
    }
  }
]
```

</details>

![Query Performance - GROUP_BY/top_n_groups_plaintext](query_group_by_top_n_groups_plaintext_chart.png)

