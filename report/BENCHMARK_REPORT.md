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

**Indexes:**
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

| Data Set Size | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------------------|---------------------------|
| 10,000 | 423.36μs | 457.02μs |
| 100,000 | 424.13μs | 409.56μs |
| 1,000,000 | 448.81μs | 422.23μs |
| 10,000,000 | 419.44μs | 420.53μs |

![Query Performance - EXACT/eql_cast](query_exact_eql_cast_chart.png)

#### eql_hash

**Description:** Exact match using EQL HMAC-256 hash function

**SQL Query:**
```sql
SELECT value FROM {TABLE} WHERE eql_v2.hmac_256(value) = eql_v2.hmac_256($1::jsonb) LIMIT 1
```

**Parameter:** `Bob Johnson`

**Table: `string_encrypted_{rows}` with encrypted string values. Index: Hash-based unique index using `eql_v2.hmac_256`.**

**Indexes:**
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

| Data Set Size | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------------------|---------------------------|
| 10,000 | 400.27μs | 427.53μs |
| 100,000 | 394.22μs | 409.15μs |
| 1,000,000 | 386.27μs | 387.07μs |
| 10,000,000 | 409.12μs | 408.84μs |

![Query Performance - EXACT/eql_hash](query_exact_eql_hash_chart.png)

### GROUP_BY Queries

#### count_groups_encrypted

**Description:** GROUP BY in extractor form on `eql_v2.hmac_256(value)`, wrapped in `count(*)` to isolate aggregation cost from emit cost

**SQL Query:**
```sql
SELECT count(*) FROM (SELECT 1 FROM {TABLE} GROUP BY eql_v2.hmac_256(value)) g
```

**Table: `string_encrypted_{rows}` with encrypted string values (carrying an `hm` HMAC term, configured via the `unique` search index). Index: no index drives `GROUP BY` directly — hash aggregation is in-memory. The extractor's 32-byte HMAC group key fits in default `work_mem`, so the planner picks `HashAggregate` reliably across deployments. **Why the subquery wrapper.** The bench data is `fake::name::Name<EN>` — effectively unique per row, so a bare `SELECT count(*) FROM tbl GROUP BY eql_v2.hmac_256(value)` emits ~one row per input row. Wall-clock time on that shape is dominated by result emission (server-side row construction, network round-trip, sqlx deserialisation, bench iter-and-sum), not by the aggregation work the recipe is actually about. Wrapping the GROUP BY in `count(*)` keeps the inner HashAggregate identical but emits a single row, so the bench measures aggregation cost. The companion `count_groups_plaintext` scenario runs the same query shape against an unencrypted column for comparison. Natural-form `GROUP BY value` against an encrypted column was removed from this bench in an earlier pass because the planner picks `GroupAggregate` + sort against the full ~1-2 KB ciphertext payload at scale — see §5 of the EQL query-performance guide.**

**Indexes:**
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

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------------------|---------------------------|
| 10,000 | 4.95ms | N/A |
| 100,000 | 66.51ms | N/A |
| 1,000,000 | ⚠️ 829.22ms | N/A |

![Query Performance - GROUP_BY/count_groups_encrypted](query_group_by_count_groups_encrypted_chart.png)

#### count_groups_plaintext

**Description:** Plaintext baseline: GROUP BY on a plain TEXT column, same query shape as the encrypted scenario

**SQL Query:**
```sql
SELECT count(*) FROM (SELECT 1 FROM {TABLE} GROUP BY value) g
```

**Table: `string_plaintext_{rows}` with unencrypted high-cardinality random strings (`md5(random()::text || ordinal)`). Populated via SQL by `mise run prepare:string_plaintext` — no encryption-client dependency. Index: none. Same `SELECT count(*) FROM (SELECT 1 ... GROUP BY value) g` shape as the encrypted scenario, so the wall-clock delta between this and `count_groups_encrypted` is the EQL recipe's overhead relative to a bare-PG aggregate on a TEXT column at the same row count and cardinality.**

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------------------|---------------------------|
| 10,000 | 3.21ms | N/A |
| 100,000 | 32.53ms | N/A |
| 1,000,000 | ⚠️ 727.76ms | N/A |

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

**Indexes:**
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

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------------------|---------------------------|
| 10,000 | 1.23ms | 28.20ms |
| 100,000 | 2.97ms | 30.93ms |
| 1,000,000 | 17.76ms | 42.96ms |
| 10,000,000 | ⚠️ 163.65ms | ⚠️ 187.56ms |

![Query Performance - MATCH/eql_bloom](query_match_eql_bloom_chart.png)

#### eql_cast_firstname

**Description:** Pattern matching on first name using EQL cast and LIKE

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value LIKE $1 LIMIT 10
```

**Parameter:** `Bob`

**Table: `string_encrypted_{rows}` with encrypted string values. Index: MATCH index for substring searches. Query returns LIMIT 10 results.**

**Indexes:**
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

| Data Set Size | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------------------|---------------------------|
| 10,000 | 702.86μs | 25.88ms |
| 100,000 | 1.34ms | 27.30ms |
| 1,000,000 | 4.90ms | 30.45ms |
| 10,000,000 | 39.34ms | 65.04ms |

![Query Performance - MATCH/eql_cast_firstname](query_match_eql_cast_firstname_chart.png)

#### eql_cast_lastname

**Description:** Pattern matching on last name using EQL cast and LIKE

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value LIKE $1 LIMIT 10
```

**Parameter:** `Johnson`

**Table: `string_encrypted_{rows}` with encrypted string values. Index: MATCH index for substring searches. Query returns LIMIT 10 results.**

**Indexes:**
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

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------------------|---------------------------|
| 10,000 | 1.10ms | 27.58ms |
| 100,000 | 2.66ms | 29.25ms |
| 1,000,000 | 17.99ms | 42.79ms |
| 10,000,000 | ⚠️ 161.83ms | ⚠️ 186.51ms |

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

**Indexes:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

| Data Set Size | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------------------|---------------------------|
| 10,000 | 1.52ms | 28.54ms |
| 100,000 | 1.20ms | 27.98ms |
| 1,000,000 | 1.17ms | 29.11ms |
| 10,000,000 | 1.48ms | 28.54ms |

![Query Performance - ORE/range_gt_10](query_ore_range_gt_10_chart.png)

#### range_gt_100

**Description:** Range query (greater than) returning 100 results

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 100
```

**Parameter:** `5000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value > 5000 LIMIT 100.**

**Indexes:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

| Data Set Size | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------------------|---------------------------|
| 10,000 | 6.69ms | 45.93ms |
| 100,000 | 6.87ms | 44.72ms |
| 1,000,000 | 7.26ms | 45.68ms |
| 10,000,000 | 7.15ms | 43.42ms |

![Query Performance - ORE/range_gt_100](query_ore_range_gt_100_chart.png)

#### range_lt_10

**Description:** Range query (less than) returning 10 results

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 LIMIT 10
```

**Parameter:** `5000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value < 5000 LIMIT 10.**

**Indexes:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

| Data Set Size | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------------------|---------------------------|
| 10,000 | 1.62ms | 29.18ms |
| 100,000 | 1.05ms | 28.56ms |
| 1,000,000 | 1.10ms | 28.33ms |
| 10,000,000 | 1.11ms | 28.18ms |

![Query Performance - ORE/range_lt_10](query_ore_range_lt_10_chart.png)

#### range_lt_100

**Description:** Range query (less than) returning 100 results

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 LIMIT 100
```

**Parameter:** `5000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value < 5000 LIMIT 100.**

**Indexes:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

| Data Set Size | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------------------|---------------------------|
| 10,000 | 6.95ms | 45.12ms |
| 100,000 | 6.94ms | 46.61ms |
| 1,000,000 | 6.91ms | 48.31ms |
| 10,000,000 | 6.77ms | 45.41ms |

![Query Performance - ORE/range_lt_100](query_ore_range_lt_100_chart.png)

#### range_lt_hybrid_ordered_10

**Description:** Ordered range query (hybrid form: natural WHERE, extractor ORDER BY)

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 ORDER BY eql_v2.ore_block_u64_8_256(value) LIMIT 10
```

**Parameter:** `5000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value < 5000 ORDER BY eql_v2.ore_block_u64_8_256(value) LIMIT 10. The sort key matches the index expression syntactically, so rows stream out of the index already ordered — no Sort node. See §4 of the EQL query-performance guide for the natural-form sort-key trap that this shape avoids.**

**Indexes:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

| Data Set Size | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------------------|---------------------------|
| 10,000 | 1.38ms | 27.91ms |
| 100,000 | 1.02ms | 28.22ms |
| 1,000,000 | 1.16ms | 27.73ms |
| 10,000,000 | 1.02ms | 27.23ms |

![Query Performance - ORE/range_lt_hybrid_ordered_10](query_ore_range_lt_hybrid_ordered_10_chart.png)


---

*Report generated by `report_benchmarks.py`*
