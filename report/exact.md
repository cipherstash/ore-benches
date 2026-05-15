# EXACT Queries

[← Back to overview](./BENCHMARK_REPORT.md)

Per-tier query performance. Each scenario lists its SQL, the indexes available on the target table, the indexes the planner actually picked per tier, the timing table, and the full EXPLAIN plan in a collapsed block.

## eql_cast

**Description:** Exact match using EQL cast operator

**SQL Query:**
```sql
SELECT value FROM {TABLE} WHERE value = $1 LIMIT 1
```

**Parameter:** `Bob Johnson`

**Table: `string_encrypted_{rows}` with encrypted string values. Index: UNIQUE index on the encrypted value column.**

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

- 10,000: `string_encrypted_10000_hash_index`
- 100,000: `string_encrypted_100000_hash_index`
- 1,000,000: `string_encrypted_1000000_hash_index`
- 10,000,000: `string_encrypted_10000000_hash_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 0 | 417.91μs | 411.64μs |
| 100,000 | 0 | 398.18μs | 415.60μs |
| 1,000,000 | 0 | 518.60μs | 413.16μs |
| 10,000,000 | 0 | 429.34μs | 423.97μs |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using string_encrypted_10000_hash_index on string_encrypted_10000
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
      "Plan Width": 1164,
      "Plans": [
        {
          "Alias": "string_encrypted_10000",
          "Async Capable": false,
          "Index Cond": "(((value).data ->> 'hm'::text) = '9b50b28ba1880a29710f2713a7afa5fb67d3f44d65357495b1f65bf5bdabb02e'::text)",
          "Index Name": "string_encrypted_10000_hash_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 1164,
          "Relation Name": "string_encrypted_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.0,
          "Total Cost": 8.02
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 8.02
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using string_encrypted_100000_hash_index on string_encrypted_100000
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
      "Plan Width": 1163,
      "Plans": [
        {
          "Alias": "string_encrypted_100000",
          "Async Capable": false,
          "Index Cond": "(((value).data ->> 'hm'::text) = '9b50b28ba1880a29710f2713a7afa5fb67d3f44d65357495b1f65bf5bdabb02e'::text)",
          "Index Name": "string_encrypted_100000_hash_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 1163,
          "Relation Name": "string_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.0,
          "Total Cost": 8.02
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 8.02
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using string_encrypted_1000000_hash_index on string_encrypted_1000000
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
      "Plan Width": 1161,
      "Plans": [
        {
          "Alias": "string_encrypted_1000000",
          "Async Capable": false,
          "Index Cond": "(((value).data ->> 'hm'::text) = '9b50b28ba1880a29710f2713a7afa5fb67d3f44d65357495b1f65bf5bdabb02e'::text)",
          "Index Name": "string_encrypted_1000000_hash_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 2,
          "Plan Width": 1161,
          "Relation Name": "string_encrypted_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.0,
          "Total Cost": 12.04
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 6.02
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using string_encrypted_10000000_hash_index on string_encrypted_10000000
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
      "Plan Width": 1163,
      "Plans": [
        {
          "Alias": "string_encrypted_10000000",
          "Async Capable": false,
          "Index Cond": "(((value).data ->> 'hm'::text) = '9b50b28ba1880a29710f2713a7afa5fb67d3f44d65357495b1f65bf5bdabb02e'::text)",
          "Index Name": "string_encrypted_10000000_hash_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50000,
          "Plan Width": 1163,
          "Relation Name": "string_encrypted_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.0,
          "Total Cost": 198371.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 3.97
    }
  }
]
```

</details>

![Query Performance - EXACT/eql_cast](query_exact_eql_cast_chart.png)

## eql_hash

**Description:** Exact match using EQL HMAC-256 hash function

**SQL Query:**
```sql
SELECT value FROM {TABLE} WHERE eql_v2.hmac_256(value) = eql_v2.hmac_256($1::jsonb) LIMIT 1
```

**Parameter:** `Bob Johnson`

**Table: `string_encrypted_{rows}` with encrypted string values. Index: Hash-based unique index using `eql_v2.hmac_256`.**

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

- 10,000: `string_encrypted_10000_hash_index`
- 100,000: `string_encrypted_100000_hash_index`
- 1,000,000: `string_encrypted_1000000_hash_index`
- 10,000,000: `string_encrypted_10000000_hash_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 0 | 412.70μs | 395.84μs |
| 100,000 | 0 | 406.77μs | 400.13μs |
| 1,000,000 | 0 | 401.58μs | 397.75μs |
| 10,000,000 | 0 | 387.61μs | 399.45μs |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using string_encrypted_10000_hash_index on string_encrypted_10000
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
      "Plan Width": 1164,
      "Plans": [
        {
          "Alias": "string_encrypted_10000",
          "Async Capable": false,
          "Index Cond": "(((value).data ->> 'hm'::text) = '9b50b28ba1880a29710f2713a7afa5fb67d3f44d65357495b1f65bf5bdabb02e'::text)",
          "Index Name": "string_encrypted_10000_hash_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 1164,
          "Relation Name": "string_encrypted_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.0,
          "Total Cost": 8.02
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 8.02
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using string_encrypted_100000_hash_index on string_encrypted_100000
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
      "Plan Width": 1163,
      "Plans": [
        {
          "Alias": "string_encrypted_100000",
          "Async Capable": false,
          "Index Cond": "(((value).data ->> 'hm'::text) = '9b50b28ba1880a29710f2713a7afa5fb67d3f44d65357495b1f65bf5bdabb02e'::text)",
          "Index Name": "string_encrypted_100000_hash_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 1163,
          "Relation Name": "string_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.0,
          "Total Cost": 8.02
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 8.02
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using string_encrypted_1000000_hash_index on string_encrypted_1000000
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
      "Plan Width": 1161,
      "Plans": [
        {
          "Alias": "string_encrypted_1000000",
          "Async Capable": false,
          "Index Cond": "(((value).data ->> 'hm'::text) = '9b50b28ba1880a29710f2713a7afa5fb67d3f44d65357495b1f65bf5bdabb02e'::text)",
          "Index Name": "string_encrypted_1000000_hash_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 2,
          "Plan Width": 1161,
          "Relation Name": "string_encrypted_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.0,
          "Total Cost": 12.04
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 6.02
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using string_encrypted_10000000_hash_index on string_encrypted_10000000
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
      "Plan Width": 1163,
      "Plans": [
        {
          "Alias": "string_encrypted_10000000",
          "Async Capable": false,
          "Index Cond": "(((value).data ->> 'hm'::text) = '9b50b28ba1880a29710f2713a7afa5fb67d3f44d65357495b1f65bf5bdabb02e'::text)",
          "Index Name": "string_encrypted_10000000_hash_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50000,
          "Plan Width": 1163,
          "Relation Name": "string_encrypted_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.0,
          "Total Cost": 198371.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 3.97
    }
  }
]
```

</details>

![Query Performance - EXACT/eql_hash](query_exact_eql_hash_chart.png)

