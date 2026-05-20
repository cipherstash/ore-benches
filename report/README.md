# Benchmark Reports

This directory contains generated benchmark reports and associated charts.

## Files

- `BENCHMARK_REPORT.md` - Main benchmark report (generated)
- `ingest_*_chart.png` - Ingest throughput charts (generated, requires matplotlib)
- `query_*_chart.png` - Query performance charts (generated, requires matplotlib)

## Generating Reports

To generate or update the report:

```bash
mise run report:build
```

This will create/update `BENCHMARK_REPORT.md` and associated chart images in this directory.

For a quick terminal overview of every scenario's median runtime (no file
generated), run `mise run report`.

## Version Control

These generated files are tracked in git to provide a historical record of benchmark performance over time.
