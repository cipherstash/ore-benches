#!/usr/bin/env python3
"""
Benchmark Report Generator

Generates a comprehensive report from benchmark results including:
- Ingest throughput for int, json_small, and string tests
- Query performance charts across different data set sizes
"""

import json
import sys
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
import argparse
import re

try:
    import matplotlib.pyplot as plt
    import matplotlib
    matplotlib.use('Agg')  # Non-interactive backend
    HAS_MATPLOTLIB = True
except ImportError:
    HAS_MATPLOTLIB = False
    print("Warning: matplotlib not available. Charts will be skipped.", file=sys.stderr)
    print("Install with: pip3 install matplotlib", file=sys.stderr)


@dataclass
class IngestResult:
    """Results from an ingest benchmark"""
    bench_type: str
    num_records: int
    throughput: float
    total_time: float
    avg_memory_mb: float


@dataclass
class QueryResult:
    """Results from a query benchmark"""
    query_type: str  # e.g., "EXACT", "MATCH", "ORE"
    query_name: str  # e.g., "eql_cast", "range_gt_10"
    row_count: int
    decrypt: bool
    mean_ns: float
    median_ns: float


class BenchmarkReporter:
    def __init__(self, results_dir: Path, output_file: Path, sql_dir: Optional[Path] = None):
        self.results_dir = results_dir
        self.output_file = output_file
        self.sql_dir = sql_dir or Path("sql")
        self.ingest_results: List[IngestResult] = []
        self.query_results: List[QueryResult] = []
        self.index_cache: Dict[str, str] = {}  # Cache for index SQL

    def load_ingest_results(self):
        """Load ingest benchmark results"""
        ingest_dir = self.results_dir / "ingest"
        
        for bench_type in ["int", "json_small", "string"]:
            file_path = ingest_dir / f"encrypt_{bench_type}_combined.json"
            
            if not file_path.exists():
                print(f"Warning: {file_path} not found, skipping", file=sys.stderr)
                continue
            
            with open(file_path) as f:
                data = json.load(f)
            
            for result in data.get("results", []):
                self.ingest_results.append(IngestResult(
                    bench_type=bench_type,
                    num_records=result["num_records"],
                    throughput=result["throughput_records_per_second"],
                    total_time=result["total_time_seconds"],
                    avg_memory_mb=result["average_memory_usage_mb"]
                ))

    def load_query_results(self):
        """Load query benchmark results from criterion JSON output"""
        query_dir = self.results_dir / "query"
        
        for json_file in query_dir.glob("*.json"):
            # Parse filename: {query_type}_rows_{count}.json
            parts = json_file.stem.split("_rows_")
            if len(parts) != 2:
                continue
            
            query_type = parts[0].upper()  # EXACT, MATCH, ORE
            row_count = int(parts[1])
            
            with open(json_file) as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    
                    try:
                        data = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    
                    if data.get("reason") != "benchmark-complete":
                        continue
                    
                    # Parse benchmark ID: "QUERY_TYPE/query_variant/scenario/rows"
                    bench_id = data.get("id", "")
                    parts = bench_id.split("/")
                    
                    if len(parts) < 3:
                        continue
                    
                    # Determine if this is a decrypt variant
                    decrypt = "decrypt" in parts[1]
                    
                    # Get scenario name (e.g., "eql_cast", "range_gt_10")
                    scenario = parts[2]
                    
                    # Extract mean timing
                    mean_ns = data.get("mean", {}).get("estimate", 0)
                    median_ns = data.get("median", {}).get("estimate", 0)
                    
                    self.query_results.append(QueryResult(
                        query_type=query_type,
                        query_name=scenario,
                        row_count=row_count,
                        decrypt=decrypt,
                        mean_ns=mean_ns,
                        median_ns=median_ns
                    ))

    def format_time(self, ns: float, include_indicator: bool = True) -> str:
        """Format nanoseconds into human-readable time with performance indicator
        
        Args:
            ns: Time in nanoseconds
            include_indicator: If True, adds emoji indicator for times > 100ms
        """
        # Convert to milliseconds for threshold check
        ms = ns / 1_000_000
        
        # Format the time
        if ns >= 1_000_000_000:
            formatted = f"{ns / 1_000_000_000:.3f}s"
        elif ns >= 1_000_000:
            formatted = f"{ns / 1_000_000:.2f}ms"
        elif ns >= 1_000:
            formatted = f"{ns / 1_000:.2f}μs"
        else:
            formatted = f"{ns:.0f}ns"
        
        # Add indicator if time exceeds 100ms
        if include_indicator and ms > 100:
            return f"⚠️ {formatted}"
        
        return formatted

    def format_throughput(self, throughput: float) -> str:
        """Format throughput with appropriate units"""
        if throughput >= 1_000_000:
            return f"{throughput / 1_000_000:.2f}M"
        elif throughput >= 1_000:
            return f"{throughput / 1_000:.2f}K"
        else:
            return f"{throughput:.2f}"

    def get_query_sql_and_param(self, query_type: str, query_name: str) -> Tuple[str, str]:
        """Get the SQL query template and parameter value for a query"""
        # Map from bench files
        sql_map = {
            "EXACT": {
                "eql_cast": (
                    "SELECT value FROM {TABLE} WHERE value = $1 LIMIT 1",
                    "Bob Johnson"
                ),
                "eql_hash": (
                    "SELECT value FROM {TABLE} WHERE eql_v2.hmac_256(value) = eql_v2.hmac_256($1::jsonb) LIMIT 1",
                    "Bob Johnson"
                )
            },
            "MATCH": {
                "eql_cast_firstname": (
                    "SELECT id,value::jsonb FROM {TABLE} WHERE value LIKE $1 LIMIT 10",
                    "Bob"
                ),
                "eql_cast_lastname": (
                    "SELECT id,value::jsonb FROM {TABLE} WHERE value LIKE $1 LIMIT 10",
                    "Johnson"
                ),
                "eql_bloom": (
                    "SELECT id,value::jsonb FROM {TABLE} WHERE eql_v2.bloom_filter(value) @> eql_v2.bloom_filter($1) LIMIT 10",
                    "Johnson"
                )
            },
            "ORE": {
                "exact": (
                    "SELECT value FROM {TABLE} WHERE value = $1 LIMIT 1",
                    "5000"
                ),
                "range_gt_10": (
                    "SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 10",
                    "5000"
                ),
                "range_gt_100": (
                    "SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 100",
                    "5000"
                ),
                "range_lt_10": (
                    "SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 LIMIT 10",
                    "5000"
                ),
                "range_lt_100": (
                    "SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 LIMIT 100",
                    "5000"
                ),
                "range_lt_ordered_10": (
                    "SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 ORDER BY value LIMIT 10",
                    "5000"
                )
            }
        }
        
        return sql_map.get(query_type, {}).get(query_name, ("", ""))

    def get_query_description(self, query_type: str, query_name: str) -> Tuple[str, str]:
        """Get description and table info for a query"""
        descriptions = {
            "EXACT": {
                "eql_cast": (
                    "Exact match using EQL cast operator",
                    "Table: `string_encrypted_{rows}` with encrypted string values. "
                    "Index: UNIQUE index on the encrypted value column."
                ),
                "eql_hash": (
                    "Exact match using EQL HMAC-256 hash function",
                    "Table: `string_encrypted_{rows}` with encrypted string values. "
                    "Index: Hash-based unique index using `eql_v2.hmac_256`."
                )
            },
            "MATCH": {
                "eql_cast_firstname": (
                    "Pattern matching on first name using EQL cast and LIKE",
                    "Table: `string_encrypted_{rows}` with encrypted string values. "
                    "Index: MATCH index for substring searches. "
                    "Query returns LIMIT 10 results."
                ),
                "eql_cast_lastname": (
                    "Pattern matching on last name using EQL cast and LIKE",
                    "Table: `string_encrypted_{rows}` with encrypted string values. "
                    "Index: MATCH index for substring searches. "
                    "Query returns LIMIT 10 results."
                ),
                "eql_bloom": (
                    "Pattern matching using EQL bloom filter containment",
                    "Table: `string_encrypted_{rows}` with encrypted string values. "
                    "Index: Bloom filter index using `eql_v2.bloom_filter`. "
                    "Query returns LIMIT 10 results."
                )
            },
            "ORE": {
                "exact": (
                    "Exact match query on encrypted integer",
                    "Table: `integer_encrypted_{rows}` with ORE-encrypted integer values. "
                    "Index: ORE index supporting equality and range queries. "
                    "Query returns LIMIT 1 result."
                ),
                "range_gt_10": (
                    "Range query (greater than) returning 10 results",
                    "Table: `integer_encrypted_{rows}` with ORE-encrypted integer values. "
                    "Index: ORE index supporting equality and range queries. "
                    "Query: WHERE value > 5000 LIMIT 10."
                ),
                "range_gt_100": (
                    "Range query (greater than) returning 100 results",
                    "Table: `integer_encrypted_{rows}` with ORE-encrypted integer values. "
                    "Index: ORE index supporting equality and range queries. "
                    "Query: WHERE value > 5000 LIMIT 100."
                ),
                "range_lt_10": (
                    "Range query (less than) returning 10 results",
                    "Table: `integer_encrypted_{rows}` with ORE-encrypted integer values. "
                    "Index: ORE index supporting equality and range queries. "
                    "Query: WHERE value < 5000 LIMIT 10."
                ),
                "range_lt_100": (
                    "Range query (less than) returning 100 results",
                    "Table: `integer_encrypted_{rows}` with ORE-encrypted integer values. "
                    "Index: ORE index supporting equality and range queries. "
                    "Query: WHERE value < 5000 LIMIT 100."
                ),
                "range_lt_ordered_10": (
                    "Ordered range query (less than) with ORDER BY",
                    "Table: `integer_encrypted_{rows}` with ORE-encrypted integer values. "
                    "Index: ORE index supporting equality and range queries. "
                    "Query: WHERE value < 5000 ORDER BY value LIMIT 10."
                )
            }
        }
        
        return descriptions.get(query_type, {}).get(query_name, ("Unknown query", ""))

    def get_table_indexes(self, table_name: str) -> Optional[str]:
        """Get index SQL for a table by reading from sql/indexes directory"""
        # Check cache first
        if table_name in self.index_cache:
            return self.index_cache[table_name]
        
        # Try to find the index file
        index_file = self.sql_dir / "indexes" / f"{table_name}_up.sql"
        
        if not index_file.exists():
            # Try without row count suffix (base table)
            # e.g., string_encrypted_10000 -> string_encrypted
            base_table = re.sub(r'_(\d+)$', '', table_name)
            index_file = self.sql_dir / "indexes" / f"{base_table}_up.sql"
        
        if not index_file.exists():
            return None
        
        try:
            with open(index_file) as f:
                content = f.read().strip()
                self.index_cache[table_name] = content
                return content
        except Exception as e:
            print(f"Warning: Could not read index file {index_file}: {e}", file=sys.stderr)
            return None

    def generate_report(self):
        """Generate the full benchmark report"""
        with open(self.output_file, 'w') as f:
            self._write_header(f)
            self._write_ingest_section(f)
            self._write_query_sections(f)
            self._write_footer(f)

    def _write_header(self, f):
        f.write("# Benchmark Report\n\n")
        f.write("This report summarizes the performance benchmarks for encrypted database operations.\n\n")
        f.write("## Table of Contents\n\n")
        f.write("1. [Ingest Throughput](#ingest-throughput)\n")
        f.write("2. [Query Performance](#query-performance)\n")
        
        # Add subsections for each query type
        query_types = set(r.query_type for r in self.query_results)
        for qt in sorted(query_types):
            f.write(f"   - [{qt} Queries](#{qt.lower()}-queries)\n")
        
        f.write("\n---\n\n")

    def _write_ingest_section(self, f):
        f.write("## Ingest Throughput\n\n")
        f.write("This section measures the throughput of inserting encrypted records into the database.\n\n")
        
        # Group by bench_type
        types = ["int", "json_small", "string"]
        
        for bench_type in types:
            results = [r for r in self.ingest_results if r.bench_type == bench_type]
            if not results:
                continue
            
            # Sort by num_records
            results.sort(key=lambda x: x.num_records)
            
            f.write(f"### {bench_type.replace('_', ' ').title()}\n\n")
            
            if bench_type == "int":
                f.write("Tests insertion of encrypted integer values.\n\n")
            elif bench_type == "json_small":
                f.write("Tests insertion of small encrypted JSON objects.\n\n")
            elif bench_type == "string":
                f.write("Tests insertion of encrypted string values.\n\n")
            
            # Table
            f.write("| Records | Throughput (records/sec) | Total Time | Avg Memory |\n")
            f.write("|---------|--------------------------|------------|------------|\n")
            
            for r in results:
                f.write(f"| {r.num_records:,} | {self.format_throughput(r.throughput)} | "
                       f"{r.total_time:.2f}s | {r.avg_memory_mb:.2f} MB |\n")
            
            f.write("\n")
            
            # Generate chart if matplotlib is available
            if HAS_MATPLOTLIB:
                chart_path = self.output_file.parent / f"ingest_{bench_type}_chart.png"
                self._create_ingest_chart(results, bench_type, chart_path)
                f.write(f"![Ingest Throughput - {bench_type}]({chart_path.name})\n\n")

    def _create_ingest_chart(self, results: List[IngestResult], bench_type: str, output_path: Path):
        """Create a bar chart for ingest throughput"""
        fig, ax = plt.subplots(figsize=(10, 6))
        
        records = [r.num_records for r in results]
        throughput = [r.throughput for r in results]
        
        ax.bar(range(len(records)), throughput, color='steelblue')
        ax.set_xlabel('Number of Records')
        ax.set_ylabel('Throughput (records/sec)')
        ax.set_title(f'Ingest Throughput - {bench_type.replace("_", " ").title()}')
        ax.set_xticks(range(len(records)))
        ax.set_xticklabels([f"{r:,}" for r in records])
        ax.grid(axis='y', alpha=0.3)
        
        plt.tight_layout()
        plt.savefig(output_path, dpi=100, bbox_inches='tight')
        plt.close()

    def _write_query_sections(self, f):
        f.write("## Query Performance\n\n")
        f.write("This section measures query performance across different data set sizes. "
               "Each query is tested with and without decryption of results.\n\n")
        
        # Group by query type (EXACT, MATCH, ORE)
        query_types = sorted(set(r.query_type for r in self.query_results))
        
        for query_type in query_types:
            self._write_query_type_section(f, query_type)

    def _write_query_type_section(self, f, query_type: str):
        f.write(f"### {query_type} Queries\n\n")
        
        # Get all unique query names for this type
        type_results = [r for r in self.query_results if r.query_type == query_type]
        query_names = sorted(set(r.query_name for r in type_results))
        
        for query_name in query_names:
            self._write_query_subsection(f, query_type, query_name)

    def _write_query_subsection(self, f, query_type: str, query_name: str):
        # Get results for this specific query
        results = [r for r in self.query_results 
                  if r.query_type == query_type and r.query_name == query_name]
        
        if not results:
            return
        
        # Sort by row count
        results.sort(key=lambda x: (x.row_count, x.decrypt))
        
        # Get description
        description, table_info = self.get_query_description(query_type, query_name)
        sql_query, param = self.get_query_sql_and_param(query_type, query_name)
        
        f.write(f"#### {query_name}\n\n")
        f.write(f"**Description:** {description}\n\n")
        
        # Add SQL query and parameter
        if sql_query:
            f.write(f"**SQL Query:**\n```sql\n{sql_query}\n```\n\n")
            f.write(f"**Parameter:** `{param}`\n\n")
        
        f.write(f"**{table_info}**\n\n")
        
        # Add index information for one of the row counts (they all use same indexes)
        if results:
            # Determine table name based on query type
            sample_row_count = results[0].row_count
            if query_type in ["EXACT", "MATCH"]:
                table_name = f"string_encrypted_{sample_row_count}"
            elif query_type == "ORE":
                table_name = f"integer_encrypted_{sample_row_count}"
            else:
                table_name = ""
            
            if table_name:
                indexes_sql = self.get_table_indexes(table_name)
                if indexes_sql:
                    f.write(f"**Indexes:**\n```sql\n{indexes_sql}\n```\n\n")
        
        # Create table with legend if any results exceed 100ms
        has_slow_queries = any((r.mean_ns / 1_000_000) > 100 for r in results)
        
        if has_slow_queries:
            f.write("*⚠️ = Query time exceeds 100ms*\n\n")
        
        f.write("| Data Set Size | Query Time (no decrypt) | Query Time (with decrypt) |\n")
        f.write("|---------------|-------------------------|---------------------------|\n")
        
        # Group by row_count
        row_counts = sorted(set(r.row_count for r in results))
        
        for row_count in row_counts:
            no_decrypt = next((r for r in results if r.row_count == row_count and not r.decrypt), None)
            with_decrypt = next((r for r in results if r.row_count == row_count and r.decrypt), None)
            
            no_decrypt_str = self.format_time(no_decrypt.mean_ns) if no_decrypt else "N/A"
            with_decrypt_str = self.format_time(with_decrypt.mean_ns) if with_decrypt else "N/A"
            
            f.write(f"| {row_count:,} | {no_decrypt_str} | {with_decrypt_str} |\n")
        
        f.write("\n")
        
        # Generate chart if matplotlib is available
        if HAS_MATPLOTLIB and len(row_counts) > 1:
            chart_path = self.output_file.parent / f"query_{query_type.lower()}_{query_name}_chart.png"
            self._create_query_chart(results, query_type, query_name, chart_path)
            f.write(f"![Query Performance - {query_type}/{query_name}]({chart_path.name})\n\n")

    def _create_query_chart(self, results: List[QueryResult], query_type: str, 
                           query_name: str, output_path: Path):
        """Create a line chart for query performance"""
        fig, ax = plt.subplots(figsize=(12, 6))
        
        # Separate no-decrypt and with-decrypt results
        row_counts = sorted(set(r.row_count for r in results))
        
        no_decrypt_times = []
        with_decrypt_times = []
        
        for row_count in row_counts:
            no_decrypt = next((r for r in results if r.row_count == row_count and not r.decrypt), None)
            with_decrypt = next((r for r in results if r.row_count == row_count and r.decrypt), None)
            
            # Convert to milliseconds for better readability
            no_decrypt_times.append(no_decrypt.mean_ns / 1_000_000 if no_decrypt else None)
            with_decrypt_times.append(with_decrypt.mean_ns / 1_000_000 if with_decrypt else None)
        
        # Plot lines
        if any(t is not None for t in no_decrypt_times):
            ax.plot(row_counts, no_decrypt_times, marker='o', label='Without Decryption', linewidth=2)
        
        if any(t is not None for t in with_decrypt_times):
            ax.plot(row_counts, with_decrypt_times, marker='s', label='With Decryption', linewidth=2)
        
        ax.set_xlabel('Data Set Size (rows)', fontsize=12)
        ax.set_ylabel('Query Time (ms)', fontsize=12)
        ax.set_title(f'{query_type} - {query_name}', fontsize=14, fontweight='bold')
        ax.set_xscale('log')
        ax.grid(True, alpha=0.3)
        ax.legend(fontsize=10)
        
        # Format x-axis labels
        ax.set_xticks(row_counts)
        ax.set_xticklabels([f"{r:,}" for r in row_counts])
        
        plt.tight_layout()
        plt.savefig(output_path, dpi=100, bbox_inches='tight')
        plt.close()

    def _write_footer(self, f):
        f.write("\n---\n\n")
        f.write("*Report generated by `report_benchmarks.py`*\n")


def main():
    parser = argparse.ArgumentParser(description="Generate benchmark report")
    parser.add_argument("--results-dir", type=Path, default=Path("results"),
                       help="Directory containing benchmark results (default: results)")
    parser.add_argument("--sql-dir", type=Path, default=Path("sql"),
                       help="Directory containing SQL schema and index files (default: sql)")
    parser.add_argument("--output", "-o", type=Path, default=Path("report/BENCHMARK_REPORT.md"),
                       help="Output file path (default: report/BENCHMARK_REPORT.md)")
    
    args = parser.parse_args()
    
    if not args.results_dir.exists():
        print(f"Error: Results directory '{args.results_dir}' does not exist", file=sys.stderr)
        sys.exit(1)
    
    # Create output directory if it doesn't exist
    args.output.parent.mkdir(parents=True, exist_ok=True)
    
    reporter = BenchmarkReporter(args.results_dir, args.output, args.sql_dir)
    
    print("Loading ingest results...")
    reporter.load_ingest_results()
    print(f"  Found {len(reporter.ingest_results)} ingest results")
    
    print("Loading query results...")
    reporter.load_query_results()
    print(f"  Found {len(reporter.query_results)} query results")
    
    print(f"Generating report: {args.output}")
    reporter.generate_report()
    
    print(f"\n✓ Report generated successfully: {args.output}")
    
    if not HAS_MATPLOTLIB:
        print("\nNote: Charts were not generated. Install matplotlib to enable charts:")
        print("  pip3 install matplotlib")


if __name__ == "__main__":
    main()
