# MATCH Queries

[← Back to overview](./BENCHMARK_REPORT.md)

Per-tier query performance. Each scenario lists its SQL, the indexes available on the target table, the indexes the planner actually picked per tier, the timing table, and the full EXPLAIN plan in a collapsed block.

## eql_bloom

**Description:** Pattern matching using EQL bloom filter containment

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE eql_v2.bloom_filter(value) @> eql_v2.bloom_filter($1) LIMIT 10
```

**Parameter:** `Johnson`

**Table: `string_encrypted_{rows}` with encrypted string values. Index: Bloom filter index using `eql_v2.bloom_filter`. Query returns LIMIT 10 results.**

**Indexes available on the table:**
```sql
CREATE INDEX
string_encrypted_10000_hash_index
ON string_encrypted_10000 using hash (
    eql_v2.hmac_256(value)
);

CREATE INDEX
string_encrypted_10000_gin_index
ON string_encrypted_10000 USING GIN (
    eql_v2.bloom_filter(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `string_encrypted_10000_gin_index`
- 100,000: `string_encrypted_100000_gin_index`
- 1,000,000: `string_encrypted_1000000_gin_index`
- 10,000,000: `string_encrypted_10000000_gin_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 0 | 590.02μs | 627.32μs |
| 100,000 | 0 | 1.42ms | 1.48ms |
| 1,000,000 | 0 | 9.36ms | 9.19ms |
| 10,000,000 | 0 | 83.30ms | 82.09ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_10000
    Bitmap Index Scan using string_encrypted_10000_gin_index
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
          "Alias": "string_encrypted_10000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1967,1061,1185,97,242,237,702,820,1987,1659,1098,2024,1665,44,523,1715,461,1196,450,835,765,543,1846,1988,518,290,1381,574,1609,513}'::smallint[])",
              "Index Name": "string_encrypted_10000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 264.6
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1967,1061,1185,97,242,237,702,820,1987,1659,1098,2024,1665,44,523,1715,461,1196,450,835,765,543,1846,1988,518,290,1381,574,1609,513}'::smallint[])",
          "Relation Name": "string_encrypted_10000",
          "Startup Cost": 264.6,
          "Total Cost": 269.11
        }
      ],
      "Startup Cost": 264.6,
      "Total Cost": 269.11
    }
  }
]
```

**100,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_100000
    Bitmap Index Scan using string_encrypted_100000_gin_index
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
          "Alias": "string_encrypted_100000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{44,1098,1381,290,237,461,1609,242,1665,1715,1967,1988,2024,1196,1846,1659,1061,1987,450,1185,97,820,523,765,543,513,702,518,574,835}'::smallint[])",
              "Index Name": "string_encrypted_100000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 454.35
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{44,1098,1381,290,237,461,1609,242,1665,1715,1967,1988,2024,1196,1846,1659,1061,1987,450,1185,97,820,523,765,543,513,702,518,574,835}'::smallint[])",
          "Relation Name": "string_encrypted_100000",
          "Startup Cost": 454.35,
          "Total Cost": 458.86
        }
      ],
      "Startup Cost": 454.35,
      "Total Cost": 458.86
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_1000000
    Bitmap Index Scan using string_encrypted_1000000_gin_index
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
          "Alias": "string_encrypted_1000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{523,1381,1846,450,1185,765,1609,574,543,1665,702,513,1659,1715,1988,1098,835,97,242,290,461,237,44,820,1987,2024,1061,1196,1967,518}'::smallint[])",
              "Index Name": "string_encrypted_1000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 1530.98
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{523,1381,1846,450,1185,765,1609,574,543,1665,702,513,1659,1715,1988,1098,835,97,242,290,461,237,44,820,1987,2024,1061,1196,1967,518}'::smallint[])",
          "Relation Name": "string_encrypted_1000000",
          "Startup Cost": 1530.98,
          "Total Cost": 1535.49
        }
      ],
      "Startup Cost": 1530.98,
      "Total Cost": 1535.49
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_10000000
    Bitmap Index Scan using string_encrypted_10000000_gin_index
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
          "Alias": "string_encrypted_10000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{820,237,1987,1846,1196,44,1659,1098,1381,513,835,1665,1715,2024,461,518,765,1967,702,1988,290,574,543,450,97,1061,1185,242,1609,523}'::smallint[])",
              "Index Name": "string_encrypted_10000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 11294.85
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{820,237,1987,1846,1196,44,1659,1098,1381,513,835,1665,1715,2024,461,518,765,1967,702,1988,290,574,543,450,97,1061,1185,242,1609,523}'::smallint[])",
          "Relation Name": "string_encrypted_10000000",
          "Startup Cost": 11294.85,
          "Total Cost": 11299.36
        }
      ],
      "Startup Cost": 11294.85,
      "Total Cost": 11299.36
    }
  }
]
```

</details>

![Query Performance - MATCH/eql_bloom](query_match_eql_bloom_chart.png)

## eql_cast_firstname

**Description:** Pattern matching on first name using EQL cast and LIKE

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value LIKE $1 LIMIT 10
```

**Parameter:** `Bob`

**Table: `string_encrypted_{rows}` with encrypted string values. Index: MATCH index for substring searches. Query returns LIMIT 10 results.**

**Indexes available on the table:**
```sql
CREATE INDEX
string_encrypted_10000_hash_index
ON string_encrypted_10000 using hash (
    eql_v2.hmac_256(value)
);

CREATE INDEX
string_encrypted_10000_gin_index
ON string_encrypted_10000 USING GIN (
    eql_v2.bloom_filter(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `string_encrypted_10000_gin_index`
- 100,000: `string_encrypted_100000_gin_index`
- 1,000,000: `string_encrypted_1000000_gin_index`
- 10,000,000: `string_encrypted_10000000_gin_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 0 | 537.11μs | 476.72μs |
| 100,000 | 0 | 1.02ms | 835.98μs |
| 1,000,000 | 0 | 4.32ms | 4.27ms |
| 10,000,000 | 0 | 30.20ms | 30.69ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_10000
    Bitmap Index Scan using string_encrypted_10000_gin_index
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
          "Alias": "string_encrypted_10000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1789,1164,1603,1555,36,10}'::smallint[])",
              "Index Name": "string_encrypted_10000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 56.22
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1789,1164,1603,1555,36,10}'::smallint[])",
          "Relation Name": "string_encrypted_10000",
          "Startup Cost": 56.22,
          "Total Cost": 60.73
        }
      ],
      "Startup Cost": 56.22,
      "Total Cost": 60.73
    }
  }
]
```

**100,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_100000
    Bitmap Index Scan using string_encrypted_100000_gin_index
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
          "Alias": "string_encrypted_100000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1603,10,1789,1555,1164,36}'::smallint[])",
              "Index Name": "string_encrypted_100000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 93.35
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1603,10,1789,1555,1164,36}'::smallint[])",
          "Relation Name": "string_encrypted_100000",
          "Startup Cost": 93.35,
          "Total Cost": 97.86
        }
      ],
      "Startup Cost": 93.35,
      "Total Cost": 97.86
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_1000000
    Bitmap Index Scan using string_encrypted_1000000_gin_index
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
          "Alias": "string_encrypted_1000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1164,1789,1603,36,1555,10}'::smallint[])",
              "Index Name": "string_encrypted_1000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 307.85
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1164,1789,1603,36,1555,10}'::smallint[])",
          "Relation Name": "string_encrypted_1000000",
          "Startup Cost": 307.85,
          "Total Cost": 312.36
        }
      ],
      "Startup Cost": 307.85,
      "Total Cost": 312.36
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_10000000
    Bitmap Index Scan using string_encrypted_10000000_gin_index
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
          "Alias": "string_encrypted_10000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1789,1555,1603,36,10,1164}'::smallint[])",
              "Index Name": "string_encrypted_10000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 2258.97
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1789,1555,1603,36,10,1164}'::smallint[])",
          "Relation Name": "string_encrypted_10000000",
          "Startup Cost": 2258.97,
          "Total Cost": 2263.48
        }
      ],
      "Startup Cost": 2258.97,
      "Total Cost": 2263.48
    }
  }
]
```

</details>

![Query Performance - MATCH/eql_cast_firstname](query_match_eql_cast_firstname_chart.png)

## eql_cast_lastname

**Description:** Pattern matching on last name using EQL cast and LIKE

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value LIKE $1 LIMIT 10
```

**Parameter:** `Johnson`

**Table: `string_encrypted_{rows}` with encrypted string values. Index: MATCH index for substring searches. Query returns LIMIT 10 results.**

**Indexes available on the table:**
```sql
CREATE INDEX
string_encrypted_10000_hash_index
ON string_encrypted_10000 using hash (
    eql_v2.hmac_256(value)
);

CREATE INDEX
string_encrypted_10000_gin_index
ON string_encrypted_10000 USING GIN (
    eql_v2.bloom_filter(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `string_encrypted_10000_gin_index`
- 100,000: `string_encrypted_100000_gin_index`
- 1,000,000: `string_encrypted_1000000_gin_index`
- 10,000,000: `string_encrypted_10000000_gin_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 0 | 697.29μs | 653.88μs |
| 100,000 | 0 | 1.58ms | 1.54ms |
| 1,000,000 | 0 | 9.94ms | 9.68ms |
| 10,000,000 | 0 | 82.38ms | 82.15ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_10000
    Bitmap Index Scan using string_encrypted_10000_gin_index
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
          "Alias": "string_encrypted_10000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{290,237,523,2024,1098,835,1967,1665,1196,242,461,518,574,513,1987,1846,702,543,1381,1061,1609,44,1988,820,1185,765,450,97,1659,1715}'::smallint[])",
              "Index Name": "string_encrypted_10000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 264.6
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{290,237,523,2024,1098,835,1967,1665,1196,242,461,518,574,513,1987,1846,702,543,1381,1061,1609,44,1988,820,1185,765,450,97,1659,1715}'::smallint[])",
          "Relation Name": "string_encrypted_10000",
          "Startup Cost": 264.6,
          "Total Cost": 269.11
        }
      ],
      "Startup Cost": 264.6,
      "Total Cost": 269.11
    }
  }
]
```

**100,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_100000
    Bitmap Index Scan using string_encrypted_100000_gin_index
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
          "Alias": "string_encrypted_100000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{513,1967,574,523,97,1987,820,242,765,290,1098,1196,450,543,835,1988,44,1846,518,237,1659,1609,1381,1061,1665,1185,702,2024,1715,461}'::smallint[])",
              "Index Name": "string_encrypted_100000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 454.35
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{513,1967,574,523,97,1987,820,242,765,290,1098,1196,450,543,835,1988,44,1846,518,237,1659,1609,1381,1061,1665,1185,702,2024,1715,461}'::smallint[])",
          "Relation Name": "string_encrypted_100000",
          "Startup Cost": 454.35,
          "Total Cost": 458.86
        }
      ],
      "Startup Cost": 454.35,
      "Total Cost": 458.86
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_1000000
    Bitmap Index Scan using string_encrypted_1000000_gin_index
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
          "Alias": "string_encrypted_1000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{290,237,702,461,543,1098,44,523,1665,518,450,2024,1987,242,1609,1185,1967,1196,1381,835,765,1846,1659,1061,820,1988,574,513,97,1715}'::smallint[])",
              "Index Name": "string_encrypted_1000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 1530.98
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{290,237,702,461,543,1098,44,523,1665,518,450,2024,1987,242,1609,1185,1967,1196,1381,835,765,1846,1659,1061,820,1988,574,513,97,1715}'::smallint[])",
          "Relation Name": "string_encrypted_1000000",
          "Startup Cost": 1530.98,
          "Total Cost": 1535.49
        }
      ],
      "Startup Cost": 1530.98,
      "Total Cost": 1535.49
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_10000000
    Bitmap Index Scan using string_encrypted_10000000_gin_index
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
          "Alias": "string_encrypted_10000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 36,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{702,1381,835,97,765,450,1988,44,1061,523,242,461,1609,1846,1196,237,1715,2024,290,1659,543,1987,820,513,574,1098,1967,1185,1665,518}'::smallint[])",
              "Index Name": "string_encrypted_10000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 11294.85
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{702,1381,835,97,765,450,1988,44,1061,523,242,461,1609,1846,1196,237,1715,2024,290,1659,543,1987,820,513,574,1098,1967,1185,1665,518}'::smallint[])",
          "Relation Name": "string_encrypted_10000000",
          "Startup Cost": 11294.85,
          "Total Cost": 11299.36
        }
      ],
      "Startup Cost": 11294.85,
      "Total Cost": 11299.36
    }
  }
]
```

</details>

![Query Performance - MATCH/eql_cast_lastname](query_match_eql_cast_lastname_chart.png)

