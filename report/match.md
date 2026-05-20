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

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 1.02ms | 28.36ms |
| 100,000 | 10 | 2.50ms | 27.24ms |
| 1,000,000 | 10 | 18.21ms | 41.76ms |
| 10,000,000 | 10 | ⚠️ 215.72ms | ⚠️ 237.82ms |

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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1196,1659,2024,543,1967,518,1381,242,1185,450,290,835,820,1987,765,1846,237,461,44,1609,523,1715,1988,1098,702,97,574,1665,1061,513}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1196,1659,2024,543,1967,518,1381,242,1185,450,290,835,820,1987,765,1846,237,461,44,1609,523,1715,1988,1098,702,97,574,1665,1061,513}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{582,1751,1735,2028,830,792,1057,1574,1453,998,1760,61,1183,1068,1500,895,1031,1109,587,1845,682,1200,1018,1143,421,1496,1596,865,710,1637}'::smallint[])",
              "Index Name": "string_encrypted_100000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 458.47
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{582,1751,1735,2028,830,792,1057,1574,1453,998,1760,61,1183,1068,1500,895,1031,1109,587,1845,682,1200,1018,1143,421,1496,1596,865,710,1637}'::smallint[])",
          "Relation Name": "string_encrypted_100000",
          "Startup Cost": 458.48,
          "Total Cost": 462.99
        }
      ],
      "Startup Cost": 458.48,
      "Total Cost": 462.99
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1574,421,1068,1496,1143,1183,1751,2028,1018,792,587,710,61,682,582,1596,1057,830,1845,865,1637,1735,1453,1760,895,1500,1031,1200,1109,998}'::smallint[])",
              "Index Name": "string_encrypted_1000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 1526.85
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1574,421,1068,1496,1143,1183,1751,2028,1018,792,587,710,61,682,582,1596,1057,830,1845,865,1637,1735,1453,1760,895,1500,1031,1200,1109,998}'::smallint[])",
          "Relation Name": "string_encrypted_1000000",
          "Startup Cost": 1526.85,
          "Total Cost": 1531.36
        }
      ],
      "Startup Cost": 1526.85,
      "Total Cost": 1531.36
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{574,702,1846,237,1659,44,242,1967,1196,518,290,1381,1665,450,1609,835,2024,513,1098,765,820,1715,1185,1987,97,1061,1988,523,543,461}'::smallint[])",
              "Index Name": "string_encrypted_10000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 11270.1
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{574,702,1846,237,1659,44,242,1967,1196,518,290,1381,1665,450,1609,835,2024,513,1098,765,820,1715,1185,1987,97,1061,1988,523,543,461}'::smallint[])",
          "Relation Name": "string_encrypted_10000000",
          "Startup Cost": 11270.1,
          "Total Cost": 11274.61
        }
      ],
      "Startup Cost": 11270.1,
      "Total Cost": 11274.61
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
| 10,000 | 10 | 729.02μs | 28.41ms |
| 100,000 | 10 | 1.13ms | 25.54ms |
| 1,000,000 | 10 | 6.31ms | 29.28ms |
| 10,000,000 | 10 | 35.48ms | 60.96ms |

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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{36,1603,1164,10,1555,1789}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{36,1603,1164,10,1555,1789}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{91,1033,461,1554,1393,453}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{91,1033,461,1554,1393,453}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1033,461,453,91,1554,1393}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1033,461,453,91,1554,1393}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{36,1789,1603,1555,10,1164}'::smallint[])",
              "Index Name": "string_encrypted_10000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 2254.85
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{36,1789,1603,1555,10,1164}'::smallint[])",
          "Relation Name": "string_encrypted_10000000",
          "Startup Cost": 2254.85,
          "Total Cost": 2259.36
        }
      ],
      "Startup Cost": 2254.85,
      "Total Cost": 2259.36
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

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 1.55ms | 30.64ms |
| 100,000 | 10 | 2.64ms | 26.92ms |
| 1,000,000 | 10 | 18.83ms | 42.66ms |
| 10,000,000 | 10 | ⚠️ 210.49ms | ⚠️ 241.26ms |

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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1381,1185,1609,835,1715,513,1967,543,461,1098,237,523,1987,820,1196,1846,1665,242,765,290,1061,702,44,1659,97,574,518,2024,450,1988}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1381,1185,1609,835,1715,513,1967,543,461,1098,237,523,1987,820,1196,1846,1665,242,765,290,1061,702,44,1659,97,574,518,2024,450,1988}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1453,1183,830,1496,895,1500,1574,61,1143,1845,1596,1735,865,421,1031,587,1057,582,682,792,1760,1751,1068,1200,1637,710,2028,1018,998,1109}'::smallint[])",
              "Index Name": "string_encrypted_100000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 458.47
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1453,1183,830,1496,895,1500,1574,61,1143,1845,1596,1735,865,421,1031,587,1057,582,682,792,1760,1751,1068,1200,1637,710,2028,1018,998,1109}'::smallint[])",
          "Relation Name": "string_encrypted_100000",
          "Startup Cost": 458.48,
          "Total Cost": 462.99
        }
      ],
      "Startup Cost": 458.48,
      "Total Cost": 462.99
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1018,1143,1574,1500,1453,61,1200,587,1735,895,1057,865,1183,1760,682,1068,1109,1637,2028,710,1845,1496,1031,998,582,830,1751,792,421,1596}'::smallint[])",
              "Index Name": "string_encrypted_1000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 1526.85
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1018,1143,1574,1500,1453,61,1200,587,1735,895,1057,865,1183,1760,682,1068,1109,1637,2028,710,1845,1496,1031,998,582,830,1751,792,421,1596}'::smallint[])",
          "Relation Name": "string_encrypted_1000000",
          "Startup Cost": 1526.85,
          "Total Cost": 1531.36
        }
      ],
      "Startup Cost": 1526.85,
      "Total Cost": 1531.36
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{461,450,513,765,1846,2024,1665,543,97,242,1609,835,1061,290,1987,1381,1715,237,1967,574,1185,44,523,702,1196,1988,1098,820,518,1659}'::smallint[])",
              "Index Name": "string_encrypted_10000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 11270.1
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{461,450,513,765,1846,2024,1665,543,97,242,1609,835,1061,290,1987,1381,1715,237,1967,574,1185,44,523,702,1196,1988,1098,820,518,1659}'::smallint[])",
          "Relation Name": "string_encrypted_10000000",
          "Startup Cost": 11270.1,
          "Total Cost": 11274.61
        }
      ],
      "Startup Cost": 11270.1,
      "Total Cost": 11274.61
    }
  }
]
```

</details>

![Query Performance - MATCH/eql_cast_lastname](query_match_eql_cast_lastname_chart.png)

