#!/usr/bin/env python3
"""Overview of ingest benchmark results — throughput per scenario, highest first.

Scans `results/ingest/*_combined.json` (one per ingest bench, produced
by `combine_benchmark`), flattens the inner `results[]` (one entry per
num_records run), and prints them sorted by throughput descending so
the headline numbers surface first. The `last run` column comes from
the result file's mtime.

Usage:
    python3 report_ingest.py                       # all ingest results
    python3 report_ingest.py encrypt_json          # filter by name prefix
    python3 report_ingest.py encrypt_int           # narrower prefix
    python3 report_ingest.py --results-dir other/path/results

Output is plain text (one row per scenario):

    [throughput]  [records]  [time]  [last run]  [bench]
"""

import argparse
import json
import sys
import time
from pathlib import Path


def format_throughput(rec_per_sec: float) -> str:
    """Format records/sec as a fixed-width string with thousands separators."""
    return f"{int(round(rec_per_sec)):>10,} rec/s"


def format_records(n: int) -> str:
    """Right-align the record count with commas (10M → '10,000,000')."""
    return f"{n:>11,}"


def format_seconds(s: float) -> str:
    """Format wall-clock seconds at 3 decimals (sub-second resolution)."""
    return f"{s:>7.3f} s"


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


def bench_version(bench: str) -> int:
    """EQL version axis for an ingest bench name. `_v3`-suffixed benches
    write v3 payloads; `convert_overhead_encrypt_convert` performs the
    v2→v3 conversion (its `encrypt_only` twin is the v2-shaped baseline).
    Everything else — including all pre-existing result files — is v2.
    """
    if bench.endswith("_v3") or bench == "convert_overhead_encrypt_convert":
        return 3
    return 2


def collect_ingest(results_dir, prefix):
    """Walk `results_dir/ingest/*_combined.json` and flatten the inner results.

    Returns a list of `(throughput, records, total_time, mtime, bench)`
    tuples — unsorted. Bench name (the filename minus `_combined`) is
    filtered by `prefix` (str.startswith) when given.
    """
    ingest_dir = results_dir / "ingest"
    if not ingest_dir.is_dir():
        sys.exit(f"no `ingest/` subdirectory under {results_dir}")

    rows = []

    for path in sorted(ingest_dir.glob("*_combined.json")):
        bench = path.stem
        if bench.endswith("_combined"):
            bench = bench[:-len("_combined")]
        if prefix and not bench.startswith(prefix):
            continue

        mtime = path.stat().st_mtime
        try:
            with path.open() as f:
                doc = json.load(f)
        except (OSError, json.JSONDecodeError) as e:
            print(f"warning: skipping {path}: {e}", file=sys.stderr)
            continue

        for r in doc.get("results", []):
            throughput = r.get("throughput_records_per_second")
            records = r.get("num_records")
            total_time = r.get("total_time_seconds")
            if throughput is None or records is None or total_time is None:
                continue
            rows.append((throughput, records, total_time, mtime, bench))

    return rows


def main():
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--results-dir", type=Path, default=Path("results"),
                        help="benchmark results root (default ./results)")
    parser.add_argument("prefix", nargs="?", default=None,
                        help="if given, only include benches whose name starts with this prefix "
                             "(e.g. 'encrypt_int', 'encrypt_json')")
    args = parser.parse_args()

    rows = collect_ingest(args.results_dir, args.prefix)
    # Highest throughput first.
    rows.sort(key=lambda r: r[0], reverse=True)

    prefix_note = f" matching prefix '{args.prefix}'" if args.prefix else ""

    if not rows:
        print(f"No ingest results found under {args.results_dir}/ingest{prefix_note}.")
        return

    print(f"All {len(rows)} ingest scenarios{prefix_note} "
          f"(scanned {args.results_dir}/ingest), highest throughput first:")
    print()

    now = time.time()
    print(f"{'throughput':>16}  {'records':>11}  {'time':>9}  {'last run':>9}  {'eql':>3}  bench")
    print(f"{'-' * 16}  {'-' * 11}  {'-' * 9}  {'-' * 9}  {'-' * 3}  {'-' * 40}")
    for throughput, records, total_time, mtime, bench in rows:
        print(f"{format_throughput(throughput)}  {format_records(records)}  "
              f"{format_seconds(total_time)}  {format_age(mtime, now):>9}  "
              f"{'v' + str(bench_version(bench)):>3}  {bench}")


if __name__ == "__main__":
    main()
