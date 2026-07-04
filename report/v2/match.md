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
| 10,000 | 10 | 411.27μs | 27.04ms |
| 100,000 | 10 | 1.77ms | 26.41ms |
| 1,000,000 | 10 | 14.48ms | 39.91ms |
| 10,000,000 | 10 | ⚠️ 144.09ms | ⚠️ 169.68ms |

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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1109,1596,61,1500,587,582,865,1845,1760,792,1057,1018,1637,421,710,1496,1751,830,1068,1183,895,1574,1031,2028,998,1143,1453,1735,1200,682}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1109,1596,61,1500,587,582,865,1845,1760,792,1057,1018,1637,421,710,1496,1751,830,1068,1183,895,1574,1031,2028,998,1143,1453,1735,1200,682}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1637,1760,998,1845,1574,1453,582,1068,421,865,1143,1057,1751,1183,1031,710,1596,1018,830,587,1109,1200,682,895,792,1500,1496,1735,2028,61}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1637,1760,998,1845,1574,1453,582,1068,421,865,1143,1057,1751,1183,1031,710,1596,1018,830,587,1109,1200,682,895,792,1500,1496,1735,2028,61}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1031,587,1200,998,1574,792,1183,865,1596,1109,1057,895,1143,1751,682,582,1735,1496,1018,2028,421,830,1845,1068,61,1637,1760,1500,1453,710}'::smallint[])",
              "Index Name": "string_encrypted_1000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 1407.6
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1031,587,1200,998,1574,792,1183,865,1596,1109,1057,895,1143,1751,682,582,1735,1496,1018,2028,421,830,1845,1068,61,1637,1760,1500,1453,710}'::smallint[])",
          "Relation Name": "string_encrypted_1000000",
          "Startup Cost": 1407.6,
          "Total Cost": 1412.11
        }
      ],
      "Startup Cost": 1407.6,
      "Total Cost": 1412.11
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1453,421,61,2028,895,1596,1574,865,1845,792,587,1018,1735,998,1143,1496,1109,1031,1637,710,1200,1751,682,582,1760,1068,830,1183,1500,1057}'::smallint[])",
              "Index Name": "string_encrypted_10000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 11237.48
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1453,421,61,2028,895,1596,1574,865,1845,792,587,1018,1735,998,1143,1496,1109,1031,1637,710,1200,1751,682,582,1760,1068,830,1183,1500,1057}'::smallint[])",
          "Relation Name": "string_encrypted_10000000",
          "Startup Cost": 11237.48,
          "Total Cost": 11241.99
        }
      ],
      "Startup Cost": 11237.48,
      "Total Cost": 11241.99
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
| 10,000 | 2 | 152.35μs | 24.28ms |
| 100,000 | 10 | 634.91μs | 26.58ms |
| 1,000,000 | 10 | 3.77ms | 30.13ms |
| 10,000,000 | 10 | 33.69ms | 58.11ms |

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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1033,91,1393,453,1554,461}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1033,91,1393,453,1554,461}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{91,461,1393,1033,1554,453}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{91,461,1393,1033,1554,453}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{91,453,1033,1554,461,1393}'::smallint[])",
              "Index Name": "string_encrypted_1000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 282.35
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{91,453,1033,1554,461,1393}'::smallint[])",
          "Relation Name": "string_encrypted_1000000",
          "Startup Cost": 282.35,
          "Total Cost": 286.86
        }
      ],
      "Startup Cost": 282.35,
      "Total Cost": 286.86
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1033,91,1554,461,453,1393}'::smallint[])",
              "Index Name": "string_encrypted_10000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 2249.97
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1033,91,1554,461,453,1393}'::smallint[])",
          "Relation Name": "string_encrypted_10000000",
          "Startup Cost": 2249.97,
          "Total Cost": 2254.48
        }
      ],
      "Startup Cost": 2249.97,
      "Total Cost": 2254.48
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
| 10,000 | 10 | 437.17μs | 27.30ms |
| 100,000 | 10 | 1.79ms | 28.08ms |
| 1,000,000 | 10 | 14.40ms | 39.25ms |
| 10,000,000 | 10 | ⚠️ 144.81ms | ⚠️ 168.13ms |

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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1574,1109,1637,710,61,792,1760,1143,1057,1068,865,1453,1031,1751,587,1200,1596,895,1496,1845,1500,998,830,421,582,2028,1735,682,1183,1018}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1574,1109,1637,710,61,792,1760,1143,1057,1068,865,1453,1031,1751,587,1200,1596,895,1496,1845,1500,998,830,421,582,2028,1735,682,1183,1018}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{895,1637,1183,582,1109,1845,1496,1031,865,682,1500,1760,1068,1200,1574,830,1453,1018,1735,792,1143,998,587,1057,2028,1751,421,710,61,1596}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{895,1637,1183,582,1109,1845,1496,1031,865,682,1500,1760,1068,1200,1574,830,1453,1018,1735,792,1143,998,587,1057,2028,1751,421,710,61,1596}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{2028,1735,865,1845,1596,1637,1760,1031,792,682,1496,1751,1109,61,582,587,1500,1057,895,1143,998,1183,710,1574,1068,1018,1453,1200,421,830}'::smallint[])",
              "Index Name": "string_encrypted_1000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 1407.6
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{2028,1735,865,1845,1596,1637,1760,1031,792,682,1496,1751,1109,61,582,587,1500,1057,895,1143,998,1183,710,1574,1068,1018,1453,1200,421,830}'::smallint[])",
          "Relation Name": "string_encrypted_1000000",
          "Startup Cost": 1407.6,
          "Total Cost": 1412.11
        }
      ],
      "Startup Cost": 1407.6,
      "Total Cost": 1412.11
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{830,1500,1143,1031,1574,1109,1068,1453,1845,895,1057,1496,1751,582,61,587,1183,998,2028,1735,1018,792,1760,710,865,1200,1637,421,682,1596}'::smallint[])",
              "Index Name": "string_encrypted_10000000_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 11237.48
            }
          ],
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{830,1500,1143,1031,1574,1109,1068,1453,1845,895,1057,1496,1751,582,61,587,1183,998,2028,1735,1018,792,1760,710,865,1200,1637,421,682,1596}'::smallint[])",
          "Relation Name": "string_encrypted_10000000",
          "Startup Cost": 11237.48,
          "Total Cost": 11241.99
        }
      ],
      "Startup Cost": 11237.48,
      "Total Cost": 11241.99
    }
  }
]
```

</details>

![Query Performance - MATCH/eql_cast_lastname](query_match_eql_cast_lastname_chart.png)

