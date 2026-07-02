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


def eql_version_for(query_type: str) -> int:
    """EQL version axis for a query-type family: the `_V3` families run
    against the eql_v3 schema; everything else (including pre-version
    result files) is v2."""
    return 3 if query_type.endswith("_V3") else 2


@dataclass
class QueryResult:
    """Results from a query benchmark"""
    query_type: str  # e.g., "EXACT", "MATCH", "ORE", "EXACT_V3"
    query_name: str  # e.g., "eql_cast", "range_gt_10"
    row_count: int
    decrypt: bool
    mean_ns: float
    median_ns: float
    version: int = 2  # EQL version axis (2 | 3)


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
    # EQL version axis (2 | 3). Sidecars written before the axis existed
    # carry no `version` field and are v2.
    version: int = 2


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
        """Load ingest benchmark results.

        Discovers every `*_combined.json` under results/ingest (rather than
        a hardcoded family list) so the `encrypt_*_v3` and
        `convert_overhead_*` families are picked up alongside the v2 ones.
        The bench type is the filename minus a leading `encrypt_` and the
        trailing `_combined` — v2 names are unchanged (`int`, `string`, …)
        and the v3 twins sort adjacent to them (`int_v3`, `string_v3`, …)
        for side-by-side reading.
        """
        ingest_dir = self.results_dir / "ingest"
        if not ingest_dir.is_dir():
            print(f"Warning: {ingest_dir} not found, skipping ingest results", file=sys.stderr)
            return

        for file_path in sorted(ingest_dir.glob("*_combined.json")):
            bench_type = file_path.stem
            if bench_type.endswith("_combined"):
                bench_type = bench_type[:-len("_combined")]
            if bench_type.startswith("encrypt_"):
                bench_type = bench_type[len("encrypt_"):]

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
                    
                    # Parse benchmark ID: "QUERY_TYPE/query_variant/scenario.../rows"
                    # `scenario...` may be one component (most benches) or several
                    # (e.g. the JSON bench uses `contains/functional`,
                    # `field_eq/bare`, etc. — variant in parts[-2], scenario
                    # head in parts[2]). Join everything between parts[2] and
                    # the trailing row count so multi-part scenario IDs stay
                    # distinct in the report.
                    bench_id = data.get("id", "")
                    parts = bench_id.split("/")

                    if len(parts) < 4:
                        continue

                    # Determine if this is a decrypt variant
                    decrypt = "decrypt" in parts[1]

                    # Get scenario name (e.g., "eql_cast", "range_gt_10",
                    # "contains/functional", "field_eq/bare")
                    scenario = "/".join(parts[2:-1])
                    
                    # Extract mean timing
                    mean_ns = data.get("mean", {}).get("estimate", 0)
                    median_ns = data.get("median", {}).get("estimate", 0)
                    
                    self.query_results.append(QueryResult(
                        query_type=query_type,
                        query_name=scenario,
                        row_count=row_count,
                        decrypt=decrypt,
                        mean_ns=mean_ns,
                        median_ns=median_ns,
                        version=eql_version_for(query_type)
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
                if len(parts) < 4:
                    continue
                query_type = parts[0]                # "EXACT", "ORE", "JSON", ...
                # Join `parts[2:-1]` so multi-part scenario IDs (e.g.
                # `contains/functional`, `field_eq/bare`) stay distinct.
                scenario_name = "/".join(parts[2:-1])
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
                    # Absent field = pre-version sidecar = v2.
                    version=s.get("version", 2),
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
                "range_selective_gt_100": (
                    "SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 100",
                    "2140000000"
                ),
                "range_highly_selective_gt_10": (
                    "SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 10",
                    "2147000000"
                ),
                "range_selective_gt_count": (
                    "SELECT count(*) FROM {TABLE} WHERE value > $1",
                    "2140000000"
                ),
                "range_highly_selective_gt_count": (
                    "SELECT count(*) FROM {TABLE} WHERE value > $1",
                    "2147000000"
                ),
                "range_lt_hybrid_ordered_10": (
                    "SELECT id,value::jsonb FROM {TABLE} "
                    "WHERE value < $1 "
                    "ORDER BY eql_v2.ore_block_u64_8_256(value) LIMIT 10",
                    "5000"
                ),
                "range_lt_natural_ordered_10": (
                    "SELECT id,value::jsonb FROM {TABLE} "
                    "WHERE value < $1 "
                    "ORDER BY value LIMIT 10",
                    "5000"
                )
            },
            "GROUP_BY": {
                "low_cardinality_groups_encrypted": (
                    "SELECT count(*) FROM "
                    "(SELECT 1 FROM {TABLE} GROUP BY eql_v2.hmac_256(value)) g",
                    ""
                ),
                "low_cardinality_groups_plaintext": (
                    "SELECT count(*) FROM "
                    "(SELECT 1 FROM {TABLE} GROUP BY value) g",
                    ""
                ),
                "top_n_groups_encrypted": (
                    "SELECT eql_v2.hmac_256(value), count(*) FROM {TABLE} "
                    "GROUP BY 1 ORDER BY count(*) DESC LIMIT 10",
                    ""
                ),
                "top_n_groups_plaintext": (
                    "SELECT value, count(*) FROM {TABLE} "
                    "GROUP BY 1 ORDER BY count(*) DESC LIMIT 10",
                    ""
                )
            },
            "COMBO": {
                "bloom_ore_order_limit": (
                    "SELECT id FROM {TABLE} "
                    "WHERE name LIKE $1 "
                    "ORDER BY eql_v2.ore_block_u64_8_256(age) LIMIT 10",
                    "Bob"
                ),
                "filtered_group_by": (
                    "SELECT eql_v2.hmac_256(category), count(*) FROM {TABLE} "
                    "WHERE name LIKE $1 "
                    "GROUP BY 1",
                    "Bob"
                ),
                "top_n_filtered_group_by": (
                    "SELECT eql_v2.hmac_256(category), count(*) FROM {TABLE} "
                    "WHERE name LIKE $1 "
                    "GROUP BY 1 ORDER BY count(*) DESC LIMIT 10",
                    "Bob"
                )
            },
            "JSON": {
                "contains/functional": (
                    "SELECT id FROM {TABLE} "
                    "WHERE eql_v2.ste_vec(value) @> eql_v2.ste_vec($1::jsonb::eql_v2_encrypted) "
                    "LIMIT 10",
                    "<sampled-row-value-as-jsonb>"
                ),
                "field_eq/bare": (
                    "SELECT id FROM {TABLE} "
                    "WHERE (value -> '<selector-hash>'::text) = $1::jsonb::eql_v2_encrypted "
                    "LIMIT 10",
                    "<sampled-sv-element-as-jsonb>"
                ),
                "field_eq/extractor": (
                    "SELECT id FROM {TABLE} "
                    "WHERE eql_v2.hmac_256_terms(value) @> $1::jsonb "
                    "LIMIT 10",
                    "[{\"s\":\"<selector-hash>\",\"hm\":\"<hmac>\"}]"
                ),
                "field_eq/functional": (
                    "SELECT id FROM {TABLE} "
                    "WHERE eql_v2.hmac_256(value, '<selector-hash>') "
                    "= eql_v2.hmac_256($1::eql_v2_encrypted) "
                    "LIMIT 10",
                    "<sampled-sv-element-as-eql_v2_encrypted>"
                ),
                "field_order/bare": (
                    "SELECT id FROM {TABLE} "
                    "ORDER BY (value -> '<selector-hash>'::text) LIMIT 10",
                    ""
                ),
                "field_order/functional": (
                    "SELECT id FROM {TABLE} "
                    "ORDER BY <ore_extractor>(value -> '<selector-hash>'::text) LIMIT 10",
                    ""
                )
            },
            # --- EQL v3 twins. Probe parameters are STORED-shape payloads
            # converted with from_v2 (no v3 scalar query wire shape exists);
            # the SQL compares via the eql_v3.*_term extractors or the
            # inlinable typed operators.
            "EXACT_V3": {
                "eql_cast": (
                    "SELECT id, value::jsonb FROM {TABLE} "
                    "WHERE value = $1::eql_v3.text_search LIMIT 1",
                    "<sampled row plaintext>"
                ),
                "eql_hash": (
                    "SELECT id, value::jsonb FROM {TABLE} "
                    "WHERE eql_v3.eq_term(value) = eql_v3.eq_term($1::eql_v3.text_search) LIMIT 1",
                    "<sampled row plaintext>"
                )
            },
            "MATCH_V3": {
                "eql_bloom": (
                    "SELECT id, value::jsonb FROM {TABLE} "
                    "WHERE eql_v3.match_term(value) @> eql_v3.match_term($1::eql_v3.text_search) LIMIT 10",
                    "Johnson"
                ),
                "eql_bloom_bare": (
                    "SELECT id, value::jsonb FROM {TABLE} "
                    "WHERE value @> $1::eql_v3.text_search LIMIT 10",
                    "Johnson"
                )
            },
            "ORE_V3": {
                "range_gt_10": (
                    "SELECT id, value::jsonb FROM {TABLE} "
                    "WHERE value > $1::eql_v3.int4_ord_ore LIMIT 10",
                    "5000"
                ),
                "range_gt_100": (
                    "SELECT id, value::jsonb FROM {TABLE} "
                    "WHERE value > $1::eql_v3.int4_ord_ore LIMIT 100",
                    "5000"
                ),
                "range_lt_10": (
                    "SELECT id, value::jsonb FROM {TABLE} "
                    "WHERE value < $1::eql_v3.int4_ord_ore LIMIT 10",
                    "5000"
                ),
                "range_lt_100": (
                    "SELECT id, value::jsonb FROM {TABLE} "
                    "WHERE value < $1::eql_v3.int4_ord_ore LIMIT 100",
                    "5000"
                ),
                "range_lt_ordered_10": (
                    "SELECT id, value::jsonb FROM {TABLE} "
                    "WHERE value < $1::eql_v3.int4_ord_ore "
                    "ORDER BY eql_v3.ord_term(value) LIMIT 10",
                    "5000"
                )
            },
            "GROUP_BY_V3": {
                "low_cardinality_groups_encrypted": (
                    "SELECT count(*) FROM "
                    "(SELECT 1 FROM {TABLE} GROUP BY eql_v3.eq_term(value)) g",
                    ""
                ),
                "top_n_groups_encrypted": (
                    "SELECT eql_v3.eq_term(value), count(*) FROM {TABLE} "
                    "GROUP BY 1 ORDER BY count(*) DESC LIMIT 10",
                    ""
                )
            },
            "COMBO_V3": {
                "bloom_ore_order_limit": (
                    "SELECT id FROM {TABLE} "
                    "WHERE eql_v3.match_term(name) @> eql_v3.match_term($1::eql_v3.text_match) "
                    "ORDER BY eql_v3.ord_term(age) LIMIT 10",
                    "Bob"
                ),
                "filtered_group_by": (
                    "SELECT eql_v3.eq_term(category), count(*) FROM {TABLE} "
                    "WHERE eql_v3.match_term(name) @> eql_v3.match_term($1::eql_v3.text_match) "
                    "GROUP BY 1",
                    "Bob"
                ),
                "top_n_filtered_group_by": (
                    "SELECT eql_v3.eq_term(category), count(*) FROM {TABLE} "
                    "WHERE eql_v3.match_term(name) @> eql_v3.match_term($1::eql_v3.text_match) "
                    "GROUP BY 1 ORDER BY count(*) DESC LIMIT 10",
                    "Bob"
                )
            },
            "JSON_V3": {
                "contains/functional": (
                    "SELECT id FROM {TABLE} "
                    "WHERE value @> $1::jsonb::eql_v3.jsonb_query LIMIT 10",
                    "<sampled row via eql_v3.to_ste_vec_query>"
                ),
                "field_eq/bare": (
                    "SELECT id FROM {TABLE} "
                    "WHERE (value -> '<selector-hash>'::text) = $1::jsonb::eql_v3.jsonb_entry "
                    "LIMIT 10",
                    "<sampled sv entry as jsonb>"
                ),
                "field_eq/extractor": (
                    "SELECT id FROM {TABLE} "
                    "WHERE value @> $1::jsonb::eql_v3.jsonb_query LIMIT 10",
                    "{\"sv\":[{\"s\":\"<selector-hash>\",\"hm\":\"<hmac>\"}]}"
                ),
                "field_eq/functional": (
                    "SELECT id FROM {TABLE} "
                    "WHERE eql_v3.eq_term(value -> '<selector-hash>'::text) "
                    "= eql_v3.eq_term($1::jsonb::eql_v3.jsonb_entry) LIMIT 10",
                    "<sampled sv entry as jsonb>"
                ),
                "field_order/functional": (
                    "SELECT id FROM {TABLE} "
                    "ORDER BY eql_v3.ore_cllw(value -> '<selector-hash>'::text) LIMIT 10",
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
                "range_selective_gt_100": (
                    "Selective range query (~0.17% selectivity) with LIMIT 100",
                    "Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. "
                    "Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. "
                    "Query: WHERE value > 2_140_000_000 LIMIT 100. The threshold sits 7.5M "
                    "values below `i32::MAX`, so ~0.17% of rows match on `Faker.fake::<i32>()` "
                    "uniform random data. Engages the ORE btree at every tier (10k → 10M) — "
                    "walking the b-tree from the top and returning the first 100 matches is "
                    "cheaper than scanning the table once the planner knows the predicate is "
                    "selective. **Note on stats**: this requires up-to-date planner stats on "
                    "the functional index expression (`ANALYZE <table>` after re-ingest). "
                    "Without current stats the planner falls back to default `>` selectivity "
                    "(~14%) and picks Seq Scan, which is silent but produces misleading "
                    "timing. The bench's `prepare:_table` now ANALYZE's automatically."
                ),
                "range_highly_selective_gt_10": (
                    "Highly selective range query (~0.011% selectivity) with LIMIT 10",
                    "Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. "
                    "Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. "
                    "Query: WHERE value > 2_147_000_000 LIMIT 10. Threshold sits 483k values "
                    "below `i32::MAX` (~0.011% selectivity). Engages the ORE btree at every "
                    "tier (with current stats — see the note on `range_selective_gt_100`). "
                    "Useful as the upper-bound demonstration of how cheap a selective range "
                    "lookup becomes when the functional index engages."
                ),
                "range_selective_gt_count": (
                    "Selective range count (~0.17% selectivity), no LIMIT",
                    "Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. "
                    "Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. "
                    "Query: `SELECT count(*) FROM tbl WHERE value > 2_140_000_000`. With no "
                    "LIMIT the planner must process every matching row, which at low "
                    "selectivity strongly favours Index Scan over Seq Scan. The companion "
                    "to `range_selective_gt_100` — removes any LIMIT-related cost-model "
                    "edge cases and demonstrates the index path in pure form."
                ),
                "range_highly_selective_gt_count": (
                    "Highly selective range count (~0.011% selectivity), no LIMIT",
                    "Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. "
                    "Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. "
                    "Query: `SELECT count(*) FROM tbl WHERE value > 2_147_000_000`. Tighter "
                    "selectivity than `range_selective_gt_count`; near-floor cost for an "
                    "indexed lookup."
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
                ),
                "range_lt_natural_ordered_10": (
                    "Ordered range query (natural form: column in ORDER BY)",
                    "Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. "
                    "Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. "
                    "Query: WHERE value < 5000 ORDER BY value LIMIT 10. The sort key doesn't "
                    "match the index expression, so the plan keeps a residual Top-N Sort over "
                    "the bitmap-scan output. Post-EQL #218 each comparison in the sort is the "
                    "inlined ORE-term path (no plpgsql dispatch per row), but the Sort cost "
                    "still scales with the size of the post-WHERE set. Companion to "
                    "`range_lt_hybrid_ordered_10`; the cost delta is the price of the §4 "
                    "sort-key shortcut."
                )
            },
            "GROUP_BY": {
                "low_cardinality_groups_encrypted": (
                    "Low-cardinality GROUP BY (~250 buckets) on `eql_v2.hmac_256(value)`, "
                    "wrapped in `count(*)` to isolate aggregation cost from emit cost",
                    "Table: `category_encrypted_{rows}` with encrypted categorical values "
                    "(`CAT_001`..`CAT_250`, uniform random — ~250 distinct buckets). The "
                    "encrypted value carries an `hm` HMAC term via the `unique` search "
                    "index. **Index: hash index on `eql_v2.hmac_256(value)`, but `GROUP BY` "
                    "doesn't engage it directly** — the planner picks `HashAggregate`, "
                    "building an in-memory hash table keyed on the 32-byte HMAC. With "
                    "only 250 distinct keys the hash table fits comfortably in default "
                    "`work_mem`. The outer `count(*)` keeps the result-set emission at "
                    "exactly one row, so wall-clock time tracks aggregation cost. The "
                    "companion `low_cardinality_groups_plaintext` scenario runs the same "
                    "query shape against an unindexed TEXT column for a baseline."
                ),
                "low_cardinality_groups_plaintext": (
                    "Plaintext baseline: low-cardinality GROUP BY on a plain TEXT column, "
                    "same query shape as the encrypted scenario",
                    "Table: `category_plaintext_{rows}` with the same `CAT_001`..`CAT_250` "
                    "distribution (uniform random, populated by SQL via "
                    "`mise run prepare:category_plaintext` — no encryption-client "
                    "dependency). Index: none. The wall-clock delta between this and "
                    "`low_cardinality_groups_encrypted` is the EQL recipe's overhead "
                    "relative to a bare-PG aggregate at the same row count and cardinality."
                ),
                "top_n_groups_encrypted": (
                    "Dashboard analytic: top 10 categories by frequency, EQL recipe form",
                    "Table: `category_encrypted_{rows}` (same data as the "
                    "`low_cardinality_*` scenarios above). Query: "
                    "`SELECT eql_v2.hmac_256(value), count(*) FROM tbl GROUP BY 1 "
                    "ORDER BY count(*) DESC LIMIT 10`. The bench always emits 10 rows "
                    "regardless of input size, so the cost is dominated by the inner "
                    "HashAggregate (per-row HMAC + hash-table insert) plus a tiny "
                    "sort over the 250 group entries. Realistic shape for analytics "
                    "queries that surface the most common categories in an encrypted "
                    "dataset."
                ),
                "top_n_groups_plaintext": (
                    "Plaintext baseline: top 10 categories by frequency on a plain TEXT "
                    "column",
                    "Table: `category_plaintext_{rows}`. Same query shape as the "
                    "encrypted top-N scenario; the delta is the EQL recipe's overhead "
                    "for the same shape on the same cardinality data."
                )
            },
            "COMBO": {
                "bloom_ore_order_limit": (
                    "Composite predicate: filter by name pattern (bloom), order by age "
                    "(ORE), limit 10",
                    "Table: `combo_encrypted_{rows}` with three encrypted columns — "
                    "`name` (match + hmac), `age` (ORE), `category` (hmac). Indexes: "
                    "functional GIN on `eql_v2.bloom_filter(name)`, functional btree on "
                    "`eql_v2.ore_block_u64_8_256(age)`, functional hash on "
                    "`eql_v2.hmac_256(category)`. **The bloom GIN index engages for the "
                    "LIKE predicate**, narrowing the input to ~0.01–0.1% of rows; the "
                    "planner then sorts the small filtered set by `eql_v2.ore_block_u64_8_256(age)` "
                    "and returns the top 10. The ORE btree doesn't engage here — PostgreSQL "
                    "can't merge two unrelated indexes on different columns (bloom on `name`, "
                    "btree on `age`), so the ORDER BY is satisfied by a Sort node above the "
                    "Bitmap Heap Scan. With the bloom narrowing so aggressively, that Sort "
                    "is cheap; the cost is dominated by the bloom + heap fetch."
                ),
                "filtered_group_by": (
                    "Composite predicate: filter by name pattern, GROUP BY category",
                    "Table: `combo_encrypted_{rows}`. Query: `SELECT eql_v2.hmac_256(category), "
                    "count(*) FROM tbl WHERE name LIKE $1 GROUP BY 1`. Bloom filter on "
                    "`name` filters the input set; HashAggregate then groups the small "
                    "post-filter set by the 32-byte category HMAC. With ~0.01-0.1% of "
                    "names matching a typical bloom pattern and 250 category buckets, the "
                    "aggregate stage is essentially free — the cost is bloom filter scan "
                    "plus per-matching-row HMAC."
                ),
                "top_n_filtered_group_by": (
                    "Dashboard analytic: top 10 categories for customers matching a name "
                    "pattern",
                    "Table: `combo_encrypted_{rows}`. Query: `SELECT eql_v2.hmac_256(category), "
                    "count(*) FROM tbl WHERE name LIKE $1 GROUP BY 1 ORDER BY count(*) "
                    "DESC LIMIT 10`. Same shape as `filtered_group_by` with an outer "
                    "Top-N sort + LIMIT 10. Realistic analytics shape for surfacing the "
                    "categories that contain the most customers matching a filter, "
                    "without revealing the underlying names or category labels."
                )
            },
            "JSON": {
                "contains/functional": (
                    "Whole-document JSON containment via `ste_vec(...) @> ste_vec(...)`",
                    "Table: `json_ste_vec_small_encrypted_{rows}` with encrypted JSON "
                    "documents (small four-field shape — first_name / last_name / age / email). "
                    "Index: functional GIN on `eql_v2.ste_vec(value)`. Both sides of `@>` "
                    "resolve to `eql_v2_encrypted[]`, which matches the GIN opclass directly. "
                    "The needle is a sampled row's value, so the query matches at least that "
                    "source row.\n\n"
                    "Note: the bare form `WHERE value @> $1::eql_v2_encrypted` does NOT engage "
                    "the GIN today. `eql_v2.\"@>\"` is marked inlinable SQL but wraps "
                    "`ste_vec_contains()` which is PL/pgSQL — inlining stops at the wrapper, "
                    "leaving the planner with a black-box function call and no path to the "
                    "indexed expression. The bench omits the bare form because it would not "
                    "complete at the 1M / 10M tiers."
                ),
                "field_eq/bare": (
                    "Field-level equality via `value -> 'sel' = $1::eql_v2_encrypted` (no index)",
                    "Table: `json_ste_vec_small_encrypted_{rows}`. `eql_v2.\"->\"` is plpgsql "
                    "(not inlinable), so the planner cannot match any functional index against "
                    "the LHS — forces Seq Scan + per-row sv walk. This is the natural form a "
                    "JS/ORM caller would write; the bench includes it to show the cost of "
                    "*not* having an inlinable extractor on `->`."
                ),
                "field_eq/extractor": (
                    "Field-level equality via `hmac_256_terms @> [{s,hm}]` (functional GIN)",
                    "Table: `json_ste_vec_small_encrypted_{rows}`. Index: functional GIN on "
                    "`eql_v2.hmac_256_terms(value)`. One index covers field-level equality "
                    "across every selector that carries `hm`, vs the per-selector recipe "
                    "below. The bench picks a (selector, hmac) pair from `sv[0]` of a sample "
                    "row at startup; needle is `[{\"s\":\"<sel>\",\"hm\":\"<hash>\"}]`."
                ),
                "field_eq/functional": (
                    "Field-level equality via per-selector `hmac_256(col, 'sel')`",
                    "Table: `json_ste_vec_small_encrypted_{rows}`. Would engage "
                    "`hash (eql_v2.hmac_256(col, '<sel>'))` if one existed; benches/main "
                    "only creates the `hmac_256_terms` GIN (one index for all selectors), "
                    "so this scenario serves as a baseline showing the cost of the "
                    "per-selector recipe without a matching index."
                ),
                "field_order/bare": (
                    "Field-level ORDER BY via `ORDER BY value -> 'sel'` (no index)",
                    "Table: `json_ste_vec_small_encrypted_{rows}`. Same `->` non-inlining "
                    "problem as `field_eq/bare`. ORDER BY on `eql_v2_encrypted` uses ORE "
                    "under the hood, but the planner can't see through `->` to engage any "
                    "functional ORE index. Forces Seq Scan + Top-N sort."
                ),
                "field_order/functional": (
                    "Field-level ORDER BY via ORE extractor on `value -> 'sel'`",
                    "Table: `json_ste_vec_small_encrypted_{rows}`. Index: functional btree on "
                    "`<ore_extractor>(value -> '<selector>'::text)` using the appropriate "
                    "opclass for the term type. `<ore_extractor>` is selected at bench startup "
                    "based on which orderable tag the sampled sv element carries:\n"
                    "  - `oc` → `eql_v2.ore_cllw` (Standard mode, ORE CLLW — requires the "
                    "`eql_v2.ore_cllw_ops` btree opclass from EQL #221)\n"
                    "  - `op` → `eql_v2.ope_cllw` (Compat mode, OPE CLLW)\n"
                    "  - `ob` → `eql_v2.ore_block_u64_8_256` (Block ORE — root scalars only)\n"
                    "When the table's `oc` index is present, the plan engages Index Scan + "
                    "LIMIT (no Sort node). When absent (older bench run / index not yet "
                    "rebuilt), falls back to Seq Scan + Top-N sort."
                )
            },
            # --- EQL v3 twins ---
            "EXACT_V3": {
                "eql_cast": (
                    "EQL v3 exact match using the inlinable `=` operator",
                    "Table: `string_encrypted_v3_{rows}` (column `eql_v3.text_search`). "
                    "Index: `hash (eql_v3.eq_term(value))`. The typed `=` inlines to "
                    "`eql_v3.eq_term(a) = eql_v3.eq_term(b)` and engages the index."
                ),
                "eql_hash": (
                    "EQL v3 exact match using the `eql_v3.eq_term` extractor",
                    "Table: `string_encrypted_v3_{rows}` (column `eql_v3.text_search`). "
                    "Index: `hash (eql_v3.eq_term(value))` — the explicit extractor form "
                    "of `eql_cast`, structurally identical after operator inlining."
                )
            },
            "MATCH_V3": {
                "eql_bloom": (
                    "EQL v3 bloom-filter token containment via the `eql_v3.match_term` extractor",
                    "Table: `string_encrypted_v3_{rows}` (column `eql_v3.text_search`). "
                    "Index: `GIN (eql_v3.match_term(value))`. v3 removes LIKE/ILIKE — the "
                    "two v2 `eql_cast_*` LIKE scenarios have no v3 twin; bloom containment "
                    "is the only encrypted text-matching surface."
                ),
                "eql_bloom_bare": (
                    "EQL v3 bloom containment via the bare typed `@>` operator",
                    "Table: `string_encrypted_v3_{rows}` (column `eql_v3.text_search`). "
                    "The ORM-shaped form: `value @> $1::eql_v3.text_search` inlines to the "
                    "same match_term expression as `eql_bloom` and should engage the same "
                    "GIN — the scenario prices exactly that inlining."
                )
            },
            "ORE_V3": {
                "range_gt_10": (
                    "EQL v3 range query (greater than) returning 10 results",
                    "Table: `integer_encrypted_v3_{rows}` (column `eql_v3.int4_ord_ore`). "
                    "Index: `btree (eql_v3.ord_term(value))`. Bare-form range operators "
                    "inline to ord_term comparisons and match the index; planner usage is "
                    "a selectivity question exactly as in the v2 family."
                ),
                "range_gt_100": (
                    "EQL v3 range query (greater than) returning 100 results",
                    "Table: `integer_encrypted_v3_{rows}` (column `eql_v3.int4_ord_ore`). "
                    "Index: `btree (eql_v3.ord_term(value))`."
                ),
                "range_lt_10": (
                    "EQL v3 range query (less than) returning 10 results",
                    "Table: `integer_encrypted_v3_{rows}` (column `eql_v3.int4_ord_ore`). "
                    "Index: `btree (eql_v3.ord_term(value))`."
                ),
                "range_lt_100": (
                    "EQL v3 range query (less than) returning 100 results",
                    "Table: `integer_encrypted_v3_{rows}` (column `eql_v3.int4_ord_ore`). "
                    "Index: `btree (eql_v3.ord_term(value))`."
                ),
                "range_lt_ordered_10": (
                    "EQL v3 ordered range query (extractor ORDER BY)",
                    "Table: `integer_encrypted_v3_{rows}` (column `eql_v3.int4_ord_ore`). "
                    "Index: `btree (eql_v3.ord_term(value))`. `ORDER BY eql_v3.ord_term(value)` "
                    "matches the index expression, so rows stream out already sorted — no "
                    "Sort node. Same sort-key rule as v2."
                )
            },
            "GROUP_BY_V3": {
                "low_cardinality_groups_encrypted": (
                    "EQL v3 low-cardinality GROUP BY (~250 buckets) on `eql_v3.eq_term(value)`",
                    "Table: `category_encrypted_v3_{rows}` (column `eql_v3.text_eq`, "
                    "same CAT_001..CAT_250 distribution as the v2 family). HashAggregate "
                    "keyed on the small deterministic eq_term. Compare against the shared "
                    "`low_cardinality_groups_plaintext` baseline in the v2 GROUP_BY family "
                    "— the plaintext tables are version-independent and not re-run for v3."
                ),
                "top_n_groups_encrypted": (
                    "EQL v3 dashboard analytic: top 10 categories by frequency",
                    "Table: `category_encrypted_v3_{rows}` (column `eql_v3.text_eq`). "
                    "Same shape as the v2 scenario with `eql_v3.eq_term` as the group key. "
                    "Plaintext baseline lives in the v2 GROUP_BY family."
                )
            },
            "COMBO_V3": {
                "bloom_ore_order_limit": (
                    "EQL v3 composite: bloom containment filter + ORE ORDER BY + LIMIT",
                    "Table: `combo_encrypted_v3_{rows}` (name `eql_v3.text_match`, age "
                    "`eql_v3.int4_ord_ore`, category `eql_v3.text_eq`). The v2 LIKE filter "
                    "becomes `eql_v3.match_term(name) @> eql_v3.match_term($1)` (v3 removes "
                    "LIKE); ORDER BY uses `eql_v3.ord_term(age)`."
                ),
                "filtered_group_by": (
                    "EQL v3 composite: bloom containment filter + GROUP BY category",
                    "Table: `combo_encrypted_v3_{rows}`. Bloom GIN narrows the input; "
                    "HashAggregate groups by `eql_v3.eq_term(category)`."
                ),
                "top_n_filtered_group_by": (
                    "EQL v3 dashboard analytic: top 10 categories for a bloom-filtered set",
                    "Table: `combo_encrypted_v3_{rows}`. Same as filtered_group_by plus "
                    "`ORDER BY count(*) DESC LIMIT 10`."
                )
            },
            "JSON_V3": {
                "contains/functional": (
                    "EQL v3 whole-document containment via the typed `@>` + jsonb_query needle",
                    "Table: `json_ste_vec_small_encrypted_v3_{rows}` (column `eql_v3.json`). "
                    "Index: `GIN ((eql_v3.to_ste_vec_query(value))::jsonb jsonb_path_ops)`. "
                    "The typed `@>` inlines to a native jsonb `@>` over the same expression. "
                    "Replaces the v2 `eql_v2.jsonb_array(...) @> ...` recipe; the needle is "
                    "the sampled row's own normalized query shape."
                ),
                "field_eq/bare": (
                    "EQL v3 field-level equality via `value -> 'sel' = $1` (inlinable)",
                    "Table: `json_ste_vec_small_encrypted_v3_{rows}`. Unlike v2 (plpgsql "
                    "`->`, unmatchable by the planner), the v3 `->` and `=` are inlinable — "
                    "the predicate reduces to eq_term comparisons and can engage the "
                    "per-selector `btree (eql_v3.eq_term(value -> '<sel>'::text))` index "
                    "the bench builds at startup."
                ),
                "field_eq/extractor": (
                    "EQL v3 field-level equality via the jsonb_query containment needle",
                    "Table: `json_ste_vec_small_encrypted_v3_{rows}`. Single-field needle "
                    "`{\"sv\":[{s,hm}]}` through the same to_ste_vec_query GIN as "
                    "contains/functional — one index covers every selector."
                ),
                "field_eq/functional": (
                    "EQL v3 field-level equality via the explicit `eql_v3.eq_term` form",
                    "Table: `json_ste_vec_small_encrypted_v3_{rows}`. The explicit extractor "
                    "spelling of field_eq/bare; engages the same per-selector btree."
                ),
                "field_order/functional": (
                    "EQL v3 field-level ORDER BY via `eql_v3.ore_cllw` on the extracted entry",
                    "Table: `json_ste_vec_small_encrypted_v3_{rows}`. Index: per-selector "
                    "`btree (eql_v3.ore_cllw(value -> '<sel>'::text))` built at bench "
                    "startup (the `eql_v3.ore_cllw_ops` opclass is DEFAULT for the type). "
                    "In v3 the only per-entry orderable tag is `oc` — no ob/op variants."
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
        
        # v3 tables keep their index scripts under sql/indexes/v3/.
        index_dir = self.sql_dir / "indexes"
        if "_v3" in table_name:
            index_dir = index_dir / "v3"

        # Try to find the index file
        index_file = index_dir / f"{table_name}_up.sql"

        if not index_file.exists():
            # Try without row count suffix (base table)
            # e.g., string_encrypted_10000 -> string_encrypted
            base_table = re.sub(r'_(\d+)$', '', table_name)
            index_file = index_dir / f"{base_table}_up.sql"
        
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
        """Generate the report as an index page plus one per-query-type page.

        Each per-type page lives at `report/<type>.md` (e.g. `exact.md`); the
        top-level `BENCHMARK_REPORT.md` is the index and links into them.
        """
        query_types = sorted(set(r.query_type for r in self.query_results))

        # Per-query-type pages first so the index can link to them.
        scenario_pages: Dict[str, str] = {}
        for query_type in query_types:
            page_name = f"{query_type.lower()}.md"
            page_path = self.output_file.parent / page_name
            with open(page_path, 'w') as pf:
                self._write_query_type_page_content(pf, query_type)
            scenario_pages[query_type] = page_name

        with open(self.output_file, 'w') as f:
            self._write_header(f, scenario_pages)
            self._write_ingest_section(f)
            self._write_query_overview(f, scenario_pages)
            self._write_footer(f)

    def _write_header(self, f, scenario_pages: Dict[str, str]):
        f.write("# Benchmark Report\n\n")
        f.write("This report summarises the performance benchmarks for encrypted database operations. "
                "Per-query-type detail lives on its own page — click through from the "
                "Query Performance section below.\n\n")
        f.write("## Table of Contents\n\n")
        f.write("1. [Ingest Throughput](#ingest-throughput)\n")

        # Add subsections for each ingest type
        ingest_types = sorted(set(r.bench_type for r in self.ingest_results))
        for it in ingest_types:
            title = it.replace('_', ' ').title()
            anchor = it.replace('_', '-')
            f.write(f"   - [{title}](#{anchor})\n")

        f.write("2. [Query Performance](#query-performance)\n")
        for qt in sorted(scenario_pages.keys()):
            f.write(f"   - [{qt} Queries]({scenario_pages[qt]})\n")

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
                "int_v3": "EQL v3 twin of `int`: same encrypt workload plus a from_v2 "
                          "v2→v3 conversion per payload, inserting into "
                          "`integer_encrypted_v3` (eql_v3.int4_ord_ore).",
                "string_v3": "EQL v3 twin of `string`, inserting into `string_encrypted_v3` "
                             "(eql_v3.text_search). NOTE: not directly comparable to "
                             "`string` — text_search requires the `ob` term, so this "
                             "workload encrypts an additional ORE index that v2's "
                             "encrypt_string does not.",
                "ste_vec_small_v3": "EQL v3 twin of `ste_vec_small`: same SteVec encrypt "
                                    "workload plus the from_v2 document conversion, "
                                    "inserting into `json_ste_vec_small_encrypted_v3` "
                                    "(eql_v3.json).",
                "convert_overhead_encrypt_only": "Conversion-overhead baseline: encrypt the "
                                                 "string_v3 workload (hm+bf+ob) with NO "
                                                 "conversion and NO database writes.",
                "convert_overhead_encrypt_convert": "Conversion-overhead treatment: identical "
                                                    "workload to `convert_overhead_encrypt_only` "
                                                    "plus a from_v2 v2→v3 conversion per payload "
                                                    "(still no database writes). The delta "
                                                    "between the two families is pure from_v2 "
                                                    "cost.",
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
        ax.set_ylim(bottom=0)

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
        ax.set_ylim(bottom=0)

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
        ax.set_ylim(bottom=0)

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
        ax.set_ylim(bottom=0)

        # Add "smaller is better" annotation
        ax.text(0.98, 0.98, 'smaller is better ↓', transform=ax.transAxes,
                fontsize=11, verticalalignment='top', horizontalalignment='right',
                bbox=dict(boxstyle='round', facecolor='lightblue', alpha=0.5))
        
        plt.tight_layout()
        plt.savefig(output_path, dpi=100, bbox_inches='tight')
        plt.close()

    def _write_query_overview(self, f, scenario_pages: Dict[str, str]):
        """Brief overview of the per-query-type pages on the index file.

        Picks the bits most useful for orienting a reader before they click
        through: which scenarios live under each type, which row-count tiers
        ran, and what the median timing looks like at the largest tier so the
        index gives an immediate sense of where the costs live.
        """
        f.write("## Query Performance\n\n")
        f.write("Per-query-type detail is broken out into separate pages — click into a "
                "scenario family for the SQL, per-tier timings, the indexes the planner "
                "picked, and the EXPLAIN plan tree. The EQL column is the version axis: "
                "`_V3` families run the same scenario intents against the `eql_v3` "
                "schema, and sort next to their v2 counterparts for side-by-side "
                "comparison.\n\n")
        f.write("| Query Type | EQL | Scenarios | Tiers | Largest-tier median (no decrypt) | Detail |\n")
        f.write("|-|-|-|-|-|-|\n")
        for qt in sorted(scenario_pages.keys()):
            type_results = [r for r in self.query_results if r.query_type == qt]
            if not type_results:
                continue
            # The sidecar-recorded version is the source of truth when the
            # family's sidecars agree; fall back to the (name-derived)
            # QueryResult version for families without metadata — criterion's
            # NDJSON carries no version field.
            meta_versions = {m.version for m in self.metadata.values()
                             if m.query_type == qt}
            if len(meta_versions) == 1:
                version = meta_versions.pop()
            else:
                version = type_results[0].version
            scenarios = sorted(set(r.query_name for r in type_results))
            tiers = sorted(set(r.row_count for r in type_results))
            tiers_str = ", ".join(f"{t:,}" for t in tiers)
            scenarios_str = ", ".join(f"`{s}`" for s in scenarios)
            # Median timing at the largest tier, averaged across scenarios for
            # a single-number summary. Not a substitute for the detail page;
            # just enough to flag "this family runs in seconds" vs "this one
            # is sub-millisecond".
            biggest = max(tiers)
            biggest_results = [r for r in type_results
                               if r.row_count == biggest and not r.decrypt]
            if biggest_results:
                med_ns = sum(r.median_ns for r in biggest_results) / len(biggest_results)
                med_str = self.format_time(med_ns, include_indicator=False)
            else:
                med_str = "—"
            page = scenario_pages[qt]
            f.write(f"| {qt} | v{version} | {scenarios_str} | {tiers_str} | {med_str} | [open]({page}) |\n")
        f.write("\n")

    def _write_query_type_page_content(self, f, query_type: str):
        """Write a self-contained per-query-type page."""
        f.write(f"# {query_type} Queries\n\n")
        f.write(f"[← Back to overview](./{self.output_file.name})\n\n")
        f.write("Per-tier query performance. Each scenario lists its SQL, the indexes "
                "available on the target table, the indexes the planner actually picked "
                "per tier, the timing table, and the full EXPLAIN plan in a collapsed "
                "block.\n\n")
        type_results = [r for r in self.query_results if r.query_type == query_type]
        query_names = sorted(set(r.query_name for r in type_results))
        for query_name in query_names:
            self._write_query_subsection(f, query_type, query_name, heading="##")

    def _write_query_subsection(self, f, query_type: str, query_name: str,
                                heading: str = "##"):
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

        f.write(f"{heading} {query_name}\n\n")
        f.write(f"**Description:** {description}\n\n")
        
        # Add SQL query and parameter
        if sql_query:
            f.write(f"**SQL Query:**\n```sql\n{sql_query}\n```\n\n")
            if param:
                f.write(f"**Parameter:** `{param}`\n\n")

        f.write(f"**{table_info}**\n\n")

        # Add index information for one of the row counts (they all use same indexes)
        if results:
            # Determine table name based on query type / scenario. The _V3
            # families map to the same base tables with a `_v3` infix
            # (e.g. string_encrypted_v3_10000).
            sample_row_count = results[0].row_count
            base_type = query_type
            v3_infix = ""
            if query_type.endswith("_V3"):
                base_type = query_type[:-len("_V3")]
                v3_infix = "_v3"
            if base_type == "GROUP_BY" and query_name.endswith("_plaintext"):
                # Plaintext baselines run against a plain TEXT column — no
                # functional EQL indexes; lookup will return None and the
                # Indexes block will be skipped. (v2 only — the v3 family
                # has no plaintext scenarios.)
                table_name = f"category_plaintext_{sample_row_count}"
            elif base_type == "GROUP_BY":
                # Encrypted GROUP BY scenarios run against the categorical
                # 250-bucket table family.
                table_name = f"category_encrypted{v3_infix}_{sample_row_count}"
            elif base_type in ["EXACT", "MATCH"]:
                # String-encrypted scenarios.
                table_name = f"string_encrypted{v3_infix}_{sample_row_count}"
            elif base_type == "ORE":
                table_name = f"integer_encrypted{v3_infix}_{sample_row_count}"
            elif base_type == "JSON":
                table_name = f"json_ste_vec_small_encrypted{v3_infix}_{sample_row_count}"
            elif base_type == "COMBO":
                table_name = f"combo_encrypted{v3_infix}_{sample_row_count}"
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
            # Sanitise `query_name` for filesystem use — multi-part scenario
            # IDs (e.g. `contains/functional`, `field_eq/bare`) carry a `/`
            # which the file system would interpret as a directory separator.
            safe_name = query_name.replace("/", "_")
            chart_path = self.output_file.parent / f"query_{query_type.lower()}_{safe_name}_chart.png"
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
        ax.set_ylim(bottom=0)
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
