#!/usr/bin/env python3
"""List benchmark scenarios whose median runtime exceeds a threshold.

Default threshold: 100 ms.

Scans `results/query/*_rows_*.json` (criterion's JSONL output stream),
extracts `benchmark-complete` events, and prints any whose median time
exceeds the threshold — sorted descending so the worst offenders surface
first. The `last run` column comes from the result file's mtime, so you
can see at a glance how fresh each scenario's data is.

Usage:
    python3 find_slow_queries.py                 # default 100 ms threshold
    python3 find_slow_queries.py --ms 250        # 250 ms threshold
    python3 find_slow_queries.py --all           # every scenario, no threshold
    python3 find_slow_queries.py --all ORE       # only scenarios whose id starts with "ORE"
    python3 find_slow_queries.py --all EXACT/exact_decrypt   # narrower prefix
    python3 find_slow_queries.py --results-dir other/path/results

Output is plain text (one row per scenario):

    [median_ms]  [rows]  [last run]  [bench id]

So you can pipe through `wc -l` / `grep` / etc.
"""

import argparse
import json
import re
import sys
import time
from pathlib import Path


def format_ms(ns: float) -> str:
    """Format nanoseconds as a fixed-width millisecond string."""
    return f"{ns / 1_000_000:>10.1f} ms"


def format_rows(n: int) -> str:
    """Right-align the row count with commas (1M → '1,000,000')."""
    return f"{n:>12,}"


def format_age(mtime: float, now: float) -> str:
    """Render mtime-vs-now as a compact age ('12m ago', '3h ago', '2d ago')."""
    delta = max(0, int(now - mtime))
    if delta < 60:
        return f"{delta}s ago"
    if delta < 3600:
        return f"{delta // 60}m ago"
    if delta < 86400:
        return f"{delta // 3600}h ago"
    if delta < 86400 * 14:
        return f"{delta // 86400}d ago"
    return f"{delta // (86400 * 7)}w ago"


def collect_slow(results_dir, threshold_ns, prefix, v3=False):
    """Walk `results_dir/query/*_rows_*.json`, return events above threshold.

    With `v3=True`, scans `results_dir/query/v3/` instead — the EQL v3 bench
    results live in a subdirectory so the committed v2 baseline files stay
    untouched (see report_v3_compare.py).

    Returns a list of `(median_ns, rows, bench_id, mtime)` tuples — unsorted.
    Bench ids are filtered by `prefix` (str.startswith) when given.
    """
    query_dir = results_dir / "query" / "v3" if v3 else results_dir / "query"
    if not query_dir.is_dir():
        sys.exit(f"no `{query_dir}` directory")

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
        mtime = path.stat().st_mtime

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
                if prefix and not bench_id.startswith(prefix):
                    continue
                if median > threshold_ns:
                    slow.append((median, rows, bench_id, mtime))

    return slow


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--ms", type=float, default=100.0,
                        help="threshold in milliseconds (default 100)")
    parser.add_argument("--all", action="store_true",
                        help="list every scenario, ignoring the threshold")
    parser.add_argument("--results-dir", type=Path, default=Path("results"),
                        help="benchmark results root (default ./results)")
    parser.add_argument("--v3", action="store_true",
                        help="scan the EQL v3 results (results/query/v3/) instead of the v2 baseline")
    parser.add_argument("prefix", nargs="?", default=None,
                        help="if given, only include scenarios whose id starts with this prefix "
                             "(e.g. 'ORE' or 'EXACT/exact_decrypt')")
    args = parser.parse_args()

    # --all lists everything; a -inf threshold admits every (positive) median.
    threshold_ns = float("-inf") if args.all else args.ms * 1_000_000

    slow = collect_slow(args.results_dir, threshold_ns, args.prefix, v3=args.v3)
    # Worst offenders first.
    slow.sort(key=lambda r: r[0], reverse=True)

    prefix_note = f" matching prefix '{args.prefix}'" if args.prefix else ""
    scanned = f"{args.results_dir}/query/v3" if args.v3 else f"{args.results_dir}/query"

    if not slow:
        if args.all:
            print(f"No benchmark results found under {scanned}{prefix_note}.")
        else:
            print(f"No queries exceed {args.ms:g} ms{prefix_note}.")
        return

    if args.all:
        print(f"All {len(slow)} benchmark scenarios{prefix_note} "
              f"(scanned {scanned}), slowest first:")
    else:
        print(f"Queries with median runtime > {args.ms:g} ms{prefix_note} "
              f"(scanned {scanned}):")
    print()

    now = time.time()
    print(f"{'median':>13}  {'rows':>12}  {'last run':>9}  scenario")
    print(f"{'-' * 13}  {'-' * 12}  {'-' * 9}  {'-' * 60}")
    for median_ns, rows, bench_id, mtime in slow:
        print(f"{format_ms(median_ns)}  {format_rows(rows)}  "
              f"{format_age(mtime, now):>9}  {bench_id}")


if __name__ == "__main__":
    main()
