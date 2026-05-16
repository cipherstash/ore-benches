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
| 10,000 | 0 | 428.29μs | 435.88μs |
| 100,000 | 0 | 450.88μs | 411.44μs |
| 1,000,000 | 0 | 414.56μs | 423.32μs |
| 10,000,000 | 0 | 410.21μs | 419.32μs |

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
          "Index Cond": "(((value).data ->> 'hm'::text) = 'b8140713f4ae49445a3220c3d13c28af7a08e31776368d943d1cba2860870712'::text)",
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
          "Index Cond": "(((value).data ->> 'hm'::text) = 'b8140713f4ae49445a3220c3d13c28af7a08e31776368d943d1cba2860870712'::text)",
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
          "Index Cond": "(((value).data ->> 'hm'::text) = 'b8140713f4ae49445a3220c3d13c28af7a08e31776368d943d1cba2860870712'::text)",
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
          "Index Cond": "(((value).data ->> 'hm'::text) = 'b8140713f4ae49445a3220c3d13c28af7a08e31776368d943d1cba2860870712'::text)",
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
| 10,000 | 0 | 403.48μs | 421.56μs |
| 100,000 | 0 | 404.76μs | 418.98μs |
| 1,000,000 | 0 | 407.13μs | 386.56μs |
| 10,000,000 | 0 | 396.38μs | 403.55μs |

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
          "Index Cond": "(((value).data ->> 'hm'::text) = 'b8140713f4ae49445a3220c3d13c28af7a08e31776368d943d1cba2860870712'::text)",
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
          "Index Cond": "(((value).data ->> 'hm'::text) = 'b8140713f4ae49445a3220c3d13c28af7a08e31776368d943d1cba2860870712'::text)",
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
          "Index Cond": "(((value).data ->> 'hm'::text) = 'b8140713f4ae49445a3220c3d13c28af7a08e31776368d943d1cba2860870712'::text)",
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
          "Index Cond": "(((value).data ->> 'hm'::text) = 'b8140713f4ae49445a3220c3d13c28af7a08e31776368d943d1cba2860870712'::text)",
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

