# ORE Benchmarks

Performance benchmarks for CipherStash's searchable encryption operations using Order-Revealing Encryption (ORE) and the Encrypt Query Language (EQL).

## 📊 View Results

The latest benchmark results are available in the [`report/`](report/) directory:

- **[Benchmark Report](report/BENCHMARK_REPORT.md)** - Comprehensive report with performance tables and charts
- Includes ingest throughput, query performance, SQL statements, and index configurations
- Performance indicators (⚠️) highlight queries exceeding 100ms

### Other benchmarks

- **[ZeroKMS vs AWS KMS](kms-app/REPORT.md)** — bulk field‑encryption throughput
  vs AWS KMS. ZeroKMS encrypts/decrypts a whole batch in one round‑trip; AWS KMS
  has no bulk API and throttle‑fails past a few hundred values per request.

### Headline numbers

Query-only medians (no decrypt) from the latest full run against EQL 2.3, across four row-count tiers. Full per-scenario detail — SQL, planner index choices, EXPLAIN plans — is in [`report/`](report/).

| Family | Scenario | 10k | 100k | 1M | 10M |
|---|---|--:|--:|--:|--:|
| **JSON** | contains/functional | 0.2 ms | 0.3 ms | 0.4 ms | 0.8 ms |
| JSON | field_eq/functional | 0.1 ms | 0.1 ms | 0.1 ms | 0.1 ms |
| JSON | field_order/functional | 0.3 ms | 0.3 ms | 0.4 ms | 0.4 ms |
| **ORE** | range_gt_100 | 4.0 ms | 4.2 ms | 4.1 ms | 4.0 ms |
| ORE | range_lt_ordered_10 | 0.5 ms | 0.5 ms | 0.5 ms | 0.5 ms |
| **EXACT** | eql_hash | 0.1 ms | 0.1 ms | 0.1 ms | 0.1 ms |
| **MATCH** | eql_bloom | 0.4 ms | 1.8 ms | 15 ms | 144 ms |
| **GROUP_BY** | low_cardinality — encrypted | 2.2 ms | 20 ms | 93 ms | 776 ms |
| GROUP_BY | low_cardinality — plaintext baseline | 1.2 ms | 9.0 ms | 39 ms | 339 ms |
| **COMBO** | top_n_filtered_group_by | 0.2 ms | 1.0 ms | 5.3 ms | 52 ms |

Selective ORE range scenarios are currently disabled (a planner selectivity mis-estimate) — see [encrypt-query-language#230](https://github.com/cipherstash/encrypt-query-language/issues/230). The pathological `ORDER BY value` shape (no extractor; ~60 s at the 10M tier because the sort key can't match any allowed index — a btree directly on `value` would, but the encrypted body trips Postgres's btree entry-size limit) has been removed from the suite — the EQL perf guide covers it as the documented anti-pattern.

## 🔧 Test Setup

### Hardware & Software

The benchmarks are designed to run on a local development machine with the following stack:

- **Database**: PostgreSQL 17 (native — Homebrew `postgresql@17`)
- **Language**: Rust (latest stable)
- **Framework**: Criterion.rs for benchmarking
- **Encryption**: CipherStash EQL v2 with ORE support

### Database Configuration

```yaml
PostgreSQL 17 (native, Homebrew postgresql@17)
Data directory: ~/.eqlbench/pgdata
Port: 5400
User: postgres
Database: postgres
```

### Test Data

The benchmarks use three types of encrypted data:

1. **Integer values** - ORE-encrypted integers for range queries
2. **String values** - Encrypted strings for exact and pattern matching
3. **JSON objects** - Small encrypted JSON documents

### Data Set Sizes

Benchmarks are run against multiple data set sizes:
- 10,000 rows
- 100,000 rows
- 1,000,000 rows
- 10,000,000 rows (optional)

### Query Types

Three categories of queries are benchmarked:

**EXACT Queries** - Exact match lookups
- Using EQL cast operator
- Using EQL HMAC-256 hash

**MATCH Queries** - Pattern matching
- LIKE queries with EQL cast
- Bloom filter containment queries

**ORE Queries** - Range queries on encrypted integers
- Exact match
- Range queries (>, <)
- Ordered range queries with ORDER BY

Each query is tested with and without decryption of results.

## 🆕 EQL v3 Scenarios

The suite carries an **EQL version axis**: every v2 scenario family has an
additive v3 twin that runs the same scenario intent against the `eql_v3`
schema (per-scalar-per-capability jsonb domains, term extractor functions,
no generic envelope type). Nothing in the v2 path changes — v2 filenames,
tables and tasks are untouched.

### How v3 payloads are produced

cipherstash-client 0.38 emits EQL v2.3 wire payloads only. The v3 paths
encrypt through the unchanged v2 pipeline and convert each STORED payload
with [`eql-bindings`](https://github.com/cipherstash/encrypt-query-language)'s
`from_v2` (currently a path dependency on the EQL repo checkout). Scalar
QUERY conversion is unsupported upstream (no v3 scalar query wire shape
exists), so the v3 query benches encrypt each probe value as a storage
payload, convert it, and compare via the `eql_v3.*_term` extractors (or the
inlinable typed operators, which reduce to the same expressions), e.g.:

```sql
WHERE eql_v3.eq_term(value) = eql_v3.eq_term($1::eql_v3.text_search)
```

### Table / domain mapping

| v2 table | v3 twin | Domain | Notes |
|---|---|---|---|
| `string_encrypted` | `string_encrypted_v3` | `eql_v3.text_search` | Only single-column v3 domain serving both EXACT (hm) and MATCH (bf); requires an extra `ob` ORE term the v2 string ingest doesn't encrypt — `encrypt_string_v3` throughput is therefore not directly comparable to `encrypt_string`. |
| `integer_encrypted` | `integer_encrypted_v3` | `eql_v3.integer_ord_ore` | v2 encrypts `i32` (int4). |
| `category_encrypted` | `category_encrypted_v3` | `eql_v3.text_eq` | |
| `combo_encrypted` | `combo_encrypted_v3` | `text_match` / `integer_ord_ore` / `text_eq` | Per-column capability match. |
| `json_ste_vec_small_encrypted` | `json_ste_vec_small_encrypted_v3` | `eql_v3.json` | SteVec document domain. |
| plaintext baselines | *(shared)* | — | Version-independent; not duplicated. |

### Scenario changes vs v2

- **MATCH**: v3 removes `LIKE` / `ILIKE` — the two v2 `eql_cast_*` LIKE
  scenarios have no twin. Bloom containment (`@>`) is the only encrypted
  text-matching surface; the v3 bench adds an `eql_bloom_bare` scenario to
  price the typed-operator inlining.
- **GROUP BY**: v3 runs encrypted scenarios only; compare against the
  shared plaintext baselines in the v2 family.
- **JSON**: containment uses the canonical v3 recipe
  (`value @> $1::eql_v3.jsonb_query` over a
  `GIN ((eql_v3.to_ste_vec_query(value))::jsonb jsonb_path_ops)` index);
  `field_eq/bare` becomes index-capable (the v3 `->` is inlinable SQL,
  unlike v2's plpgsql).
- **ORE (OPE)**: the `eql_v3.*_ord_ope` domains (OPE-CLLW ordering, wire
  key `op`) are scaffolded but **disabled** — cipherstash-client 0.38.0
  does not emit `op` (CIP-3280 unreleased). See the TODOs in
  `sql/schema-v3.sql` and `benches/ore_v3.rs`.
- A dedicated **conversion-overhead** ingest family
  (`convert_overhead_encrypt_only` vs `convert_overhead_encrypt_convert`)
  quantifies the pure `from_v2` cost: identical encrypt workloads, no
  database writes, delta = conversion.

### Version axis in results and reports

v3 result files carry a `_v3` family prefix (`exact_v3_rows_10000.json`,
`exact_v3_metadata_10000.json`) and every sidecar scenario records
`"version": 3` (absent / `2` = v2, so old files parse unchanged). The
report generators group by version and sort each `_V3` family next to its
v2 counterpart for side-by-side comparison.

### Running the v3 suite

```bash
# Build the v3 SQL installer in the EQL repo (no released artifact yet):
#   (cd ../encrypt-query-language && mise run build)
mise run setup-db-v3        # install eql_v3 + create the _v3 tables

# Query benches (per family / tier, or the full sweep)
mise run bench:query:exact:v3 10000
mise run bench:query:all:v3 1000000

# Ingest benches + the conversion-overhead scenario
mise run bench:ingest:encrypt_string:v3
mise run bench:ingest:convert-overhead
```

## 🚀 Running Benchmarks

### Prerequisites

1. **Install mise** (tool version manager):
   ```bash
   curl https://mise.run | sh
   ```

2. **Install Rust** (via mise):
   ```bash
   mise install
   ```

3. **Set up environment variables**:
   ```bash
   cp .env.example .env
   # Edit .env with your CipherStash credentials
   ```

4. **Install PostgreSQL 17** — the benches run a native local cluster:
   ```bash
   brew install postgresql@17
   ```
   The cluster itself (`~/.eqlbench/pgdata`, port 5400) is created automatically
   on first use by the `postgres-init` task — no manual `initdb` needed.

### Quick Start

```bash
# Start PostgreSQL
mise run postgres

# Set up database (creates tables and installs EQL extension)
mise run setup-db

# Run all ingest benchmarks
mise run bench:ingest

# Run query benchmarks for a specific row count
mise run bench:query:exact 10000
mise run bench:query:match 10000
mise run bench:query:ore 10000

# Run all query benchmarks (all row counts)
mise run bench:query:all

# Generate report
mise run report:build
```

### Step-by-Step Guide

#### 1. Start PostgreSQL

```bash
mise run postgres
```

This starts the native PostgreSQL 17 cluster on port 5400 (data directory `~/.eqlbench/pgdata`), creating and configuring it first if it doesn't yet exist.

#### 2. Initialize Database

```bash
mise run reset-db    # Reset database (if needed)
mise run setup-db    # Install EQL extension and create tables
```

#### 3. Run Ingest Benchmarks

```bash
# Run individual ingest benchmarks
mise run bench:ingest:encrypt_int
mise run bench:ingest:encrypt_string
mise run bench:ingest:encrypt_json_small

# Or run all at once
mise run bench:ingest
```

Results are saved to `results/ingest/*.json`

#### 4. Prepare Tables for Query Benchmarks

Before running query benchmarks, tables need to be populated and indexed:

```bash
# Prepare string_encrypted table with 10,000 rows
mise run prepare:string_encrypted 10000

# Prepare integer_encrypted table with 10,000 rows
mise run prepare:integer_encrypted 10000
```

This process:
1. Checks current row count
2. Drops indexes
3. Inserts additional rows if needed
4. Creates indexes

#### 5. Run Query Benchmarks

```bash
# Run specific query benchmark with specific row count
mise run bench:query:exact 10000
mise run bench:query:match 100000
mise run bench:query:ore 1000000

# Run all query benchmarks for all row counts (10k, 100k, 1M, 10M)
mise run bench:query:all
```

Each query bench writes two files to `results/query/` per row-count tier:

- `<bench>_rows_<N>.json` — criterion's NDJSON message stream (one
  `benchmark-complete` event per scenario, with mean / median / etc.).
- `<bench>_metadata_<N>.json` — JSON sidecar with one record per
  scenario: the exact SQL the bench ran, the bound parameter (encrypted
  payload as JSON, or empty for raw-SQL benches), the
  `EXPLAIN (FORMAT JSON)` plan captured once at startup, and the list of
  indexes the planner picked. Join the two files by the `id` field.

#### 6. Generate Report

Quick overview of all results in the terminal — median runtime per scenario, slowest first:

```bash
mise run report
```

Or build the full Markdown report file:

```bash
mise run report:build
```

This generates:
- `report/BENCHMARK_REPORT.md` - Markdown report
- `report/*_chart.png` - Performance charts (requires matplotlib)

To enable chart generation:
```bash
pip3 install matplotlib
```

## 📁 Project Structure

```
ore-benches/
├── benches/              # Criterion benchmark definitions
│   ├── exact.rs          # EXACT query benchmarks
│   ├── match.rs          # MATCH query benchmarks
│   ├── ore.rs            # ORE range query benchmarks
│   └── *_v3.rs           # EQL v3 twins (exact_v3, match_v3, ore_v3,
│                         #   group_by_v3, combo_v3, json_v3)
├── src/
│   ├── bin/              # Binary utilities
│   │   ├── encrypt_int.rs
│   │   ├── encrypt_string.rs
│   │   ├── encrypt_*_v3.rs      # EQL v3 ingest twins
│   │   ├── convert_overhead.rs  # from_v2 conversion-cost scenario
│   │   └── combine_benchmark.rs
│   └── lib.rs            # Shared benchmark code (incl. the v3 module)
├── sql/
│   ├── schema.sql        # Database schema (EQL v2 tables)
│   ├── schema-v3.sql     # EQL v3 twin tables (eql_v3 domains)
│   └── indexes/          # Index creation scripts
│       └── v3/           # EQL v3 functional index scripts
├── results/              # Benchmark results (JSON)
│   ├── ingest/           # Ingest throughput results
│   └── query/            # Query performance results
├── report/               # Generated reports
│   ├── BENCHMARK_REPORT.md
│   └── *.png             # Charts
├── report_benchmarks.py  # Report generator script
├── mise.toml             # Task definitions
└── README.md             # This file
```

## 🛠️ Advanced Usage

### Custom Row Counts

```bash
# Prepare and benchmark custom row count
mise run prepare:string_encrypted 50000
TARGET_ROWS=50000 cargo criterion --bench exact
```

### Individual Benchmark Runs

```bash
# Build in release mode
mise run bench:build

# Run specific benchmark manually
TARGET_ROWS=10000 cargo criterion --bench ore --message-format json > results/query/ore_rows_10000.json
```

### Database Management

```bash
# Connect to database
mise run psql

# View PostgreSQL logs
mise run postgres-logs

# Stop PostgreSQL
mise run postgres-stop
```

### Report Generation Options

```bash
# Generate report with custom filename
mise run report:build custom_report.md

# Or use Python script directly
python3 report_benchmarks.py --output report/my_report.md

# Specify custom directories
python3 report_benchmarks.py \
  --results-dir results \
  --sql-dir sql \
  --output report/BENCHMARK_REPORT.md
```

## 📈 Understanding Results

### Ingest Throughput

Measures how many encrypted records can be inserted per second. Higher is better.

### Query Performance

Query times are reported both:
- **Without decryption** - Time to execute query and retrieve encrypted results
- **With decryption** - Time including client-side decryption

Times exceeding 100ms are marked with ⚠️ for easy identification.

### Performance Factors

Query performance is affected by:
1. **Data set size** - Larger datasets generally increase query time
2. **Index type** - Hash indexes are faster for exact matches; ORE indexes enable range queries
3. **Query complexity** - Pattern matching is slower than exact lookups
4. **Result set size** - LIMIT clause affects decryption overhead

## 🔍 Troubleshooting

### PostgreSQL Connection Issues

```bash
# Check / start PostgreSQL (idempotent — reports if already running)
mise run postgres

# Restart PostgreSQL
mise run postgres-stop
mise run postgres
```

### Missing EQL Extension

```bash
mise run setup-db
```

### Benchmark Failures

Check that:
1. Database is running and accessible
2. Tables have been prepared with correct row counts
3. Environment variables are set in `.env`
4. CipherStash credentials are valid

## 📚 Additional Documentation

- [Report Generator Documentation](README_REPORT.md) - Detailed guide for the report generation script
- [Report Directory](report/README.md) - Information about generated reports
- [CipherStash Documentation](https://cipherstash.com/docs) - Official CipherStash docs

## 🤝 Contributing

When adding new benchmarks:

1. Add benchmark definition to `benches/`
2. Update `mise.toml` with new tasks
3. Add query descriptions to `report_benchmarks.py`
4. Document the benchmark in this README
5. Run benchmarks and commit results to `report/`

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.
