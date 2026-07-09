# PLAINTEXT Queries

[← Back to overview](./BENCHMARK_REPORT.md)

Per-tier query performance. Each scenario lists its SQL, the indexes available on the target table, the indexes the planner actually picked per tier, the timing table, and the full EXPLAIN plan in a collapsed block.

## exact_eq

**Description:** Unknown query

****

**Indexes used by the planner (per data set size):**

- 10,000: `string_plaintext_10000_value_idx`
- 100,000: `string_plaintext_100000_value_idx`
- 1,000,000: `string_plaintext_1000000_value_idx`
- 10,000,000: `string_plaintext_10000000_value_idx`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 1 | 93.70μs | N/A |
| 100,000 | 1 | 93.66μs | N/A |
| 1,000,000 | 1 | 94.84μs | N/A |
| 10,000,000 | 1 | 93.39μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using string_plaintext_10000_value_idx on string_plaintext_10000
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
      "Plan Width": 37,
      "Plans": [
        {
          "Alias": "string_plaintext_10000",
          "Async Capable": false,
          "Index Cond": "(value = '267f4e4740f4d3598ed11f9332de54a0'::text)",
          "Index Name": "string_plaintext_10000_value_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 37,
          "Relation Name": "string_plaintext_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.29,
          "Total Cost": 8.3
        }
      ],
      "Startup Cost": 0.29,
      "Total Cost": 8.3
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using string_plaintext_100000_value_idx on string_plaintext_100000
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
      "Plan Width": 37,
      "Plans": [
        {
          "Alias": "string_plaintext_100000",
          "Async Capable": false,
          "Index Cond": "(value = 'd76d68cf03e259209831f84f5cfa3bfc'::text)",
          "Index Name": "string_plaintext_100000_value_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 37,
          "Relation Name": "string_plaintext_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.42,
          "Total Cost": 8.44
        }
      ],
      "Startup Cost": 0.42,
      "Total Cost": 8.44
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using string_plaintext_1000000_value_idx on string_plaintext_1000000
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
      "Plan Width": 37,
      "Plans": [
        {
          "Alias": "string_plaintext_1000000",
          "Async Capable": false,
          "Index Cond": "(value = '77156d51418f4afaec005ee57a10ba1e'::text)",
          "Index Name": "string_plaintext_1000000_value_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 37,
          "Relation Name": "string_plaintext_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.42,
          "Total Cost": 8.44
        }
      ],
      "Startup Cost": 0.42,
      "Total Cost": 8.44
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using string_plaintext_10000000_value_idx on string_plaintext_10000000
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
      "Plan Width": 37,
      "Plans": [
        {
          "Alias": "string_plaintext_10000000",
          "Async Capable": false,
          "Index Cond": "(value = '311751f432da063c08c83d20cb7cd05b'::text)",
          "Index Name": "string_plaintext_10000000_value_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 37,
          "Relation Name": "string_plaintext_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.56,
          "Total Cost": 8.58
        }
      ],
      "Startup Cost": 0.56,
      "Total Cost": 8.58
    }
  }
]
```

</details>

![Query Performance - PLAINTEXT/exact_eq](query_plaintext_exact_eq_chart.png)

## json_contains

**Description:** Unknown query

****

**Indexes used by the planner (per data set size):**

- 10,000: `json_small_plaintext_10000_gin_idx`
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 110.15μs | N/A |
| 100,000 | 10 | 230.86μs | N/A |
| 1,000,000 | 10 | 286.21μs | N/A |
| 10,000,000 | 10 | 152.78μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Bitmap Heap Scan on json_small_plaintext_10000
    Bitmap Index Scan using json_small_plaintext_10000_gin_idx
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
          "Alias": "json_small_plaintext_10000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 101,
          "Plan Width": 4,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "(value @> '{\"age\": 97}'::jsonb)",
              "Index Name": "json_small_plaintext_10000_gin_idx",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 101,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 13.33
            }
          ],
          "Recheck Cond": "(value @> '{\"age\": 97}'::jsonb)",
          "Relation Name": "json_small_plaintext_10000",
          "Startup Cost": 13.36,
          "Total Cost": 213.28
        }
      ],
      "Startup Cost": 13.36,
      "Total Cost": 33.15
    }
  }
]
```

**100,000 rows**

```
Limit
  Seq Scan on json_small_plaintext_100000
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
          "Alias": "json_small_plaintext_100000",
          "Async Capable": false,
          "Filter": "(value @> '{\"age\": 63}'::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1010,
          "Plan Width": 4,
          "Relation Name": "json_small_plaintext_100000",
          "Startup Cost": 0.0,
          "Total Cost": 3953.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 39.14
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Seq Scan on json_small_plaintext_1000000
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
          "Alias": "json_small_plaintext_1000000",
          "Async Capable": false,
          "Filter": "(value @> '{\"age\": 74}'::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 10101,
          "Plan Width": 4,
          "Relation Name": "json_small_plaintext_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 39528.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 39.13
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Seq Scan on json_small_plaintext_10000000
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
          "Alias": "json_small_plaintext_10000000",
          "Async Capable": false,
          "Filter": "(value @> '{\"age\": 63}'::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 202020,
          "Plan Width": 4,
          "Relation Name": "json_small_plaintext_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 395271.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 19.57
    }
  }
]
```

</details>

![Query Performance - PLAINTEXT/json_contains](query_plaintext_json_contains_chart.png)

## json_field_eq

**Description:** Unknown query

****

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 218.80μs | N/A |
| 100,000 | 10 | 223.32μs | N/A |
| 1,000,000 | 10 | 288.18μs | N/A |
| 10,000,000 | 10 | 146.63μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Seq Scan on json_small_plaintext_10000
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
          "Alias": "json_small_plaintext_10000",
          "Async Capable": false,
          "Filter": "((value -> 'age'::text) = '97'::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50,
          "Plan Width": 4,
          "Relation Name": "json_small_plaintext_10000",
          "Startup Cost": 0.0,
          "Total Cost": 421.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 84.2
    }
  }
]
```

**100,000 rows**

```
Limit
  Seq Scan on json_small_plaintext_100000
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
          "Alias": "json_small_plaintext_100000",
          "Async Capable": false,
          "Filter": "((value -> 'age'::text) = '63'::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 500,
          "Plan Width": 4,
          "Relation Name": "json_small_plaintext_100000",
          "Startup Cost": 0.0,
          "Total Cost": 4203.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 84.06
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Seq Scan on json_small_plaintext_1000000
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
          "Alias": "json_small_plaintext_1000000",
          "Async Capable": false,
          "Filter": "((value -> 'age'::text) = '74'::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5000,
          "Plan Width": 4,
          "Relation Name": "json_small_plaintext_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 42028.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 84.06
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Seq Scan on json_small_plaintext_10000000
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
          "Alias": "json_small_plaintext_10000000",
          "Async Capable": false,
          "Filter": "((value -> 'age'::text) = '63'::jsonb)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50000,
          "Plan Width": 4,
          "Relation Name": "json_small_plaintext_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 420271.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 84.05
    }
  }
]
```

</details>

![Query Performance - PLAINTEXT/json_field_eq](query_plaintext_json_field_eq_chart.png)

## range_gt_10

**Description:** Unknown query

****

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 96.72μs | N/A |
| 100,000 | 10 | 87.60μs | N/A |
| 1,000,000 | 10 | 92.98μs | N/A |
| 10,000,000 | 10 | 89.40μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Seq Scan on integer_plaintext_10000
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
      "Plan Width": 8,
      "Plans": [
        {
          "Alias": "integer_plaintext_10000",
          "Async Capable": false,
          "Filter": "(value > 5000)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5019,
          "Plan Width": 8,
          "Relation Name": "integer_plaintext_10000",
          "Startup Cost": 0.0,
          "Total Cost": 170.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 0.34
    }
  }
]
```

**100,000 rows**

```
Limit
  Seq Scan on integer_plaintext_100000
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
      "Plan Width": 8,
      "Plans": [
        {
          "Alias": "integer_plaintext_100000",
          "Async Capable": false,
          "Filter": "(value > 5000)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49805,
          "Plan Width": 8,
          "Relation Name": "integer_plaintext_100000",
          "Startup Cost": 0.0,
          "Total Cost": 1693.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 0.34
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Seq Scan on integer_plaintext_1000000
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
      "Plan Width": 8,
      "Plans": [
        {
          "Alias": "integer_plaintext_1000000",
          "Async Capable": false,
          "Filter": "(value > 5000)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 500922,
          "Plan Width": 8,
          "Relation Name": "integer_plaintext_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 16925.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 0.34
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Seq Scan on integer_plaintext_10000000
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
      "Plan Width": 8,
      "Plans": [
        {
          "Alias": "integer_plaintext_10000000",
          "Async Capable": false,
          "Filter": "(value > 5000)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4944950,
          "Plan Width": 8,
          "Relation Name": "integer_plaintext_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 169248.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 0.34
    }
  }
]
```

</details>

![Query Performance - PLAINTEXT/range_gt_10](query_plaintext_range_gt_10_chart.png)

## range_lt_ordered_10

**Description:** Unknown query

****

**Indexes used by the planner (per data set size):**

- 10,000: `integer_plaintext_10000_value_idx`
- 100,000: `integer_plaintext_100000_value_idx`
- 1,000,000: `integer_plaintext_1000000_value_idx`
- 10,000,000: `integer_plaintext_10000000_value_idx`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 99.48μs | N/A |
| 100,000 | 10 | 102.40μs | N/A |
| 1,000,000 | 10 | 101.72μs | N/A |
| 10,000,000 | 10 | 97.27μs | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using integer_plaintext_10000_value_idx on integer_plaintext_10000
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
      "Plan Width": 8,
      "Plans": [
        {
          "Alias": "integer_plaintext_10000",
          "Async Capable": false,
          "Index Cond": "(value < 5000)",
          "Index Name": "integer_plaintext_10000_value_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4980,
          "Plan Width": 8,
          "Relation Name": "integer_plaintext_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.29,
          "Total Cost": 327.43
        }
      ],
      "Startup Cost": 0.29,
      "Total Cost": 0.94
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using integer_plaintext_100000_value_idx on integer_plaintext_100000
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
      "Plan Width": 8,
      "Plans": [
        {
          "Alias": "integer_plaintext_100000",
          "Async Capable": false,
          "Index Cond": "(value < 5000)",
          "Index Name": "integer_plaintext_100000_value_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50194,
          "Plan Width": 8,
          "Relation Name": "integer_plaintext_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.29,
          "Total Cost": 3206.69
        }
      ],
      "Startup Cost": 0.29,
      "Total Cost": 0.93
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using integer_plaintext_1000000_value_idx on integer_plaintext_1000000
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
      "Plan Width": 8,
      "Plans": [
        {
          "Alias": "integer_plaintext_1000000",
          "Async Capable": false,
          "Index Cond": "(value < 5000)",
          "Index Name": "integer_plaintext_1000000_value_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 499077,
          "Plan Width": 8,
          "Relation Name": "integer_plaintext_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.42,
          "Total Cost": 31914.21
        }
      ],
      "Startup Cost": 0.42,
      "Total Cost": 1.06
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using integer_plaintext_10000000_value_idx on integer_plaintext_10000000
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
      "Plan Width": 8,
      "Plans": [
        {
          "Alias": "integer_plaintext_10000000",
          "Async Capable": false,
          "Index Cond": "(value < 5000)",
          "Index Name": "integer_plaintext_10000000_value_idx",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5055049,
          "Plan Width": 8,
          "Relation Name": "integer_plaintext_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.43,
          "Total Cost": 320895.98
        }
      ],
      "Startup Cost": 0.43,
      "Total Cost": 1.07
    }
  }
]
```

</details>

![Query Performance - PLAINTEXT/range_lt_ordered_10](query_plaintext_range_lt_ordered_10_chart.png)

