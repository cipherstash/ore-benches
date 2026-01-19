# ORE Benchmarks

Performance benchmarks for CipherStash's searchable encryption operations using Order-Revealing Encryption (ORE) and the Encrypt Query Language (EQL).

## 📊 View Results

The latest benchmark results are available in the [`report/`](report/) directory:

- **[Benchmark Report](report/BENCHMARK_REPORT.md)** - Comprehensive report with performance tables and charts
- Includes ingest throughput, query performance, SQL statements, and index configurations
- Performance indicators (⚠️) highlight queries exceeding 100ms

## 🔧 Test Setup

### Hardware & Software

The benchmarks are designed to run on a local development machine with the following stack:

- **Database**: PostgreSQL 17 (running in Docker)
- **Language**: Rust (latest stable)
- **Framework**: Criterion.rs for benchmarking
- **Encryption**: CipherStash EQL v2 with ORE support

### Database Configuration

```yaml
PostgreSQL 17
Port: 5400 (mapped from container port 5432)
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
mise run report
```

### Step-by-Step Guide

#### 1. Start PostgreSQL

```bash
mise run postgres
```

This starts PostgreSQL 17 in a Docker container on port 5400.

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

Results are saved to `results/query/*.json`

#### 6. Generate Report

```bash
mise run report
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
│   └── ore.rs            # ORE range query benchmarks
├── src/
│   ├── bin/              # Binary utilities
│   │   ├── encrypt_int.rs
│   │   ├── encrypt_string.rs
│   │   └── combine_benchmark.rs
│   └── lib.rs            # Shared benchmark code
├── sql/
│   ├── schema.sql        # Database schema
│   └── indexes/          # Index creation scripts
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
mise run report custom_report.md

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
# Check if PostgreSQL is running
docker ps | grep postgres

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
