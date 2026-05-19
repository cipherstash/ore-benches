#!/usr/bin/env python3
"""List benchmark scenarios whose median runtime exceeds a threshold.

Default threshold: 100 ms.

Scans `results/query/*_rows_*.json` (criterion's JSONL output stream),
extracts `benchmark-complete` events, and prints any whose median time
exceeds the threshold — sorted descending so the worst offenders surface
first.

Usage:
    python3 find_slow_queries.py                 # default 100 ms threshold
    python3 find_slow_queries.py --ms 250        # 250 ms threshold
    python3 find_slow_queries.py --results-dir other/path/results

Output is plain text (one row per slow query):

    [median_ms]  [rows]  [bench id]

So you can pipe through `wc -l` / `grep` / etc.
"""

import argparse
import json
import re
import sys
from pathlib import Path


def format_ms(ns: float) -> str:
    """Format nanoseconds as a fixed-width millisecond string."""
    return f"{ns / 1_000_000:>10.1f} ms"


def format_rows(n: int) -> str:
    """Right-align the row count with commas (1M → '1,000,000')."""
    return f"{n:>12,}"


def collect_slow(results_dir: Path, threshold_ns: float):
    """Walk `results_dir/query/*_rows_*.json`, return events above threshold.

    Returns a list of `(median_ns, rows, bench_id)` tuples — unsorted.
    """
    query_dir = results_dir / "query"
    if not query_dir.is_dir():
        sys.exit(f"no `query/` subdirectory under {results_dir}")

    filename_re = re.compile(r"^(.+)_rows_(\d+)\.json$")
    slow = []

    for path in sorted(query_dir.glob("*_rows_*.json")):
        m = filename_re.match(path.name)
        if not m:
            continue
        # The filename's row count is authoritative — criterion's
        # benchmark-complete events include the row count in the id but
        # parsing it out per-event is fragile. Pull from the filename.
        rows = int(m.group(2))

        with path.open() as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    event = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if event.get("reason") != "benchmark-complete":
                    continue
                median = event.get("median", {}).get("estimate")
                bench_id = event.get("id", "")
                if median is None or not bench_id:
                    continue
                if median > threshold_ns:
                    slow.append((median, rows, bench_id))

    return slow


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--ms", type=float, default=100.0,
                        help="threshold in milliseconds (default 100)")
    parser.add_argument("--results-dir", type=Path, default=Path("results"),
                        help="benchmark results root (default ./results)")
    args = parser.parse_args()

    threshold_ns = args.ms * 1_000_000

    slow = collect_slow(args.results_dir, threshold_ns)
    # Worst offenders first.
    slow.sort(key=lambda r: r[0], reverse=True)

    if not slow:
        print(f"No queries exceed {args.ms:g} ms.")
        return

    print(f"Queries with median runtime > {args.ms:g} ms "
          f"(scanned {args.results_dir}/query):")
    print()
    print(f"{'median':>13}  {'rows':>12}  scenario")
    print(f"{'-' * 13}  {'-' * 12}  {'-' * 60}")
    for median_ns, rows, bench_id in slow:
        print(f"{format_ms(median_ns)}  {format_rows(rows)}  {bench_id}")


if __name__ == "__main__":
    main()
