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


@dataclass
class ScenarioMetadata:
    """Per-scenario metadata captured at bench startup (lib.rs sidecar).

    The bench writes one of these per (scenario, row_count) tier into
    `results/query/<prefix>_metadata_<rows>.json` alongside the criterion
    timing data. We use it to enrich the report with the actual SQL ran,
    which indexes the planner picked, and what the plan looked like.
    """
    query_type: str       # e.g. "EXACT"
    query_name: str       # e.g. "eql_cast"
    row_count: int
    query: str            # SQL with $N placeholders intact
    parameters: list      # bound values as JSON (encrypted payload for EQL scenarios)
    explain: list         # PG's EXPLAIN (FORMAT JSON) output (top-level array)
    indexes_used: List[str]
    # Actual row count from a one-shot pre-bench execution. None when the
    # sidecar predates the actual-rows capture and we're falling back to
    # the planner estimate from `explain[0]["Plan"]["Plan Rows"]`.
    rows_returned: Optional[int] = None


class BenchmarkReporter:
    def __init__(self, results_dir: Path, output_file: Path, sql_dir: Optional[Path] = None):
        self.results_dir = results_dir
        self.output_file = output_file
        self.sql_dir = sql_dir or Path("sql")
        self.ingest_results: List[IngestResult] = []
        self.query_results: List[QueryResult] = []
        # Keyed by (query_type, query_name, row_count) so a scenario can
        # look up its own per-tier metadata in O(1).
        self.metadata: Dict[Tuple[str, str, int], ScenarioMetadata] = {}
        self.index_cache: Dict[str, str] = {}  # Cache for index SQL

    def load_ingest_results(self):
        """Load ingest benchmark results"""
        ingest_dir = self.results_dir / "ingest"
        
        for bench_type in ["int", "json_small", "json_large", "string", "ste_vec_small", "ste_vec_large"]:
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

    def load_query_metadata(self):
        """Load `*_metadata_*.json` sidecars written by each bench at startup."""
        query_dir = self.results_dir / "query"
        meta_pattern = re.compile(r'^(.+)_metadata_(\d+)$')
        for json_file in query_dir.glob("*_metadata_*.json"):
            m = meta_pattern.match(json_file.stem)
            if not m:
                continue
            # The prefix in the filename (e.g. "exact", "group_by") drives the
            # criterion bench name, not the query_type — we get the real
            # query_type from the bench id inside each scenario record.
            row_count = int(m.group(2))
            with open(json_file) as f:
                doc = json.load(f)
            for s in doc.get("scenarios", []):
                bench_id = s.get("id", "")
                parts = bench_id.split("/")
                if len(parts) < 3:
                    continue
                query_type = parts[0]      # "EXACT", "ORE", "GROUP_BY", ...
                scenario_name = parts[2]   # "eql_cast", "range_gt_10", ...
                key = (query_type, scenario_name, row_count)
                self.metadata[key] = ScenarioMetadata(
                    query_type=query_type,
                    query_name=scenario_name,
                    row_count=row_count,
                    query=s.get("query", ""),
                    parameters=s.get("parameters", []),
                    explain=s.get("explain", []),
                    indexes_used=s.get("indexes_used", []),
                    rows_returned=s.get("rows_returned"),
                )

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
                "range_lt_hybrid_ordered_10": (
                    "SELECT id,value::jsonb FROM {TABLE} "
                    "WHERE value < $1 "
                    "ORDER BY eql_v2.ore_block_u64_8_256(value) LIMIT 10",
                    "5000"
                )
            },
            "GROUP_BY": {
                "count_groups_encrypted": (
                    "SELECT count(*) FROM "
                    "(SELECT 1 FROM {TABLE} GROUP BY eql_v2.hmac_256(value)) g",
                    ""
                ),
                "count_groups_plaintext": (
                    "SELECT count(*) FROM "
                    "(SELECT 1 FROM {TABLE} GROUP BY value) g",
                    ""
                )
            },
            "JSON": {
                "field_eq": (
                    "SELECT id FROM {TABLE} "
                    "WHERE eql_v2.hmac_256_terms(value) @> $1::jsonb "
                    "LIMIT 10",
                    "[{\"s\":\"<selector-hash>\",\"hm\":\"<hmac>\"}]"
                ),
                "field_extract": (
                    "SELECT eql_v2.jsonb_path_query(value, '<selector-hash>') "
                    "FROM {TABLE} LIMIT 1000",
                    ""
                ),
                "field_group_by": (
                    "SELECT eql_v2.hmac_256(value, '<selector-hash>'), count(*) "
                    "FROM {TABLE} GROUP BY 1",
                    ""
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
                "range_gt_10": (
                    "Range query (greater than) returning 10 results",
                    "Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. "
                    "Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. "
                    "The bare-form `<` / `>` operators inline to "
                    "`eql_v2.ore_block_u64_8_256(a) op eql_v2.ore_block_u64_8_256(b)` "
                    "post-2.3, so the index engages without query rewriting. "
                    "Query: WHERE value > 5000 LIMIT 10."
                ),
                "range_gt_100": (
                    "Range query (greater than) returning 100 results",
                    "Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. "
                    "Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. "
                    "Query: WHERE value > 5000 LIMIT 100."
                ),
                "range_lt_10": (
                    "Range query (less than) returning 10 results",
                    "Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. "
                    "Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. "
                    "Query: WHERE value < 5000 LIMIT 10."
                ),
                "range_lt_100": (
                    "Range query (less than) returning 100 results",
                    "Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. "
                    "Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. "
                    "Query: WHERE value < 5000 LIMIT 100."
                ),
                "range_lt_hybrid_ordered_10": (
                    "Ordered range query (hybrid form: natural WHERE, extractor ORDER BY)",
                    "Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. "
                    "Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. "
                    "Query: WHERE value < 5000 ORDER BY eql_v2.ore_block_u64_8_256(value) LIMIT 10. "
                    "The sort key matches the index expression syntactically, so rows stream "
                    "out of the index already ordered — no Sort node. See §4 of the EQL "
                    "query-performance guide for the natural-form sort-key trap that this "
                    "shape avoids."
                )
            },
            "GROUP_BY": {
                "count_groups_encrypted": (
                    "GROUP BY in extractor form on `eql_v2.hmac_256(value)`, "
                    "wrapped in `count(*)` to isolate aggregation cost from emit cost",
                    "Table: `string_encrypted_{rows}` with encrypted string values "
                    "(carrying an `hm` HMAC term, configured via the `unique` search index). "
                    "Index: no index drives `GROUP BY` directly — hash aggregation is "
                    "in-memory. The extractor's 32-byte HMAC group key fits in default "
                    "`work_mem`, so the planner picks `HashAggregate` reliably across "
                    "deployments. **Why the subquery wrapper.** The bench data is "
                    "`fake::name::Name<EN>` — effectively unique per row, so a bare "
                    "`SELECT count(*) FROM tbl GROUP BY eql_v2.hmac_256(value)` emits ~one "
                    "row per input row. Wall-clock time on that shape is dominated by result "
                    "emission (server-side row construction, network round-trip, sqlx "
                    "deserialisation, bench iter-and-sum), not by the aggregation work the "
                    "recipe is actually about. Wrapping the GROUP BY in `count(*)` keeps the "
                    "inner HashAggregate identical but emits a single row, so the bench "
                    "measures aggregation cost. The companion `count_groups_plaintext` "
                    "scenario runs the same query shape against an unencrypted column for "
                    "comparison. Natural-form `GROUP BY value` against an encrypted column "
                    "was removed from this bench in an earlier pass because the planner picks "
                    "`GroupAggregate` + sort against the full ~1-2 KB ciphertext payload at "
                    "scale — see §5 of the EQL query-performance guide."
                ),
                "count_groups_plaintext": (
                    "Plaintext baseline: GROUP BY on a plain TEXT column, same query shape "
                    "as the encrypted scenario",
                    "Table: `string_plaintext_{rows}` with unencrypted high-cardinality "
                    "random strings (`md5(random()::text || ordinal)`). Populated via SQL "
                    "by `mise run prepare:string_plaintext` — no encryption-client "
                    "dependency. Index: none. Same `SELECT count(*) FROM (SELECT 1 ... "
                    "GROUP BY value) g` shape as the encrypted scenario, so the wall-clock "
                    "delta between this and `count_groups_encrypted` is the EQL recipe's "
                    "overhead relative to a bare-PG aggregate on a TEXT column at the same "
                    "row count and cardinality."
                )
            },
            "JSON": {
                "field_eq": (
                    "Field-level equality on an ste_vec document via `hmac_256_terms`",
                    "Table: `json_ste_vec_small_encrypted_{rows}` with encrypted JSON "
                    "documents (small four-field shape — first_name / last_name / age / email). "
                    "Index: functional GIN on `eql_v2.hmac_256_terms(value)`. One index covers "
                    "field-level equality across every selector that carries `hm`, vs the "
                    "per-selector `hash (eql_v2.hmac_256(col, '<selector>'))` recipe which "
                    "needs one index per hot path. The bench picks a (selector, hmac) pair "
                    "from `sv[0]` of a sample row at startup; the query body matches the "
                    "documented EQL recipe."
                ),
                "field_extract": (
                    "Sequential field extraction via `eql_v2.jsonb_path_query`",
                    "Table: `json_ste_vec_small_encrypted_{rows}`. No index — measures the "
                    "per-row cost of the inlinable `jsonb_path_query` body "
                    "(`jsonb_array_elements((val).data -> 'sv') WHERE elem ->> 's' = selector`). "
                    "Inlining means the body folds into the calling query, so each row pays "
                    "an array walk rather than a plpgsql function call. Query: "
                    "`SELECT eql_v2.jsonb_path_query(value, '<selector>') FROM tbl LIMIT 1000`."
                ),
                "field_group_by": (
                    "Field-level `GROUP BY` on an ste_vec document",
                    "Table: `json_ste_vec_small_encrypted_{rows}`. No index — HashAggregate "
                    "is in-memory. Query: "
                    "`GROUP BY eql_v2.hmac_256(value, '<selector>')`. Same extractor-form "
                    "recipe as the top-level GROUP_BY bench, scaled to a single field "
                    "inside an ste_vec doc."
                )
            }
        }

        return descriptions.get(query_type, {}).get(query_name, ("Unknown query", ""))

    def planner_estimated_rows(self, explain: list) -> Optional[int]:
        """Pull the top-level Plan's `Plan Rows` from an EXPLAIN (FORMAT JSON)
        result. That's the planner's row-count estimate for the final output;
        for LIMIT-bounded queries it matches the LIMIT, for aggregates it's
        the estimated group count. Returns None on malformed input."""
        if not explain:
            return None
        try:
            return int(explain[0]["Plan"]["Plan Rows"])
        except (KeyError, TypeError, IndexError, ValueError):
            return None

    def format_plan_tree(self, plan_node: dict, depth: int = 0) -> str:
        """Render an EXPLAIN plan node as an indented text tree. One line per
        node with the bits a human reader cares about: node type, scan
        strategy, target relation, picked index. Children indented two
        spaces."""
        indent = "  " * depth
        node_type = plan_node.get("Node Type", "?")
        parts = [node_type]
        strategy = plan_node.get("Strategy")
        # "Plain" is the default for Aggregate; only show the strategy when
        # it's something interesting (Hashed, Sorted, Mixed).
        if strategy and strategy != "Plain":
            parts.append(f"({strategy})")
        relation = plan_node.get("Relation Name")
        index = plan_node.get("Index Name")
        if index:
            parts.append(f"using {index}")
            if relation:
                parts.append(f"on {relation}")
        elif relation:
            parts.append(f"on {relation}")
        line = f"{indent}{' '.join(parts)}"
        child_lines = [
            self.format_plan_tree(child, depth + 1)
            for child in plan_node.get("Plans", [])
        ]
        return "\n".join([line] + child_lines)

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
        
        # Add subsections for each ingest type
        ingest_types = sorted(set(r.bench_type for r in self.ingest_results))
        for it in ingest_types:
            title = it.replace('_', ' ').title()
            anchor = it.replace('_', '-')
            f.write(f"   - [{title}](#{anchor})\n")
        
        f.write("2. [Query Performance](#query-performance)\n")
        
        # Add subsections for each query type
        query_types = set(r.query_type for r in self.query_results)
        for qt in sorted(query_types):
            f.write(f"   - [{qt} Queries](#{qt.lower()}-queries)\n")
        
        f.write("\n---\n\n")

    def _write_ingest_section(self, f):
        f.write("## Ingest Throughput\n\n")
        f.write("This section measures the throughput of inserting encrypted records into the database.\n\n")
        
        # Add comparison charts at the top if matplotlib is available
        if HAS_MATPLOTLIB:
            self._write_comparison_charts(f)
        
        # Group by bench_type - use all types found in results
        ingest_types = sorted(set(r.bench_type for r in self.ingest_results))
        
        for bench_type in ingest_types:
            results = [r for r in self.ingest_results if r.bench_type == bench_type]
            if not results:
                continue
            
            # Sort by num_records
            results.sort(key=lambda x: x.num_records)
            
            f.write(f"### {bench_type.replace('_', ' ').title()}\n\n")
            
            # Add descriptions for each type
            descriptions = {
                "int": "Tests insertion of encrypted integer values.",
                "json_small": "Tests insertion of small encrypted JSON objects (first_name, last_name, age, email).",
                "json_large": "Tests insertion of large encrypted JSON objects with complex nested structures (user info, company, addresses, orders).",
                "string": "Tests insertion of encrypted string values.",
                "ste_vec_small": "Tests insertion of small JSON objects with SteVec (searchable encrypted vector) indexing.",
                "ste_vec_large": "Tests insertion of large JSON objects with SteVec (searchable encrypted vector) indexing.",
            }
            
            if bench_type in descriptions:
                f.write(f"{descriptions[bench_type]}\n\n")
            
            # Table
            f.write("| Records | Throughput (records/sec) | Total Time | Avg Memory |\n")
            f.write("|---------|--------------------------|------------|------------|\n")
            
            for r in results:
                f.write(f"| {r.num_records:,} | {self.format_throughput(r.throughput)} | "
                       f"{r.total_time:.2f}s | {r.avg_memory_mb:.2f} MB |\n")
            
            f.write("\n")
            
            # Generate charts if matplotlib is available
            if HAS_MATPLOTLIB:
                # Throughput chart
                throughput_chart_path = self.output_file.parent / f"ingest_{bench_type}_throughput_chart.png"
                self._create_ingest_throughput_chart(results, bench_type, throughput_chart_path)
                f.write(f"![Ingest Throughput - {bench_type}]({throughput_chart_path.name})\n\n")
                
                # Total time chart
                time_chart_path = self.output_file.parent / f"ingest_{bench_type}_time_chart.png"
                self._create_ingest_time_chart(results, bench_type, time_chart_path)
                f.write(f"![Ingest Total Time - {bench_type}]({time_chart_path.name})\n\n")

    def _create_ingest_throughput_chart(self, results: List[IngestResult], bench_type: str, output_path: Path):
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
        
        # Add "larger is better" annotation
        ax.text(0.98, 0.98, 'larger is better ↑', transform=ax.transAxes,
                fontsize=11, verticalalignment='top', horizontalalignment='right',
                bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))
        
        plt.tight_layout()
        plt.savefig(output_path, dpi=100, bbox_inches='tight')
        plt.close()
    
    def _create_ingest_time_chart(self, results: List[IngestResult], bench_type: str, output_path: Path):
        """Create a bar chart for total ingest time"""
        fig, ax = plt.subplots(figsize=(10, 6))
        
        records = [r.num_records for r in results]
        times = [r.total_time for r in results]
        
        ax.bar(range(len(records)), times, color='coral')
        ax.set_xlabel('Number of Records')
        ax.set_ylabel('Total Time (seconds)')
        ax.set_title(f'Ingest Total Time - {bench_type.replace("_", " ").title()}')
        ax.set_xticks(range(len(records)))
        ax.set_xticklabels([f"{r:,}" for r in records])
        ax.grid(axis='y', alpha=0.3)
        
        # Add "smaller is better" annotation
        ax.text(0.98, 0.98, 'smaller is better ↓', transform=ax.transAxes,
                fontsize=11, verticalalignment='top', horizontalalignment='right',
                bbox=dict(boxstyle='round', facecolor='lightblue', alpha=0.5))
        
        plt.tight_layout()
        plt.savefig(output_path, dpi=100, bbox_inches='tight')
        plt.close()
    
    def _write_comparison_charts(self, f):
        """Write comparison charts for all benchmark types at 10000 rows"""
        target_row_count = 10000
        
        # Get results for each bench type at target row count
        comparison_data = {}
        for result in self.ingest_results:
            if result.num_records == target_row_count:
                comparison_data[result.bench_type] = result
        
        if len(comparison_data) < 2:
            # Not enough data for comparison
            return
        
        f.write(f"### Comparison at {target_row_count:,} Records\n\n")
        f.write(f"Comparing all benchmark types at {target_row_count:,} records.\n\n")
        
        # Create throughput comparison chart
        throughput_chart_path = self.output_file.parent / f"ingest_comparison_throughput_{target_row_count}.png"
        self._create_comparison_throughput_chart(comparison_data, target_row_count, throughput_chart_path)
        f.write(f"![Throughput Comparison at {target_row_count:,} records]({throughput_chart_path.name})\n\n")
        
        # Create time comparison chart
        time_chart_path = self.output_file.parent / f"ingest_comparison_time_{target_row_count}.png"
        self._create_comparison_time_chart(comparison_data, target_row_count, time_chart_path)
        f.write(f"![Total Time Comparison at {target_row_count:,} records]({time_chart_path.name})\n\n")
        
        # Create time comparison chart without ste_vec_large
        filtered_data = {k: v for k, v in comparison_data.items() if k != 'ste_vec_large'}
        if len(filtered_data) >= 2:
            time_chart_path_filtered = self.output_file.parent / f"ingest_comparison_time_{target_row_count}_filtered.png"
            self._create_comparison_time_chart(filtered_data, target_row_count, time_chart_path_filtered, exclude_label='ste_vec_large')
            f.write(f"![Total Time Comparison at {target_row_count:,} records (excluding ste_vec_large)]({time_chart_path_filtered.name})\n\n")
    
    def _create_comparison_throughput_chart(self, comparison_data: Dict[str, IngestResult], 
                                           row_count: int, output_path: Path):
        """Create a bar chart comparing throughput across all benchmark types"""
        fig, ax = plt.subplots(figsize=(12, 6))
        
        # Sort by throughput descending (highest on left)
        sorted_items = sorted(comparison_data.items(), key=lambda x: x[1].throughput, reverse=True)
        bench_types = [item[0] for item in sorted_items]
        labels = [bt.replace('_', ' ').title() for bt in bench_types]
        throughputs = [comparison_data[bt].throughput for bt in bench_types]
        
        # Use different colors for each bar
        colors = plt.cm.Set3(range(len(bench_types)))
        
        ax.bar(range(len(bench_types)), throughputs, color=colors)
        ax.set_xlabel('Benchmark Type', fontsize=12)
        ax.set_ylabel('Throughput (records/sec)', fontsize=12)
        ax.set_title(f'Ingest Throughput Comparison at {row_count:,} Records', fontsize=14, fontweight='bold')
        ax.set_xticks(range(len(bench_types)))
        ax.set_xticklabels(labels, rotation=45, ha='right')
        ax.grid(axis='y', alpha=0.3)
        
        # Add "larger is better" annotation
        ax.text(0.98, 0.98, 'larger is better ↑', transform=ax.transAxes,
                fontsize=11, verticalalignment='top', horizontalalignment='right',
                bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))
        
        plt.tight_layout()
        plt.savefig(output_path, dpi=100, bbox_inches='tight')
        plt.close()
    
    def _create_comparison_time_chart(self, comparison_data: Dict[str, IngestResult], 
                                     row_count: int, output_path: Path, exclude_label: str = None):
        """Create a bar chart comparing total time across all benchmark types"""
        fig, ax = plt.subplots(figsize=(12, 6))
        
        # Sort by throughput descending (highest on left) to match throughput chart order
        sorted_items = sorted(comparison_data.items(), key=lambda x: x[1].throughput, reverse=True)
        bench_types = [item[0] for item in sorted_items]
        labels = [bt.replace('_', ' ').title() for bt in bench_types]
        times = [comparison_data[bt].total_time for bt in bench_types]
        
        # Use different colors for each bar
        colors = plt.cm.Set2(range(len(bench_types)))
        
        ax.bar(range(len(bench_types)), times, color=colors)
        ax.set_xlabel('Benchmark Type', fontsize=12)
        ax.set_ylabel('Total Time (seconds)', fontsize=12)
        title = f'Total Ingest Time Comparison at {row_count:,} Records'
        if exclude_label:
            title += f' (excluding {exclude_label.replace("_", " ").title()})'
        ax.set_title(title, fontsize=14, fontweight='bold')
        ax.set_xticks(range(len(bench_types)))
        ax.set_xticklabels(labels, rotation=45, ha='right')
        ax.grid(axis='y', alpha=0.3)
        
        # Add "smaller is better" annotation
        ax.text(0.98, 0.98, 'smaller is better ↓', transform=ax.transAxes,
                fontsize=11, verticalalignment='top', horizontalalignment='right',
                bbox=dict(boxstyle='round', facecolor='lightblue', alpha=0.5))
        
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
            if param:
                f.write(f"**Parameter:** `{param}`\n\n")

        f.write(f"**{table_info}**\n\n")

        # Add index information for one of the row counts (they all use same indexes)
        if results:
            # Determine table name based on query type / scenario.
            sample_row_count = results[0].row_count
            if query_type == "GROUP_BY" and query_name == "count_groups_plaintext":
                # Plaintext baseline runs against a plain TEXT column — no
                # functional EQL indexes; lookup will return None and the
                # Indexes block will be skipped.
                table_name = f"string_plaintext_{sample_row_count}"
            elif query_type in ["EXACT", "MATCH", "GROUP_BY"]:
                # String-encrypted scenarios all run against the same table family.
                table_name = f"string_encrypted_{sample_row_count}"
            elif query_type == "ORE":
                table_name = f"integer_encrypted_{sample_row_count}"
            elif query_type == "JSON":
                table_name = f"json_ste_vec_small_encrypted_{sample_row_count}"
            else:
                table_name = ""

            if table_name:
                indexes_sql = self.get_table_indexes(table_name)
                if indexes_sql:
                    f.write(f"**Indexes available on the table:**\n```sql\n{indexes_sql}\n```\n\n")

        # Group by row_count for the various per-tier renderings below.
        row_counts = sorted(set(r.row_count for r in results))

        # Indexes used per data set size, sourced from the EXPLAIN metadata
        # sidecars. The list can vary between tiers — small tables often
        # take a Seq Scan even when a functional index exists.
        used_by_size = [
            (rc, self.metadata.get((query_type, query_name, rc)))
            for rc in row_counts
        ]
        if any(m is not None for _, m in used_by_size):
            f.write("**Indexes used by the planner (per data set size):**\n\n")
            for rc, meta in used_by_size:
                if meta is None:
                    f.write(f"- {rc:,}: _(no metadata)_\n")
                elif meta.indexes_used:
                    idx_list = ", ".join(f"`{i}`" for i in meta.indexes_used)
                    f.write(f"- {rc:,}: {idx_list}\n")
                else:
                    f.write(f"- {rc:,}: _none — planner picked a sequential / hash-aggregate / sort plan_\n")
            f.write("\n")

        # Create table with legend if any results exceed 100ms
        has_slow_queries = any((r.mean_ns / 1_000_000) > 100 for r in results)

        if has_slow_queries:
            f.write("*⚠️ = Query time exceeds 100ms*\n\n")

        # Decide column header based on whether any tier has actual rows
        # data (preferred) or only planner estimates from EXPLAIN.
        any_actual = any(
            (m := self.metadata.get((query_type, query_name, rc))) is not None
            and m.rows_returned is not None
            for rc in row_counts
        )
        rows_header = "Rows Returned" if any_actual else "Rows (est.)"
        f.write(f"| Data Set Size | {rows_header} | Query Time (no decrypt) | Query Time (with decrypt) |\n")
        f.write("|---------------|-")
        f.write("-" * len(rows_header))
        f.write("-|-------------------------|---------------------------|\n")

        for row_count in row_counts:
            no_decrypt = next((r for r in results if r.row_count == row_count and not r.decrypt), None)
            with_decrypt = next((r for r in results if r.row_count == row_count and r.decrypt), None)

            no_decrypt_str = self.format_time(no_decrypt.mean_ns) if no_decrypt else "N/A"
            with_decrypt_str = self.format_time(with_decrypt.mean_ns) if with_decrypt else "N/A"

            meta = self.metadata.get((query_type, query_name, row_count))
            # Prefer the actual row count from a pre-bench execute; fall
            # back to the planner's estimate when the sidecar predates
            # actual-rows capture, with an explicit (est.) suffix so the
            # number isn't misread as authoritative.
            if meta is not None and meta.rows_returned is not None:
                rows_str = f"{meta.rows_returned:,}"
            elif meta is not None:
                planner = self.planner_estimated_rows(meta.explain)
                rows_str = f"{planner:,} (est.)" if planner is not None else "—"
            else:
                rows_str = "—"

            f.write(f"| {row_count:,} | {rows_str} | {no_decrypt_str} | {with_decrypt_str} |\n")

        f.write("\n")
        if not any_actual:
            f.write("_Rows are the planner's estimate from `EXPLAIN` "
                    "captured before the bench loop; re-run the bench with the "
                    "current source to capture actual row counts._\n\n")
        else:
            f.write("_Rows Returned is the actual count from a one-shot pre-bench "
                    "execution. For LIMIT-bounded queries it matches the LIMIT (or "
                    "is lower when the table doesn't have enough matching rows); "
                    "for aggregates wrapped in `count(*)` it's 1._\n\n")

        # Per-tier EXPLAIN plans (collapsed). Useful when the plan shape
        # changes across data sizes — e.g. the ORE bench, where the planner
        # picks Seq Scan at every tier for bare-range queries but switches to
        # Index Scan for the hybrid ordered scenario.
        explain_blocks = [
            (rc, meta)
            for rc, meta in used_by_size
            if meta is not None and meta.explain
        ]
        if explain_blocks:
            f.write("<details>\n<summary>EXPLAIN plans (per data set size)</summary>\n\n")
            for rc, meta in explain_blocks:
                f.write(f"**{rc:,} rows**\n\n")
                plan = meta.explain[0].get("Plan", {})
                tree = self.format_plan_tree(plan)
                f.write(f"```\n{tree}\n```\n\n")
                f.write("Full `EXPLAIN (FORMAT JSON)`:\n\n")
                pretty = json.dumps(meta.explain, indent=2)
                f.write(f"```json\n{pretty}\n```\n\n")
            f.write("</details>\n\n")

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
        
        # Add "smaller is better" annotation
        ax.text(0.98, 0.98, 'smaller is better ↓', transform=ax.transAxes,
                fontsize=11, verticalalignment='top', horizontalalignment='right',
                bbox=dict(boxstyle='round', facecolor='lightblue', alpha=0.5))
        
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

    print("Loading query metadata sidecars...")
    reporter.load_query_metadata()
    print(f"  Found {len(reporter.metadata)} per-scenario metadata records")
    
    print(f"Generating report: {args.output}")
    reporter.generate_report()
    
    print(f"\n✓ Report generated successfully: {args.output}")
    
    if not HAS_MATPLOTLIB:
        print("\nNote: Charts were not generated. Install matplotlib to enable charts:")
        print("  pip3 install matplotlib")


if __name__ == "__main__":
    main()
