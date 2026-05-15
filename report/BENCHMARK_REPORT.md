# Benchmark Report

This report summarizes the performance benchmarks for encrypted database operations.

## Table of Contents

1. [Ingest Throughput](#ingest-throughput)
   - [Int](#int)
   - [Json Small](#json-small)
   - [Ste Vec Small](#ste-vec-small)
   - [String](#string)
2. [Query Performance](#query-performance)
   - [EXACT Queries](#exact-queries)
   - [GROUP_BY Queries](#group_by-queries)
   - [MATCH Queries](#match-queries)
   - [ORE Queries](#ore-queries)

---

## Ingest Throughput

This section measures the throughput of inserting encrypted records into the database.

### Comparison at 10,000 Records

Comparing all benchmark types at 10,000 records.

![Throughput Comparison at 10,000 records](ingest_comparison_throughput_10000.png)

![Total Time Comparison at 10,000 records](ingest_comparison_time_10000.png)

![Total Time Comparison at 10,000 records (excluding ste_vec_large)](ingest_comparison_time_10000_filtered.png)

### Int

Tests insertion of encrypted integer values.

| Records | Throughput (records/sec) | Total Time | Avg Memory |
|---------|--------------------------|------------|------------|
| 500 | 544.83 | 0.92s | 15.25 MB |
| 1,000 | 1.11K | 0.90s | 17.83 MB |
| 10,000 | 1.34K | 7.48s | 20.34 MB |

![Ingest Throughput - int](ingest_int_throughput_chart.png)

![Ingest Total Time - int](ingest_int_time_chart.png)

### Json Small

Tests insertion of small encrypted JSON objects (first_name, last_name, age, email).

| Records | Throughput (records/sec) | Total Time | Avg Memory |
|---------|--------------------------|------------|------------|
| 500 | 565.55 | 0.88s | 18.70 MB |
| 1,000 | 1.45K | 0.69s | 27.47 MB |
| 10,000 | 2.22K | 4.51s | 45.33 MB |

![Ingest Throughput - json_small](ingest_json_small_throughput_chart.png)

![Ingest Total Time - json_small](ingest_json_small_time_chart.png)

### Ste Vec Small

Tests insertion of small JSON objects with SteVec (searchable encrypted vector) indexing.

| Records | Throughput (records/sec) | Total Time | Avg Memory |
|---------|--------------------------|------------|------------|
| 10,000 | 4.03K | 2.48s | 31.90 MB |

![Ingest Throughput - ste_vec_small](ingest_ste_vec_small_throughput_chart.png)

![Ingest Total Time - ste_vec_small](ingest_ste_vec_small_time_chart.png)

### String

Tests insertion of encrypted string values.

| Records | Throughput (records/sec) | Total Time | Avg Memory |
|---------|--------------------------|------------|------------|
| 500 | 559.65 | 0.89s | 14.12 MB |
| 1,000 | 1.86K | 0.54s | 16.19 MB |
| 10,000 | 2.83K | 3.54s | 18.23 MB |

![Ingest Throughput - string](ingest_string_throughput_chart.png)

![Ingest Total Time - string](ingest_string_time_chart.png)

## Query Performance

This section measures query performance across different data set sizes. Each query is tested with and without decryption of results.

### EXACT Queries

#### eql_cast

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

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | 1 | 425.53μs | 425.41μs |
| 100,000 | 1 | 420.15μs | 393.39μs |
| 1,000,000 | 1 | 435.19μs | 412.73μs |
| 10,000,000 | 1 | 416.49μs | 425.57μs |

_Rows (est.) is the planner's estimate from `EXPLAIN` captured before the bench loop. For LIMIT-bounded queries it matches the LIMIT; for aggregates it's the estimated group count._

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

#### eql_hash

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

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | 1 | 397.38μs | 389.09μs |
| 100,000 | 1 | 391.48μs | 410.67μs |
| 1,000,000 | 1 | 396.06μs | 406.56μs |
| 10,000,000 | 1 | 396.14μs | 401.25μs |

_Rows (est.) is the planner's estimate from `EXPLAIN` captured before the bench loop. For LIMIT-bounded queries it matches the LIMIT; for aggregates it's the estimated group count._

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

### GROUP_BY Queries

#### count_groups_encrypted

**Description:** GROUP BY in extractor form on `eql_v2.hmac_256(value)`, wrapped in `count(*)` to isolate aggregation cost from emit cost

**SQL Query:**
```sql
SELECT count(*) FROM (SELECT 1 FROM {TABLE} GROUP BY eql_v2.hmac_256(value)) g
```

**Table: `string_encrypted_{rows}` with encrypted string values (carrying an `hm` HMAC term, configured via the `unique` search index). Index: no index drives `GROUP BY` directly — hash aggregation is in-memory. The extractor's 32-byte HMAC group key fits in default `work_mem`, so the planner picks `HashAggregate` reliably across deployments. **Why the subquery wrapper.** The bench data is `fake::name::Name<EN>` — effectively unique per row, so a bare `SELECT count(*) FROM tbl GROUP BY eql_v2.hmac_256(value)` emits ~one row per input row. Wall-clock time on that shape is dominated by result emission (server-side row construction, network round-trip, sqlx deserialisation, bench iter-and-sum), not by the aggregation work the recipe is actually about. Wrapping the GROUP BY in `count(*)` keeps the inner HashAggregate identical but emits a single row, so the bench measures aggregation cost. The companion `count_groups_plaintext` scenario runs the same query shape against an unencrypted column for comparison. Natural-form `GROUP BY value` against an encrypted column was removed from this bench in an earlier pass because the planner picks `GroupAggregate` + sort against the full ~1-2 KB ciphertext payload at scale — see §5 of the EQL query-performance guide.**

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

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | 1 | 4.55ms | N/A |
| 100,000 | 1 | 67.49ms | N/A |
| 1,000,000 | 1 | ⚠️ 792.92ms | N/A |
| 10,000,000 | 1 | ⚠️ 11.479s | N/A |

_Rows (est.) is the planner's estimate from `EXPLAIN` captured before the bench loop. For LIMIT-bounded queries it matches the LIMIT; for aggregates it's the estimated group count._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Aggregate
  Aggregate (Hashed)
    Seq Scan on string_encrypted_10000
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
            "(((string_encrypted_10000.value).data ->> 'hm'::text))::eql_v2.hmac_256"
          ],
          "Node Type": "Aggregate",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Partial Mode": "Simple",
          "Plan Rows": 9968,
          "Plan Width": 36,
          "Planned Partitions": 0,
          "Plans": [
            {
              "Alias": "string_encrypted_10000",
              "Async Capable": false,
              "Node Type": "Seq Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 10000,
              "Plan Width": 32,
              "Relation Name": "string_encrypted_10000",
              "Startup Cost": 0.0,
              "Total Cost": 1678.0
            }
          ],
          "Startup Cost": 1703.0,
          "Strategy": "Hashed",
          "Total Cost": 1827.6
        }
      ],
      "Startup Cost": 1952.2,
      "Strategy": "Plain",
      "Total Cost": 1952.21
    }
  }
]
```

**100,000 rows**

```
Aggregate
  Aggregate (Hashed)
    Seq Scan on string_encrypted_100000
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
            "(((string_encrypted_100000.value).data ->> 'hm'::text))::eql_v2.hmac_256"
          ],
          "Node Type": "Aggregate",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Partial Mode": "Simple",
          "Plan Rows": 93989,
          "Plan Width": 36,
          "Planned Partitions": 0,
          "Plans": [
            {
              "Alias": "string_encrypted_100000",
              "Async Capable": false,
              "Node Type": "Seq Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 100000,
              "Plan Width": 32,
              "Relation Name": "string_encrypted_100000",
              "Startup Cost": 0.0,
              "Total Cost": 16759.0
            }
          ],
          "Startup Cost": 17009.0,
          "Strategy": "Hashed",
          "Total Cost": 18183.86
        }
      ],
      "Startup Cost": 19358.72,
      "Strategy": "Plain",
      "Total Cost": 19358.73
    }
  }
]
```

**1,000,000 rows**

```
Aggregate
  Aggregate (Hashed)
    Seq Scan on string_encrypted_1000000
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
      "Partial Mode": "Simple",
      "Plan Rows": 1,
      "Plan Width": 8,
      "Plans": [
        {
          "Async Capable": false,
          "Group Key": [
            "(((string_encrypted_1000000.value).data ->> 'hm'::text))::eql_v2.hmac_256"
          ],
          "Node Type": "Aggregate",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Partial Mode": "Simple",
          "Plan Rows": 618824,
          "Plan Width": 36,
          "Planned Partitions": 16,
          "Plans": [
            {
              "Alias": "string_encrypted_1000000",
              "Async Capable": false,
              "Node Type": "Seq Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 999482,
              "Plan Width": 32,
              "Relation Name": "string_encrypted_1000000",
              "Startup Cost": 0.0,
              "Total Cost": 167309.52
            }
          ],
          "Startup Cost": 244457.04,
          "Strategy": "Hashed",
          "Total Cost": 265857.13
        }
      ],
      "Startup Cost": 273592.43,
      "Strategy": "Plain",
      "Total Cost": 273592.44
    }
  }
]
```

**10,000,000 rows**

```
Aggregate
  Aggregate (Hashed)
    Seq Scan on string_encrypted_10000000
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
        "Inlining": true,
        "Optimization": true
      }
    },
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
            "(((string_encrypted_10000000.value).data ->> 'hm'::text))::eql_v2.hmac_256"
          ],
          "Node Type": "Aggregate",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Partial Mode": "Simple",
          "Plan Rows": 4273344,
          "Plan Width": 36,
          "Planned Partitions": 128,
          "Plans": [
            {
              "Alias": "string_encrypted_10000000",
              "Async Capable": false,
              "Node Type": "Seq Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 10000000,
              "Plan Width": 32,
              "Relation Name": "string_encrypted_10000000",
              "Startup Cost": 0.0,
              "Total Cost": 1673105.0
            }
          ],
          "Startup Cost": 2444980.0,
          "Strategy": "Hashed",
          "Total Cost": 2635115.55
        }
      ],
      "Startup Cost": 2688532.35,
      "Strategy": "Plain",
      "Total Cost": 2688532.36
    }
  }
]
```

</details>

![Query Performance - GROUP_BY/count_groups_encrypted](query_group_by_count_groups_encrypted_chart.png)

#### count_groups_plaintext

**Description:** Plaintext baseline: GROUP BY on a plain TEXT column, same query shape as the encrypted scenario

**SQL Query:**
```sql
SELECT count(*) FROM (SELECT 1 FROM {TABLE} GROUP BY value) g
```

**Table: `string_plaintext_{rows}` with unencrypted high-cardinality random strings (`md5(random()::text || ordinal)`). Populated via SQL by `mise run prepare:string_plaintext` — no encryption-client dependency. Index: none. Same `SELECT count(*) FROM (SELECT 1 ... GROUP BY value) g` shape as the encrypted scenario, so the wall-clock delta between this and `count_groups_encrypted` is the EQL recipe's overhead relative to a bare-PG aggregate on a TEXT column at the same row count and cardinality.**

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 100,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 1,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_
- 10,000,000: _none — planner picked a sequential / hash-aggregate / sort plan_

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | 1 | 2.50ms | N/A |
| 100,000 | 1 | 29.32ms | N/A |
| 1,000,000 | 1 | ⚠️ 421.09ms | N/A |
| 10,000,000 | 1 | ⚠️ 23.629s | N/A |

_Rows (est.) is the planner's estimate from `EXPLAIN` captured before the bench loop. For LIMIT-bounded queries it matches the LIMIT; for aggregates it's the estimated group count._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Aggregate
  Aggregate (Hashed)
    Seq Scan on string_plaintext_10000
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
            "string_plaintext_10000.value"
          ],
          "Node Type": "Aggregate",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Partial Mode": "Simple",
          "Plan Rows": 10000,
          "Plan Width": 37,
          "Planned Partitions": 0,
          "Plans": [
            {
              "Alias": "string_plaintext_10000",
              "Async Capable": false,
              "Node Type": "Seq Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 10000,
              "Plan Width": 33,
              "Relation Name": "string_plaintext_10000",
              "Startup Cost": 0.0,
              "Total Cost": 184.0
            }
          ],
          "Startup Cost": 209.0,
          "Strategy": "Hashed",
          "Total Cost": 309.0
        }
      ],
      "Startup Cost": 434.0,
      "Strategy": "Plain",
      "Total Cost": 434.01
    }
  }
]
```

**100,000 rows**

```
Aggregate
  Aggregate (Hashed)
    Seq Scan on string_plaintext_100000
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
            "string_plaintext_100000.value"
          ],
          "Node Type": "Aggregate",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Partial Mode": "Simple",
          "Plan Rows": 100000,
          "Plan Width": 37,
          "Planned Partitions": 4,
          "Plans": [
            {
              "Alias": "string_plaintext_100000",
              "Async Capable": false,
              "Node Type": "Seq Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 100000,
              "Plan Width": 33,
              "Relation Name": "string_plaintext_100000",
              "Startup Cost": 0.0,
              "Total Cost": 1834.0
            }
          ],
          "Startup Cost": 10334.0,
          "Strategy": "Hashed",
          "Total Cost": 12896.5
        }
      ],
      "Startup Cost": 14146.5,
      "Strategy": "Plain",
      "Total Cost": 14146.51
    }
  }
]
```

**1,000,000 rows**

```
Aggregate
  Aggregate (Hashed)
    Seq Scan on string_plaintext_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "JIT": {
      "Functions": 5,
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
      "Partial Mode": "Simple",
      "Plan Rows": 1,
      "Plan Width": 8,
      "Plans": [
        {
          "Async Capable": false,
          "Group Key": [
            "string_plaintext_1000000.value"
          ],
          "Node Type": "Aggregate",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Partial Mode": "Simple",
          "Plan Rows": 1000000,
          "Plan Width": 37,
          "Planned Partitions": 16,
          "Plans": [
            {
              "Alias": "string_plaintext_1000000",
              "Async Capable": false,
              "Node Type": "Seq Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1000000,
              "Plan Width": 33,
              "Relation Name": "string_plaintext_1000000",
              "Startup Cost": 0.0,
              "Total Cost": 18334.0
            }
          ],
          "Startup Cost": 103334.0,
          "Strategy": "Hashed",
          "Total Cost": 128959.0
        }
      ],
      "Startup Cost": 141459.0,
      "Strategy": "Plain",
      "Total Cost": 141459.01
    }
  }
]
```

**10,000,000 rows**

```
Aggregate
  Group
    Gather Merge
      Group
        Sort
          Seq Scan on string_plaintext_10000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "JIT": {
      "Functions": 9,
      "Options": {
        "Deforming": true,
        "Expressions": true,
        "Inlining": true,
        "Optimization": true
      }
    },
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
            "string_plaintext_10000000.value"
          ],
          "Node Type": "Group",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 20000040,
          "Plan Width": 37,
          "Plans": [
            {
              "Async Capable": false,
              "Node Type": "Gather Merge",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 16666700,
              "Plan Width": 33,
              "Plans": [
                {
                  "Async Capable": false,
                  "Group Key": [
                    "string_plaintext_10000000.value"
                  ],
                  "Node Type": "Group",
                  "Parallel Aware": false,
                  "Parent Relationship": "Outer",
                  "Plan Rows": 8333350,
                  "Plan Width": 33,
                  "Plans": [
                    {
                      "Async Capable": false,
                      "Node Type": "Sort",
                      "Parallel Aware": false,
                      "Parent Relationship": "Outer",
                      "Plan Rows": 8333350,
                      "Plan Width": 33,
                      "Plans": [
                        {
                          "Alias": "string_plaintext_10000000",
                          "Async Capable": false,
                          "Node Type": "Seq Scan",
                          "Parallel Aware": true,
                          "Parent Relationship": "Outer",
                          "Plan Rows": 8333350,
                          "Plan Width": 33,
                          "Relation Name": "string_plaintext_10000000",
                          "Startup Cost": 0.0,
                          "Total Cost": 250000.5
                        }
                      ],
                      "Sort Key": [
                        "string_plaintext_10000000.value"
                      ],
                      "Startup Cost": 1663673.46,
                      "Total Cost": 1684506.84
                    }
                  ],
                  "Startup Cost": 1663673.46,
                  "Total Cost": 1705340.21
                }
              ],
              "Startup Cost": 1664673.49,
              "Total Cost": 3630090.96,
              "Workers Planned": 2
            }
          ],
          "Startup Cost": 1664673.49,
          "Total Cost": 3671757.71
        }
      ],
      "Startup Cost": 3921758.21,
      "Strategy": "Plain",
      "Total Cost": 3921758.22
    }
  }
]
```

</details>

![Query Performance - GROUP_BY/count_groups_plaintext](query_group_by_count_groups_plaintext_chart.png)

### MATCH Queries

#### eql_bloom

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

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | 1 | 927.51μs | 27.51ms |
| 100,000 | 1 | 2.59ms | 30.83ms |
| 1,000,000 | 1 | 17.27ms | 45.97ms |
| 10,000,000 | 1 | ⚠️ 192.31ms | ⚠️ 187.73ms |

_Rows (est.) is the planner's estimate from `EXPLAIN` captured before the bench loop. For LIMIT-bounded queries it matches the LIMIT; for aggregates it's the estimated group count._

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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{865,998,1735,1018,1751,1760,1031,1637,1057,710,1496,1200,587,895,1453,830,582,1596,1109,421,1143,61,792,1574,1845,2028,1183,1068,1500,682}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{865,998,1735,1018,1751,1760,1031,1637,1057,710,1496,1200,587,895,1453,830,582,1596,1109,421,1143,61,792,1574,1845,2028,1183,1068,1500,682}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{61,1760,1453,2028,865,1500,998,792,1031,1735,1068,1496,682,1109,1637,1143,895,587,421,1845,1596,830,582,1057,710,1018,1183,1574,1200,1751}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{61,1760,1453,2028,865,1500,998,792,1031,1735,1068,1496,682,1109,1637,1143,895,587,421,1845,1596,830,582,1057,710,1018,1183,1574,1200,1751}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{2028,865,421,587,1496,1057,582,682,1143,895,710,1751,1500,1031,1760,1596,998,1574,1068,1735,1109,61,1200,1183,1637,792,830,1018,1453,1845}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{2028,865,421,587,1496,1057,582,682,1143,895,710,1751,1500,1031,1760,1596,998,1574,1068,1735,1109,61,1200,1183,1637,792,830,1018,1453,1845}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1453,830,1057,1500,865,1637,792,1143,682,1200,895,1018,61,1574,1109,1496,1183,1845,2028,998,1760,1068,710,421,587,1596,1031,1735,1751,582}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1453,830,1057,1500,865,1637,792,1143,682,1200,895,1018,61,1574,1109,1496,1183,1845,2028,998,1760,1068,710,421,587,1596,1031,1735,1751,582}'::smallint[])",
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

#### eql_cast_firstname

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

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | 1 | 616.62μs | 25.94ms |
| 100,000 | 1 | 1.14ms | 27.95ms |
| 1,000,000 | 1 | 5.05ms | 34.23ms |
| 10,000,000 | 1 | 40.46ms | 65.60ms |

_Rows (est.) is the planner's estimate from `EXPLAIN` captured before the bench loop. For LIMIT-bounded queries it matches the LIMIT; for aggregates it's the estimated group count._

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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1033,1554,453,91,461,1393}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1033,1554,453,91,461,1393}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1554,91,453,1033,461,1393}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1554,91,453,1033,461,1393}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1033,1554,453,461,91,1393}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1033,1554,453,461,91,1393}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1033,91,1554,461,1393,453}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1033,91,1554,461,1393,453}'::smallint[])",
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

#### eql_cast_lastname

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

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | 1 | 1.04ms | 28.18ms |
| 100,000 | 1 | 2.66ms | 30.43ms |
| 1,000,000 | 1 | 17.29ms | 44.47ms |
| 10,000,000 | 1 | ⚠️ 165.82ms | ⚠️ 187.53ms |

_Rows (est.) is the planner's estimate from `EXPLAIN` captured before the bench loop. For LIMIT-bounded queries it matches the LIMIT; for aggregates it's the estimated group count._

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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1845,1500,1574,1596,1183,865,710,1637,998,1496,61,1735,587,1031,895,1068,1453,1751,1143,421,1200,2028,792,582,1109,1057,682,1018,830,1760}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1845,1500,1574,1596,1183,865,710,1637,998,1496,61,1735,587,1031,895,1068,1453,1751,1143,421,1200,2028,792,582,1109,1057,682,1018,830,1760}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1057,1845,582,1018,587,1031,1735,1453,1496,421,1183,710,1500,1760,682,830,1751,1596,998,2028,1637,61,1068,792,1574,895,1143,1200,865,1109}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1057,1845,582,1018,587,1031,1735,1453,1496,421,1183,710,1500,1760,682,830,1751,1596,998,2028,1637,61,1068,792,1574,895,1143,1200,865,1109}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1845,682,830,1735,1031,1018,865,1637,1453,61,1183,421,1760,1143,587,710,792,1596,582,1068,1500,2028,1109,1751,1574,1496,1057,895,998,1200}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1845,682,830,1735,1031,1018,865,1637,1453,61,1183,421,1760,1143,587,710,792,1596,582,1068,1500,2028,1109,1751,1574,1496,1057,895,998,1200}'::smallint[])",
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
              "Index Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1143,582,1496,865,682,1183,421,2028,1500,61,998,710,1845,1018,587,1031,1574,1735,1637,1109,1453,1760,1596,792,830,1200,1751,1057,1068,895}'::smallint[])",
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
          "Recheck Cond": "((eql_v2.bloom_filter(value))::smallint[] @> '{1143,582,1496,865,682,1183,421,2028,1500,61,998,710,1845,1018,587,1031,1574,1735,1637,1109,1453,1760,1596,792,830,1200,1751,1057,1068,895}'::smallint[])",
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

### ORE Queries

#### range_gt_10

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

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | 10 | 1.50ms | 29.40ms |
| 100,000 | 10 | 976.68μs | 30.24ms |
| 1,000,000 | 10 | 1.17ms | 28.94ms |
| 10,000,000 | 10 | 1.31ms | 28.55ms |

_Rows (est.) is the planner's estimate from `EXPLAIN` captured before the bench loop. For LIMIT-bounded queries it matches the LIMIT; for aggregates it's the estimated group count._

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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb2817ce2b93478d2e186b01c0864d365f5e681114cf4a542a2b85b0d1dcf8adcb79a5bc1e0a189f65fa7339e21bc6d88ad013dd40988b2797e29c3b64c6d98ce10c6b454aca8ac1505cd2f632b600d48d9e57450526db2b2cc0c74e8f0de7ba4519f21e6fb614cd50f8318b6f70c0f258293c57607eff11d954cbdae9443333420d75af29cda4baeaaa5f4a5cd38bc7ed2bdcad3c8e797837b384348b8aefa2d2e1833f73db493b9e53966d8c21df808e048f4a9ad86d347f9372bbe23a47e96adb85ad1bec51ae648881c6fbc789e22163f84de788e7d75e5a0b1dfb64f014238c0567bed07063b57611c170f59b65e2b251746f12e2e1030a8e8507c77304269d22671f37f5dd116e29403a471aa5cf17\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5050,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 7791.5
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28980b20bf108668388638b6e601a20ed876445198688a5d266014e00b5e34076375d129887c181c56a060289af61747a28f692f2f2b85406045b209ca5f74e3b3eb2e6e3354114cfde7715a094255a7bb7163a4f20b6ee62db4ba5dad15d203a417506398077408564d5a530fdd8b8bd290ddc255610813455226efbbdd872086bba4a7b6ee20be8bfd115d08670d2174ce5bb9cebb5ecb8e975519ff97636761def8d7d8cb65d08bd64690a80214f64d3b1f69950ff4c01ef6a012e86be580cf974114447da0d70f244af9d79753ab65dfefbcf50f52b65e8d5997bd63129e19c300ef85c697d8ef93fda724576de13dc4558ca70e75e3cf07438d6b7da103fa7c030a9d7c7031b2cf2bbf164b608d06\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 33333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 73619.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 22.09
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28e67c5c742bbb32a2787920c52f51c95fb332490558c9c4257dd62a8890009deb1875465951a7e0e5d23bc7057c3012b92c51a03714ad703d4b3a20cd537b3bcc874561522b6fb6886dc77a551840a9349c8387dfc0656c89a1ee49fc506c22c23a5a09d6e9c91d604784fee1c969e8907d5dedd7b11a34e13ebcff725212c76acaef861120137157fb8abe1ab8d1899718a40f2c72211f319294569e51dbe3501f1d8bb2d9cf4ab406891a605ba2cf78ae50ea10882f061bcf387f2c61e4f71a8981ed8c675159e8505a0ebe1f92adde7cf0994511a9d2a0a2210c5fc51dbabece4cb8868bfffd11012532a598cf5d52ee88b0348798bfde9196a66b66386365d98a74377211552acdb7460dc88039e8\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 736191.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 22.09
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb289161674bf4e1c28e73caaa834dc21dbd88eb8b876c124726a77adf6b0c73cc161cd4977ed110cc6e14ab232ad0fc14b457b37e195bd963b738310274fdd7951a3ee2e0edcabf1a7b56d1d018dc2dfcb9c87fba6c919f77f945de184e67fe62e42d837fd24eb3878be2df92aaeba606b64b8e4edad95e42015b8746db94222acda1a82e385f3d3ee5345a268b0abd6b6abfac704e703532834c57686c7e4b771049b57ae6d77655b63cfae34baf348f52bec7a45c560959f4a1ec9c2eaaf81e8fb2ac0cdd0721fc17dee72316374e6f71b5a4b23097f09ab3e0cd46281cb4d16d3d9939c7fc9f16e46eb103f607abba433abb20de3607e512b1a347ec7690d83a8946111f411893b66793de0834beb797\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 3333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7361905.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 22.09
    }
  }
]
```

</details>

![Query Performance - ORE/range_gt_10](query_ore_range_gt_10_chart.png)

#### range_gt_100

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

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | 100 | 6.73ms | 46.68ms |
| 100,000 | 100 | 6.75ms | 45.32ms |
| 1,000,000 | 100 | 6.80ms | 46.54ms |
| 10,000,000 | 100 | 6.74ms | 44.68ms |

_Rows (est.) is the planner's estimate from `EXPLAIN` captured before the bench loop. For LIMIT-bounded queries it matches the LIMIT; for aggregates it's the estimated group count._

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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28934e37c8ab2caaebae6b5e0452177b4b0858f16b024c30b9f292188adc749aa85971e5f33e45b4cc886c1ad34bacb8e03a5f1feca2391301d1d1875fc8d824ed8219ea22fdde9dae3c782c4a070ff206e16dc56c0a7b29765e86138a90c4488ade8075a193c26509d3b1dd51781afeba7461c7bcc109cc4ed1daed8469fa66e5b051e0a952a3c0032f0bda23ef2ea4a58fe7e327baae1f24fbb3abff43b198add2c520698a12cd9959c269fe8a155bd800de5d4a23c9f2aa737bddfae37bf6669fe09ad9c9348dccb57aaa75d9837be787818953a96187e13373087ddfc9e32ac3fe3f7bdd33e9d496a69ea18bbd71401642cfa3e77246558618118ec789a1d46b83349d10662fe4d1c3f6200470f01d\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5050,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 7791.5
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 154.29
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28d3ae3ec985fff16d19e5239be0dd36b6b1461cd2a9460ad72dfdb5f026bab3f68668520eb92cc682e592b2e4f766b65c6b82c848a0cdce4a00807616adedee178688db0336b8f4de57535507fee25b91a97f6ce4cd393f265395d7ceb03ca4b2be16465d06b49c96586a0828677e9939485720fa900bfe23c9ad6b92dfba19b68b010377edc76d6fa2e07815f9246cfd2747e600375ea4f5ab309a6de9a723e94808ae935a409c71f9570cb96594f69811cbd26fef4f4157b1418e4d1ca123dd78fea435a865af34b33f3186d6ccfa4c572c5ba15950024227b7d9787c6fd81c36fd6067b6812d7bccac91c3332c0fc587d367232f11304124537443c7734036f70c88e6a18dd115d3a1d419b7a26a73\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 33333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 73619.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 220.86
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb282f4dd513a78aebbec8ca692b96cdae97e9b6c41c10581f11c9ec33769a5d2eb0c8efbb27b03691cc3485710a03010eeeead21c234f65800ebb86fecbb0768cf2f8cbb843bb1b305b73b3dacc7843c8cb60c3730d327eb2fd34ec2e828f857cc77f51f7ef8bab0cf4a14a6f6cf24a2fb42dc119f0f0f665c4b15751e1d8358d8f5fc60e368012b52d4d2a0861109691e15b44d216d1a73f95f28ab34be74c5a5c26b2e9ee7a2538b2db54200201e64022728dbbbdc77bc9516b43fcc8501a5743615eaf814dc80cde01cecf99f578ab01fad07c7870ab81855b9cfddb79ecc073cd81de16c20ea228132d31ff0c6828fa5994153d6df51659ccc86465336ae659fba38b9fddbbebf969c213bf7e4af10c\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 736191.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 220.86
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) > '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb289debaf9f991b0e2e92fa616e2c52a42610746a6585a09e73f2db9ac3e0a60fed2310d939609e95e0d1adf353ad8b1517e6a2443523f2a8e60f83f5ece6864140c35cbd5ca3b737da90480ee7f6e9c9c6541b904ef83ec41efc4d62eabb4bdeb5dd201155113c10fa2bd72065862f4f71d21b668c3feb2c336a0097fe8854f5f955b9bf9e25b3e0f0d9c96d7901544fc46cff5f824ded8b4e3a0968f434c610242319a0d8e12751e8c2a17ea7a3449785baec5ca7d6782661461cde717c082c21553bbae14a5e505e2cb9d1364293b983d21696188e7e0d2312f99bdb431e88c203da9ca221ef9a387aac58ca9e7558e66ca2e9b81dc440e7da657f673ad5d4c8b71005d3bf95eb3900d967fb9712e5d4\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 3333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7361905.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 220.86
    }
  }
]
```

</details>

![Query Performance - ORE/range_gt_100](query_ore_range_gt_100_chart.png)

#### range_lt_10

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

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | 10 | 1.56ms | 28.54ms |
| 100,000 | 10 | 960.17μs | 30.66ms |
| 1,000,000 | 10 | 931.55μs | 28.29ms |
| 10,000,000 | 10 | 1.28ms | 28.38ms |

_Rows (est.) is the planner's estimate from `EXPLAIN` captured before the bench loop. For LIMIT-bounded queries it matches the LIMIT; for aggregates it's the estimated group count._

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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28fe666ef7ea3c2e11b4c35ac384045d48fc60112f8fd92c3eb87e5f6c89ca484edf7db1c0a148bf627f5079ea3a71bd55cfe91941934e4f6cd1044862af61c62c72da9cd377788e6dcf0acd7c3ebf5a682a16a1b93409717c1849bd9e6c56e052b97efe13b6d411811520c2f827e63e2d2b88a793ffd9275d1f93d920d195fe180d93bf0d30b78d736c453172bf2cd35ef7ccdf2aeb291942b9082b5ddaf0fdbc354dc47d6d3ba33ba9a0415cd52ba208fde55df87c03c9b8d23160005c156a3ca8153bb91448b78e504382bd278e4b1b4143b58c60a97eb8df45342f2edf64b9f8f0479557d69e68ed1b5f4137339f519ad10e475fb95bce99725086ffcda5a6b1c568c844c9a7540fed0b483828352a\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4949,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 7766.25
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28a033beb35af0337a1e833a4a41dbd757492fdc11701ee223c2506b57dd41d478a483190a3fd3d446e7fe27b9ed375453537a04ac6df237bbc4709f3bf6da861c7a60ca250a178f1d81d22b1a90c13557661ddd1d0c4a981523d29be5286280487150b20aa4c18c57cfd7adca0fc599b01f4f88f42f87bee6fbb221fbd29821c2b88be9f6cfd584643f19f1c95e837a664ed721daa2d5fbbcf0e95a12c7ff7b99e2a4486379c0521c2deb090aab1759fb593360b61f5da8295a4fb1c988f32f07860de7a60f755b65dacca90d5b488adcf3273c63cb9d379157fb079fe43bd1351b2aa22380490978150022c802398c0ebb19dfc312b72c509a0256d21f769f054fdea80b516e294594daf9c010aa3dff\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 33333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 73619.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 22.09
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb2852dd641af35f85678b9763d22df20b5c661af02730af1dad4f648aca466b1400466e53c2a5192636ba261eb85dc4abba14b7fba0da00e6c5d8d04073af6d64b32362459f451e8549d3ac6575ee4ce6b2feeefea4919cdab79c7a87e93de84d6f6a185ad9c8ef1da18a410fb3ce8b4c6df9cf68951e70062f95b56a96fb65adbf4913479617c41e01892b744151378f77588eb90e72a0dde315c53c525ee5b4d572cced711b4b80af9e81f2b257cc5f755940a3057ee5669ea685396f01e25ed3f29eb1202a80f6230e92668b04dae9e71188ba90ff93ccd5b971219cb01b4360ecabd4d4f644e5a5c7958d05414c8837efaad4e91dc0ff60cbf5cfe717f3648ccfeb90649367e1d42951795fac8471fe\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 736191.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 22.09
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28c4e2c6c7ebf0c5e05185e7581d967b4bcdbd02eaf3a12fc72f09e3dab676ea6eebc5f7e655db087f86ae65b1940581f49da1fc737f2e609a25b9c70c4261cc4477c25c190298e65062ce719dd458ef23f3a97b7e12ae4de8c8f7afc93108c795dbf73bac1ad0a682ca54b8bdb4002c827f19698e0d4f1d91804686248fd515621e7ad1410cde2cdd6878e72f15ce55c231a8d15486edd061651f9a16ae430e3c7ef7dab1b9b3e72018661ddd748a0b787395da7ff4e39d0a508b8bedb3c30476f0f5b5eec0cc6f58bf600084d9b97c93cee15191505fe87c3daca9cd42d68e286c99140b1ff5af98e095a83b65e304e6295f6e5db8bb41841a2732770167e87521f960a9784a2513f1b1a7bbbaf406eb\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 3333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7361905.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 22.09
    }
  }
]
```

</details>

![Query Performance - ORE/range_lt_10](query_ore_range_lt_10_chart.png)

#### range_lt_100

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

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | 100 | 6.61ms | 43.60ms |
| 100,000 | 100 | 6.53ms | 42.58ms |
| 1,000,000 | 100 | 6.74ms | 45.96ms |
| 10,000,000 | 100 | 8.80ms | 43.61ms |

_Rows (est.) is the planner's estimate from `EXPLAIN` captured before the bench loop. For LIMIT-bounded queries it matches the LIMIT; for aggregates it's the estimated group count._

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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28613a47d8d474cd8e194852d31798a638b7c5888c014b63a30adaa563dd99bd94952bc58c0146a43be7957396ae6ea520c1363afc8d2161838e7e453dd243ca8c4c413afa00fbcb057b752016d69c4df4b49cd0ecd14cd9ba3704d695b44daff2c4232fbb64a35a77caf8f869abb2ec2489ad5a87f974548c59259dba7db48917c4bd53319e5a58cce7329bd7f0b79017352a7dff4e4e3a240cb52ebbf35faa8a4ae163856d684a21f3e3a2b4b7d3d88f24c67882502ee28a5e2fa209df5ac8266606d2758b0d8f90c406ceed03faa9927e0d3414275d20e170ba39f47c370de73f4105e5c2368eb603ec02b3d10f31e63cc0adb3494a2e637be59831712de558e297b2b9762aaa023130aa2f36ccb9a6\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4949,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000",
          "Startup Cost": 0.0,
          "Total Cost": 7766.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 156.93
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb284ccf21aac875dac647810b92b06a1f9b98496c27676f668910513dd3737c176101dd7f21e2bae4dc5c08161c515855e1e583e7d7392c99dc7c403bcca3689bd71e64342b26040c6433e0a9e54794eb681babc8d24270eecc84bb92ebaa836d1037b98de5ca616c0f915605c8f954ba31ecc2a525ba46b2ee3d8ec7c7b83af6775c3a722e032cbd37948b4796dca9e3f8d7944e7ea8ba9916697b6abfa647cdf58fc78c92e8561e87b848246bd75699e804cedc763e87d245b700c492b874b50bf854e5c86b57ae975823a71c73486696023b8c4d19bd2c1be5b2f1417c252f2fc627ffb6c94480ef0fb21ddb9321d129ecf91218f3c5b9ebf8d76b47c7c076c83c9888cf8b931b2a3132c5b7771adbdf\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 33333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_100000",
          "Startup Cost": 0.0,
          "Total Cost": 73619.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 220.86
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28da9f8b2f4a04d727439450791a2d22ef57461db05fdc4394e8d2ab653878651c049dd7e35424bcf81d0f52bb8c92fe90008e3cb57b72c6f6fd66f3e4d7bcd25c97bf39e81776e9f43ffbbae6cc991e158691ca9504779797cfcf6e3de8ecae70b8a7fcbfd61e62e5fbcb0f31d6926acce14714f4f0bfcf4e9a764d6f0f31558e776619448216389a2cf97ce10b7b6fba4106e618b189460a4d6c73d6539119031b66994e250a021ce7316e2bae4a003e4edc7fb9abc985ab9cac23de2013fc3ab136b77ec453a4048a4a823dffd0946226ba13768c670a4ce53a9456005c40f86bb0d549d5819528436595605ce19013551ac4f52611c714288b97530f71939837d98ef8e90722aa9648ba03516cf5cd\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_1000000",
          "Startup Cost": 0.0,
          "Total Cost": 736191.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 220.86
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
          "Filter": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28f9533cf70d88611efd8c5f50fe07ebfb32020fe350dab197f35b352d1e8fa063389358d26faddd169633742b39e4aef4124083587abe2a5910f2bdef1c71b11c88b64ffdfe6a87a34e234fbe6509c584869cf7794efa5a71aa2ac55165c7440d5c2290afb506b90727982a6a3b0e5a1492f52bf36718e62fc4f2ab000e229584c13eb8f663f8bb2eb5d0b916204ceb8e6d2b11af6e1b54a38bbe67ed2c1e78abff6b334039af0758ae689596dcfb4db1a9d0fe2bd11eeed3eabc8ea2e83024c322657d1aea63798549269ad6d595f8479bed39c12903d95503b175fd7cd1c7a66e8d0d16d4c224621dd533ac32aaf96d359d303ee973d563e45212ca9acc6256bf01b1d50d89dc5185605245600fd8f9\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 3333333,
          "Plan Width": 36,
          "Relation Name": "integer_encrypted_10000000",
          "Startup Cost": 0.0,
          "Total Cost": 7361905.25
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 220.86
    }
  }
]
```

</details>

![Query Performance - ORE/range_lt_100](query_ore_range_lt_100_chart.png)

#### range_lt_hybrid_ordered_10

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

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | 10 | 1.15ms | 28.71ms |
| 100,000 | 10 | 1.05ms | 29.30ms |
| 1,000,000 | 10 | 1.24ms | 29.26ms |
| 10,000,000 | 10 | 1.08ms | 27.93ms |

_Rows (est.) is the planner's estimate from `EXPLAIN` captured before the bench loop. For LIMIT-bounded queries it matches the LIMIT; for aggregates it's the estimated group count._

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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb2800fe970e78f354e987ce5bb688b0859b0abd75318a9549123eaff5ae591c8951cb0bfe6ddef9f0f33d99741f144a5ae4aa5cdc4029e6db0f7c19b661fb8b5f795440bbd0eb6def2aedd59d8d30c5a41fed198793688dd572139f766274d6656e820df591b01af34320c0a51358cdb6e14402f2d3912939f6e92bc73d80c422fcf3cc11ccb6410f9d9506b69702ff8597f4075545c1d1eff2126983ee086c1ca4c6983b17a40ac51ee203dcf223a5158edef85a15f6066d85f0ff8eddf4429f63cc7d1912717f8fb7f17a3ef4ef669bddb0db37b9d9ef5c3dfde80974dbe244d8d8cdb3361d0901de4cbdc98f3eb21f228930a300c7a2502ddb310ab3e2c32574bdba542151c649d054b036b3c07c41c8\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_10000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4949,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.54,
          "Total Cost": 9829.55
        }
      ],
      "Startup Cost": 0.54,
      "Total Cost": 20.4
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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28e70a903a5ae9f1ef15e945c00baff99d426e216bafdbc3ccf0235f0c4f601198bbc1e0a3405b3dd5c7ea295a6289814d64e87934bbcee064476b1e8a05a1b93235294fe17daf9aa71de75a6b8677b8787e4b970a8886c244e320a5cb1cf71a8c781a3c55a7b3fa4002cb8dbe0e9691308662f7268e2c852e187b8617ed1f43624cb4b40bd04e2027ee5a239284bbfefbadf736acd1ffc9b23118e778dec41787be1e1c477e0154409665e96ecf83102517e2bac2ca7c14b19398dd887ee7024a0abd5ba1cdcf37692053f096d5a06b3ce6bc10cf83656b49f7edc806f29dcddffdf723d1050f0ea62961396c6c08629459f0df66c1bbfa3b66727dbe392dc36f1f0794a980ab95e895164b9fe224492f\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_100000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 33333,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.67,
          "Total Cost": 84790.49
        }
      ],
      "Startup Cost": 0.67,
      "Total Cost": 26.1
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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb2825e553b73d0e9400f14833cfc1975af57f8b79e9e873918bc567c4c6d04c923efafffc245dc8c8a338f3aa6b685249a1ab53d8422625a534e35141567a37bf0afa5157c81b9c00780121a322d78bce5bf515e7f3f34759001c3faf4687a278f0ca248323bad12fee4382f371368058ae8a177cb5c33e6eca25039ad0a0c15f588e3f7221bd024953e0f55246ca05d2e14008b09b4f928c3f1849e4939688c79fd7beaae5907770d51c30741af2a3c956e78c90345868116dd5fd21739f2669715afb276ddc6aba944d740cc9dba21672c03e66d5f2f6e00bc591b60fcf30d84ff59a813f4bc0606d313388ef9c0704aa83ebdb1daa3602ba6ea1fb602690ade2c1fab04b438be6b4999d179132eac72d\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_1000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 333333,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.8,
          "Total Cost": 847836.63
        }
      ],
      "Startup Cost": 0.8,
      "Total Cost": 26.24
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
          "Index Cond": "(eql_v2.ore_block_u64_8_256(value) < '(\"{\"\"(\\\\\"\"\\\\\\\\\\\\\\\\x4a4a4a4a7b75d35a800cfa2ff43dcd1b62771a15e33d22e69fda6d34f8e7f8be158e580893b02f753ecf7644fb8c1aba4adf0b01dd621fb107b51a023311e9484b76a975afea7fd7f7d3e6c4a96effd0ce52df7af438bca65a7cbad28e493fef601c3bf913156337fff25712e92fcecde5fdf2c4c33b0dcf6407e8b578be07e00aad33c3a870cb28e3d7300dc58ea91cf8a4191c154e03703ce05ee3e6171c7d090590845a66319db717e79d096859d7eb0554abffed13033c449740d9cddb18a67b1a6ea2f46859c04deadf6d2b57b5b20239b27be563f0994611654f3fc8e850e52b0e8249efde498f12475f55cb3401f72468f7086b8e4f9ff1be938447ffd322cfbf0105f1adaeca9a843bd877c16de3239270d1bc8f6ce925d98d41227264b1170ae964929af7c99de50a3245c056655ab2a0a7150da5f5b7be71890aef53fc9b4ce802a90298cda3b926902955d63f35cf6aeb5cae873f29baaa351235171e0a323e09baec00c4ae491d91db9f42a8ba4eca64e3d3ba7287fb6ec56fe4d98e4549004447e3786aa6f201ed6b0e63123e5c54206cea\\\\\"\")\"\"}\")'::eql_v2.ore_block_u64_8_256)",
          "Index Name": "integer_encrypted_10000000_ore_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 3333333,
          "Plan Width": 68,
          "Relation Name": "integer_encrypted_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.94,
          "Total Cost": 13113848.76
        }
      ],
      "Startup Cost": 0.94,
      "Total Cost": 40.28
    }
  }
]
```

</details>

![Query Performance - ORE/range_lt_hybrid_ordered_10](query_ore_range_lt_hybrid_ordered_10_chart.png)


---

*Report generated by `report_benchmarks.py`*
