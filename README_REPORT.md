# Benchmark Report Generator

This project includes a Python script to generate comprehensive reports from benchmark results.

## Quick Start

Generate a report from existing benchmark results:

```bash
mise run report
```

This will create `report/BENCHMARK_REPORT.md` and associated chart images in the `report/` directory.

## Usage

### Via Mise Task

```bash
# Generate report with default location (report/BENCHMARK_REPORT.md)
mise run report

# Generate report with custom location
mise run report custom_dir/my_report.md
```

### Direct Script Usage

```bash
# Basic usage (creates report/BENCHMARK_REPORT.md)
python3 report_benchmarks.py

# Custom output file
python3 report_benchmarks.py --output custom_dir/report.md

# Custom results directory and SQL directory
python3 report_benchmarks.py --results-dir path/to/results --sql-dir path/to/sql --output report/report.md

# Show help
python3 report_benchmarks.py --help
```

## Report Contents

The generated report includes:

### 1. Ingest Throughput

Performance metrics for inserting encrypted records:
- **Int**: Encrypted integer values
- **JSON Small**: Small encrypted JSON objects  
- **String**: Encrypted string values

For each type, the report shows:
- Number of records
- Throughput (records/sec)
- Total time
- Average memory usage

### 2. Query Performance

Performance metrics for queries across different data set sizes (10K, 100K, 1M rows):

#### EXACT Queries
- **eql_cast**: Exact match using EQL cast operator
  - SQL: `SELECT value FROM {TABLE} WHERE value = $1 LIMIT 1`
  - Parameter: `Bob Johnson`
- **eql_hash**: Exact match using EQL HMAC-256 hash function
  - SQL: `SELECT value FROM {TABLE} WHERE eql_v2.hmac_256(value) = eql_v2.hmac_256($1::jsonb) LIMIT 1`
  - Parameter: `Bob Johnson`

#### MATCH Queries
- **eql_cast_firstname**: Pattern matching on first name using LIKE
  - SQL: `SELECT id,value::jsonb FROM {TABLE} WHERE value LIKE $1 LIMIT 10`
  - Parameter: `Bob`
- **eql_cast_lastname**: Pattern matching on last name using LIKE
  - SQL: `SELECT id,value::jsonb FROM {TABLE} WHERE value LIKE $1 LIMIT 10`
  - Parameter: `Johnson`
- **eql_bloom**: Pattern matching using bloom filter containment
  - SQL: `SELECT id,value::jsonb FROM {TABLE} WHERE eql_v2.bloom_filter(value) @> eql_v2.bloom_filter($1) LIMIT 10`
  - Parameter: `Johnson`

#### ORE Queries
- **exact**: Exact match on encrypted integer
  - SQL: `SELECT value FROM {TABLE} WHERE value = $1 LIMIT 1`
  - Parameter: `5000`
- **range_gt_10**: Range query (>) returning 10 results
  - SQL: `SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 10`
  - Parameter: `5000`
- **range_gt_100**: Range query (>) returning 100 results
  - SQL: `SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 100`
  - Parameter: `5000`
- **range_lt_10**: Range query (<) returning 10 results
  - SQL: `SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 LIMIT 10`
  - Parameter: `5000`
- **range_lt_100**: Range query (<) returning 100 results
  - SQL: `SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 LIMIT 100`
  - Parameter: `5000`
- **range_lt_ordered_10**: Ordered range query with ORDER BY
  - SQL: `SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 ORDER BY value LIMIT 10`
  - Parameter: `5000`

Each query shows:
- The actual SQL query used
- The parameter value passed to the query
- Database indexes available on the table
- Performance both with and without decryption of results
- ⚠️ Warning indicator for query times exceeding 100ms

## Charts (Optional)

The script can generate PNG charts for visualizing benchmark results. To enable charts:

```bash
pip3 install matplotlib
```

When matplotlib is available:
- Ingest benchmarks will include bar charts showing throughput
- Query benchmarks will include line charts showing performance vs data set size

Charts are saved as PNG files in the same directory as the report (by default, the `report/` directory).

## Report Format

The report is generated as a Markdown file with:
- Table of contents with anchor links
- Tables showing performance metrics
- SQL queries and parameters for each benchmark
- Database index definitions (read from `sql/indexes/` directory)
- Performance indicators (⚠️ emoji) for query times exceeding 100ms
- Embedded chart images (when matplotlib is available)
- Descriptions of what each benchmark tests

## Input Files

The script expects benchmark results and SQL files in the following structure:

```
results/
├── ingest/
│   ├── encrypt_int_combined.json
│   ├── encrypt_json_small_combined.json
│   └── encrypt_string_combined.json
└── query/
    ├── exact_rows_10000.json
    ├── exact_rows_100000.json
    ├── exact_rows_1000000.json
    ├── match_rows_10000.json
    ├── match_rows_100000.json
    ├── match_rows_1000000.json
    ├── ore_rows_10000.json
    ├── ore_rows_100000.json
    └── ore_rows_1000000.json

sql/
├── schema.sql
└── indexes/
    ├── string_encrypted_10000_up.sql
    ├── string_encrypted_100000_up.sql
    ├── string_encrypted_1000000_up.sql
    ├── integer_encrypted_10000_up.sql
    ├── integer_encrypted_100000_up.sql
    └── integer_encrypted_1000000_up.sql
```

### Ingest Result Format

Ingest results should be JSON files with:
```json
{
  "metadata": { ... },
  "results": [
    {
      "num_records": 500,
      "total_time_seconds": 0.91,
      "throughput_records_per_second": 544.83,
      "average_memory_usage_mb": 15.25
    }
  ]
}
```

### Query Result Format

Query results should be Criterion JSON output (one JSON object per line) with benchmark results.

### Index SQL Files

Index definitions should be in `sql/indexes/{table_name}_up.sql` files. For example:
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

CREATE INDEX
string_encrypted_10000_eql_index
ON string_encrypted_10000 (
    value eql_v2.encrypted_operator_class
);
```

## Customization

The script can be modified to:
- Add new query descriptions
- Change chart formatting
- Modify table layouts
- Add additional metrics
- Change output format (e.g., HTML, PDF)

See the `BenchmarkReporter` class in `report_benchmarks.py` for customization points.

## Requirements

- Python 3.7+
- matplotlib (optional, for charts)

## Why Python?

Python was chosen for this script because it provides:
- Simple JSON parsing with the standard library
- Flexible text/markdown generation
- Rich ecosystem of charting libraries (matplotlib, plotly, etc.)
- Easy extensibility for different output formats
- Cross-platform compatibility

This makes it easier to add features like:
- Interactive HTML reports
- PDF generation
- Statistical analysis
- Different visualization styles
- Export to other formats (CSV, Excel, etc.)

## Output Directory

By default, reports are generated in the `report/` directory. This keeps generated files organized and separate from source code. The generated files in `report/` are tracked in git to provide a historical record of benchmark performance.
