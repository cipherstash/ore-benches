#!/usr/bin/env python3
"""EQL v3 vs v2 comparison report + docs/marketing charts.

Joins criterion query results and hyperfine ingest results across the two
result trees:

    v2 (committed baseline):  results/query/*_rows_*.json
                              results/ingest/*_combined.json
    v3 (this branch's runs):  results/query/v3/*_rows_*.json
                              results/ingest/v3/*_combined.json

and emits:

    report/V3_COMPARISON.md   regression tables + index-engagement audit
    report/v3/*.png|.svg      charts (v3-vs-v2, encrypted-vs-plaintext
                              overhead, ORE-vs-OPE)

Usage:
    python3 report_v3_compare.py [--threshold 10]

`--threshold` is the regression flag threshold in percent (v3 slower than v2
by more than this on a comparable scenario ⇒ REGRESSION).

Scenario ids are shared between versions by construction (the v3 benches
reuse the v2 criterion ids), so the join key is the full criterion id,
e.g. ``EXACT/exact/eql_cast/10000``. Scenarios whose SQL semantics changed
between versions are annotated from SEMANTICS_CHANGED rather than flagged.
"""

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).parent
V2_QUERY_DIR = ROOT / "results" / "query"
V3_QUERY_DIR = ROOT / "results" / "query" / "v3"
V2_INGEST_DIR = ROOT / "results" / "ingest"
V3_INGEST_DIR = ROOT / "results" / "ingest" / "v3"
REPORT_DIR = ROOT / "report"
CHART_DIR = REPORT_DIR / "v3"

# Scenario-id prefixes whose v2/v3 SQL differs in meaning, with the note the
# report shows instead of a bare REGRESSION flag.
SEMANTICS_CHANGED = {
    "MATCH/match/eql_cast_firstname": "v2 LIKE → v3 @> (no LIKE operator in v3; same bloom semantics)",
    "MATCH/match/eql_cast_lastname": "v2 LIKE → v3 @> (no LIKE operator in v3; same bloom semantics)",
    "MATCH/match_decrypt/eql_cast_firstname": "v2 LIKE → v3 @>",
    "MATCH/match_decrypt/eql_cast_lastname": "v2 LIKE → v3 @>",
    "EXACT/exact/eql_hash": "index type changed: v2 hash → v3 btree on eq_term",
    "EXACT/exact_decrypt/eql_hash": "index type changed: v2 hash → v3 btree on eq_term",
    "COMBO/combo/bloom_ore_order_limit": "v2 LIKE → v3 @>",
    "COMBO/combo/filtered_group_by": "v2 LIKE → v3 @>",
    "COMBO/combo/top_n_filtered_group_by": "v2 LIKE → v3 @>",
    # DIAGNOSED bench artifact, not a v3 defect: the sampled needle matches
    # EVERY row (selector-lookup hm — the only hm-bearing sv entry in the
    # 2.3+ wire format). v3 extractor now runs eql_v3.jsonb_contains (the
    # jsonb_array GIN recipe, single-entry needle), replacing v2's typed
    # stevec_query GIN. sqlx's prepared statements flip to a generic plan
    # after 5 executions; on the every-row needle the plan is unstable —
    # sometimes an early-stop seq scan (~0.6ms), sometimes a full-posting-list
    # GIN bitmap before LIMIT (tens of ms), which is why the number swings
    # non-monotonically across tiers (e.g. 72ms @1M vs 0.6ms @10M). With a
    # realistic selective needle the GIN bitmap is small and the plan is
    # optimal.
    "JSON/json/field_eq/extractor": "v2 stevec_query GIN → v3 jsonb_contains; every-row-needle plan instability (see notes)",
}

# Extra context notes keyed the same way (shown alongside numbers).
NOTES = {
    "EXACT/exact/eql_cast": "v3 string rows are wider (text_search adds an ORE term v2 didn't carry)",
    "MATCH/match/eql_bloom": "v3 GIN uses native array_ops on the bloom term (no shipped opclass)",
    # The sampled field_eq needle is the selector-lookup entry's hm (the only
    # hm-bearing sv entry in the 2.3+ wire format) — it matches EVERY row, in
    # v2 and v3 alike, so these scenarios measure "fetch first 10 of
    # everything", not selective equality. Plans are identical across
    # versions (extractor: seq scan in both; bare/functional: btree in
    # both) — deltas isolate per-row extractor cost.
    "JSON/json/field_eq/bare": "same btree plan as v2 (needle matches every row in both versions)",
    "JSON/json/field_eq/functional": "same btree plan as v2 (needle matches every row in both versions)",
    # v3 now runs eql_v3.jsonb_contains → jsonb_array(value) @> jsonb_array(needle),
    # the SAME recipe/GIN as v2 (no longer semantics-changed). The 10M delta
    # reflects GIN posting-list growth over the public.json domain vs the v2
    # composite, not a recipe change.
    "JSON/json/contains/functional": "same jsonb_array GIN recipe as v2; 10M delta is posting-list/data-shape",
    # Same-day v2 re-measurement (2026-07-04) adjudicated the 10M flags vs
    # the May-measured baseline: cross-session drift is ±8-19% at these
    # scales. v3 vs SAME-DAY v2: eql_cast -3.3%, top_n GROUP BY -0.7%,
    # low-cardinality GROUP BY +6.4% — parity within threshold.
    "EXACT/exact/eql_cast": "10M flag adjudicated as baseline drift: v3 is 3.3% FASTER than same-day v2",
    "GROUP_BY/group_by/low_cardinality_groups_encrypted": "10M flag adjudicated as baseline drift: +6.4% vs same-day v2",
    "GROUP_BY/group_by/top_n_groups_encrypted": "10M flag adjudicated as baseline drift: parity (-0.7%) vs same-day v2",
}

# Notes for the ingest comparison, keyed by v3 bench name.
INGEST_NOTES = {
    "encrypt_string_v3": (
        "NOT a conversion regression: v3's only eq+match text domain "
        "(text_search) also REQUIRES the ORE term, so the config adds "
        "Index::new_ore() that v2's unique+match didn't carry — string "
        "ingest is capped at ORE-generation speed. A v3 hm+bf-only domain "
        "would restore v2 throughput."
    ),
    "encrypt_int_ope_v3": (
        "real CLLW-OPE op terms (cipherstash-client 0.38.1, Index::new_ope)"
    ),
}

# v3 scenarios whose plan MUST show an index; empty indexes_used ⇒ audit FAIL.
# (Non-selective ORE/OPE range scans may legitimately seq-scan at scale, so
# only the ordered/eq/json/match shapes are gated.)
EXPECTED_INDEX_PATTERNS = [
    r"^EXACT/exact/",
    r"^ORE/ore/range_lt_ordered_10/",
    r"^OPE/ope/range_lt_ordered_10/",
    r"^MATCH/match/(?!.*_noindex)",
    # field_eq/extractor is exempt: its needle matches every row (selector-
    # lookup hm), so a seq scan is the CORRECT plan — v2 planned it the same
    # way. Gated scenarios are the selective/ordered JSON shapes.
    r"^JSON/json/(contains|field_order)/",
    r"^JSON/json/field_eq/(bare|functional)/",
    r"^SMOKE_V3/smoke/.*/range_gt_ordered_10/",
]

# Encrypted ↔ plaintext scenario mapping for the overhead chart/table.
# encrypted id prefix -> plaintext scenario name
PLAINTEXT_MAP = {
    "EXACT/exact/eql_cast": "exact_eq",
    "ORE/ore/range_gt_10": "range_gt_10",
    "ORE/ore/range_lt_ordered_10": "range_lt_ordered_10",
    "OPE/ope/range_gt_10": "range_gt_10",
    "OPE/ope/range_lt_ordered_10": "range_lt_ordered_10",
    "JSON/json/contains/functional": "json_contains",
    "JSON/json/field_eq/bare": "json_field_eq",
    "GROUP_BY/group_by/low_cardinality_groups_encrypted": "@GROUP_BY/group_by/low_cardinality_groups_plaintext",
    "GROUP_BY/group_by/top_n_groups_encrypted": "@GROUP_BY/group_by/top_n_groups_plaintext",
}

# Chart colors — validated 3-slot categorical palette (dataviz skill,
# reference palette slots 1-3; light surface). Aqua/yellow sit below 3:1 on
# the surface, so every bar carries a direct value label.
COLOR_V2 = "#2a78d6"        # blue   — EQL v2 baseline
COLOR_V3 = "#1baf7a"        # aqua   — EQL v3
COLOR_PLAIN = "#a8a69e"     # neutral — plaintext baseline (identity is
                            # 'no encryption', gray keeps it recessive)
COLOR_ORE = "#2a78d6"       # blue   — ORE path
COLOR_OPE = "#1baf7a"       # aqua   — OPE path
SURFACE = "#fcfcfb"
TEXT_PRIMARY = "#0b0b0b"
TEXT_SECONDARY = "#52514e"
GRID = "#e4e2dc"


def load_criterion_dir(path: Path) -> dict:
    """{criterion_id: median_ns} across every *_rows_*.json in `path`."""
    out = {}
    for f in sorted(path.glob("*_rows_*.json")):
        for line in f.read_text().splitlines():
            line = line.strip()
            if not line:
                continue
            try:
                data = json.loads(line)
            except json.JSONDecodeError:
                continue
            if data.get("reason") != "benchmark-complete":
                continue
            median = data.get("median", {}).get("estimate")
            if median is not None:
                out[data["id"]] = median
    return out


def load_metadata_dir(path: Path) -> dict:
    """{scenario_id: {query, indexes_used, rows_returned}}."""
    out = {}
    for f in sorted(path.glob("*_metadata_*.json")):
        data = json.loads(f.read_text())
        for s in data.get("scenarios", []):
            out[s["id"]] = {
                "query": s.get("query", ""),
                "indexes_used": s.get("indexes_used", []),
                "rows_returned": s.get("rows_returned", 0),
            }
    return out


def load_ingest_dir(path: Path) -> dict:
    """{bench_name: {num_records: throughput_rps}}."""
    out = {}
    for f in sorted(path.glob("*_combined.json")):
        data = json.loads(f.read_text())
        name = f.name.replace("_combined.json", "")
        out[name] = {
            r["num_records"]: r["throughput_records_per_second"]
            for r in data.get("results", [])
        }
    return out


def scenario_prefix(cid: str) -> str:
    """Strip the trailing tier segment from a criterion id."""
    return cid.rsplit("/", 1)[0]


def tier_of(cid: str) -> str:
    return cid.rsplit("/", 1)[1]


def fmt_ns(ns: float) -> str:
    if ns >= 1e9:
        return f"{ns / 1e9:.2f} s"
    if ns >= 1e6:
        return f"{ns / 1e6:.2f} ms"
    if ns >= 1e3:
        return f"{ns / 1e3:.1f} µs"
    return f"{ns:.0f} ns"


def semantics_note(cid: str):
    return SEMANTICS_CHANGED.get(scenario_prefix(cid))


def build_regression_rows(v2: dict, v3: dict, threshold: float):
    rows = []
    for cid in sorted(set(v2) & set(v3)):
        d = (v3[cid] - v2[cid]) / v2[cid] * 100.0
        note = semantics_note(cid)
        flag = ""
        if note:
            flag = "semantics changed"
        elif "_decrypt/" in cid:
            # Decrypt variants are dominated by ZeroKMS round-trips (~25 ms)
            # — cross-run network variance alone exceeds any sane threshold,
            # and the v2 baseline was measured under a different client
            # version. Informational only; never flagged.
            flag = ""
        elif d > threshold:
            flag = "REGRESSION"
        elif d < -threshold:
            flag = "improvement"
        rows.append(
            {
                "id": cid,
                "v2_ns": v2[cid],
                "v3_ns": v3[cid],
                "delta_pct": d,
                "flag": flag,
                "note": note or NOTES.get(scenario_prefix(cid), ""),
            }
        )
    return rows


# --- Charts ---------------------------------------------------------------


def _style_axes(ax):
    ax.set_facecolor(SURFACE)
    for spine in ("top", "right", "left"):
        ax.spines[spine].set_visible(False)
    ax.spines["bottom"].set_color(GRID)
    ax.tick_params(colors=TEXT_SECONDARY, labelsize=9)
    ax.xaxis.grid(True, color=GRID, linewidth=0.6)
    ax.set_axisbelow(True)


def _save(fig, name: str):
    CHART_DIR.mkdir(parents=True, exist_ok=True)
    for ext in ("png", "svg"):
        fig.savefig(CHART_DIR / f"{name}.{ext}", dpi=150, bbox_inches="tight",
                    facecolor=SURFACE)
    print(f"chart: report/v3/{name}.png (+.svg)")


def grouped_hbar(pairs, series_names, series_colors, title, name, unit="ms"):
    """Horizontal grouped bars. pairs = [(label, [val_per_series…]), …]."""
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    import numpy as np

    n_series = len(series_names)
    height = max(2.2, 0.42 * len(pairs) * n_series + 1.2)
    fig, ax = plt.subplots(figsize=(9, height))
    fig.patch.set_facecolor(SURFACE)
    _style_axes(ax)

    y = np.arange(len(pairs))
    bar_h = 0.78 / n_series
    for i, (sname, color) in enumerate(zip(series_names, series_colors)):
        vals = [p[1][i] for p in pairs]
        offs = y + (i - (n_series - 1) / 2) * (bar_h + 0.04)
        bars = ax.barh(offs, vals, height=bar_h, color=color, label=sname,
                       zorder=3)
        # Direct value labels (relief for below-3:1 palette slots).
        for b, v in zip(bars, vals):
            if v is None:
                continue
            ax.text(b.get_width() * 1.01, b.get_y() + b.get_height() / 2,
                    f"{v:,.2f}" if v < 100 else f"{v:,.0f}",
                    va="center", ha="left", fontsize=8, color=TEXT_PRIMARY)

    ax.set_yticks(y)
    ax.set_yticklabels([p[0] for p in pairs], fontsize=9, color=TEXT_PRIMARY)
    ax.invert_yaxis()
    ax.set_xlabel(f"median latency ({unit}) — lower is better",
                  fontsize=9, color=TEXT_SECONDARY)
    # Title on its own band, legend in the gap beneath it — neither can
    # collide with the other or with long bars.
    ax.set_title(title, fontsize=11, color=TEXT_PRIMARY, loc="left", pad=34)
    ax.legend(frameon=False, fontsize=9, labelcolor=TEXT_PRIMARY,
              loc="lower right", bbox_to_anchor=(1.0, 1.0), ncol=2)
    _save(fig, name)
    plt.close(fig)


def chart_v3_vs_v2(rows, tier: str):
    pairs = []
    for r in rows:
        if tier_of(r["id"]) != tier or "_decrypt/" in r["id"]:
            continue
        label = scenario_prefix(r["id"])
        pairs.append((label, [r["v2_ns"] / 1e6, r["v3_ns"] / 1e6]))
    if not pairs:
        return
    grouped_hbar(pairs, ["EQL v2.3", "EQL v3"], [COLOR_V2, COLOR_V3],
                 f"EQL v3 vs v2.3 — query medians at {int(tier):,} rows",
                 f"v3_vs_v2_{tier}")


def chart_plaintext_overhead(v3: dict, tier: str):
    pairs = []
    for enc_prefix, plain in PLAINTEXT_MAP.items():
        enc_id = f"{enc_prefix}/{tier}"
        if plain.startswith("@"):
            plain_id = f"{plain[1:]}/{tier}"
        else:
            plain_id = f"PLAINTEXT/plaintext/{plain}/{tier}"
        if enc_id in v3 and plain_id in v3:
            pairs.append(
                (enc_prefix, [v3[plain_id] / 1e6, v3[enc_id] / 1e6])
            )
    if not pairs:
        return
    grouped_hbar(pairs, ["plaintext Postgres", "EQL v3 encrypted"],
                 [COLOR_PLAIN, COLOR_V3],
                 f"Encrypted vs plaintext — same query shape, {int(tier):,} rows",
                 f"overhead_vs_plaintext_{tier}")


def chart_ore_vs_ope(v3: dict, tier: str):
    pairs = []
    for cid in sorted(v3):
        m = re.match(r"^ORE/ore/([^/]+)/" + re.escape(tier) + "$", cid)
        if not m:
            continue
        ope_id = f"OPE/ope/{m.group(1)}/{tier}"
        if ope_id in v3:
            pairs.append((m.group(1), [v3[cid] / 1e6, v3[ope_id] / 1e6]))
    if not pairs:
        return
    grouped_hbar(pairs, ["ORE (block, custom opclass)", "OPE (CLLW, native btree)"],
                 [COLOR_ORE, COLOR_OPE],
                 f"v3 ordering paths: ORE vs OPE at {int(tier):,} rows",
                 f"ore_vs_ope_{tier}")


# --- Report ---------------------------------------------------------------


def write_report(rows, v3_query, v3_meta, v2_ingest, v3_ingest,
                 threshold: float, tiers: list):
    REPORT_DIR.mkdir(exist_ok=True)
    out = REPORT_DIR / "V3_COMPARISON.md"
    lines = []
    w = lines.append

    w("# EQL v3 vs v2.3 — Benchmark Comparison")
    w("")
    w("Auto-generated by `report_v3_compare.py` (`mise run report:v3-compare`).")
    w("")
    w(f"Regression threshold: v3 slower by more than **{threshold:.0f}%** on a "
      "semantics-equivalent scenario.")
    w("")
    w("Methodology notes:")
    w("")
    w("- EQL v3 is installed from the pinned release bundle "
      "**eql-3.0.0-alpha.3** (`mise run setup-db-v3`). alpha.3 places the "
      "per-domain types in `public.*` (`public.text_search`, "
      "`public.integer_ord`, `public.json`, …); the raw-jsonb SEM extractors "
      "live in `eql_v3_internal`, and the benches call only the public "
      "`eql_v3.*` wrappers.")
    w("- **Re-baseline status:** the **JSON** tiers (10k–10M) were re-ingested "
      "and re-run against alpha.3. The **non-JSON** v3 scenarios below are "
      "still the pre-release alpha.2-equivalent measurements and are pending a "
      "full alpha.3 re-baseline — treat their absolute numbers accordingly.")
    w("- v3 payloads are produced by `eql-bindings::from_v2` over the pinned "
      "cipherstash-client's v2.3 output (the supported migration path); "
      "conversion cost is inside the measured ingest path.")
    w("- The JSON bench queries the encrypted document through the named EQL "
      "JSON functions — `eql_v3.jsonb_contains` (GIN over "
      "`eql_v3.jsonb_array(value)`) for containment and "
      "`eql_v3.jsonb_path_query_first(value, selector)` feeding "
      "`eql_v3.eq_term` / `eql_v3.ore_cllw` for field equality/ordering — not "
      "raw `jsonb` `@>` / `->`.")
    w("- v3 query parameters are stored-shape payloads (no v3 scalar query "
      "wire shape exists); server-side timings are unaffected.")
    w("- The v3 string column (`text_search`) carries an ORE term v2's "
      "unique+match config did not — v3 string rows are wider.")
    w("- OPE ciphertexts are real CLLW-OPE terms emitted by "
      "cipherstash-client 0.38.1 (Index::new_ope); ordering parity against "
      "decrypted plaintext is asserted at bench startup.")
    w("- v2 numbers are the committed baseline results (`results/query/`, "
      "`results/ingest/`); v3 numbers come from `results/query/v3/`, "
      "`results/ingest/v3/`. The baseline was measured in May; same-day v2 "
      "re-measurement (2026-07-04) shows ±8-19% cross-session drift at the "
      "10M tier, so 10M flags were re-adjudicated against same-day v2 — "
      "see the per-row notes.")
    w("")

    # Regression table
    w("## Query scenarios: v3 vs v2")
    w("")
    if rows:
        n_reg = sum(1 for r in rows if r["flag"] == "REGRESSION")
        n_imp = sum(1 for r in rows if r["flag"] == "improvement")
        w(f"{len(rows)} comparable scenario/tier pairs — "
          f"**{n_reg} regressions**, {n_imp} improvements beyond ±{threshold:.0f}%.")
        w("")
        w("| Scenario | Tier | v2 median | v3 median | Δ | Flag | Note |")
        w("|---|---|---|---|---|---|---|")
        for r in sorted(rows, key=lambda r: -r["delta_pct"]):
            flag = {"REGRESSION": "🔴 REGRESSION",
                    "improvement": "🟢 improvement",
                    "semantics changed": "⚠️ semantics changed"}.get(r["flag"], "")
            w(f"| {scenario_prefix(r['id'])} | {tier_of(r['id'])} "
              f"| {fmt_ns(r['v2_ns'])} | {fmt_ns(r['v3_ns'])} "
              f"| {r['delta_pct']:+.1f}% | {flag} | {r['note']} |")
    else:
        w("_No overlapping scenario ids between v2 and v3 result sets yet._")
    w("")

    # v3-only scenarios
    v3_only = sorted(
        cid for cid in v3_query
        if cid.startswith(("OPE/", "SMOKE_V3/", "PLAINTEXT/"))
        or "_noindex/" in cid
    )
    if v3_only:
        w("## v3-only scenarios (no v2 counterpart)")
        w("")
        w("| Scenario | Median |")
        w("|---|---|")
        for cid in v3_only:
            w(f"| {cid} | {fmt_ns(v3_query[cid])} |")
        w("")

    # Index engagement audit
    w("## Index engagement audit (v3 plans)")
    w("")
    w("Scenarios matching an expected-index rule must show a non-empty "
      "`indexes_used` in their EXPLAIN capture.")
    w("")
    w("| Scenario | Indexes used | Rows | Status |")
    w("|---|---|---|---|")
    failures = 0
    for cid in sorted(v3_meta):
        meta = v3_meta[cid]
        expected = any(re.search(p, cid) for p in EXPECTED_INDEX_PATTERNS)
        idx = ", ".join(meta["indexes_used"]) or "—"
        if expected and not meta["indexes_used"]:
            status = "❌ expected index, none used"
            failures += 1
        elif expected:
            status = "✅"
        else:
            status = ""
        w(f"| {cid} | {idx} | {meta['rows_returned']} | {status} |")
    w("")
    if failures:
        w(f"**{failures} scenario(s) failed the index-engagement gate.**")
        w("")

    # Ingest comparison
    w("## Ingest throughput: v3 vs v2")
    w("")
    matched = False
    w("| Bench | Records | v2 rec/s | v3 rec/s | Δ | Note |")
    w("|---|---|---|---|---|---|")
    for v3_name, v3_res in sorted(v3_ingest.items()):
        v2_name = v3_name.replace("_v3", "")
        v2_res = v2_ingest.get(v2_name, {})
        note = INGEST_NOTES.get(v3_name, "")
        for n, v3_rps in sorted(v3_res.items()):
            v2_rps = v2_res.get(n)
            if v2_rps:
                matched = True
                d = (v3_rps - v2_rps) / v2_rps * 100.0
                w(f"| {v3_name} | {n:,} | {v2_rps:,.0f} | {v3_rps:,.0f} | {d:+.1f}% | {note} |")
            else:
                w(f"| {v3_name} | {n:,} | — | {v3_rps:,.0f} | | {note} |")
            note = ""
    if not matched:
        w("")
        w("_No matching v2 ingest baselines for the v3 benches run so far._")
    w("")

    # Index build times
    build_log = V2_INGEST_DIR / "index_build_times.jsonl"
    if build_log.exists():
        w("## Index build times")
        w("")
        w("| Table | Build seconds | Recorded |")
        w("|---|---|---|")
        for line in build_log.read_text().splitlines():
            try:
                e = json.loads(line)
                w(f"| {e['table']} | {e['build_seconds']} | {e['recorded_at']} |")
            except (json.JSONDecodeError, KeyError):
                continue
        w("")

    w("## Charts")
    w("")
    for tier in tiers:
        for base in (f"v3_vs_v2_{tier}", f"overhead_vs_plaintext_{tier}",
                     f"ore_vs_ope_{tier}"):
            if (CHART_DIR / f"{base}.png").exists():
                w(f"![{base}](v3/{base}.png)")
                w("")

    out.write_text("\n".join(lines))
    print(f"report: {out}")
    return failures


def print_cli_table(rows, v3_query, threshold: float):
    """Side-by-side v2/v3 terminal view — `mise run report:v3-compare`.

    The Markdown/chart artifacts are `report:build:v3-compare`'s job; this
    prints the same join to stdout for quick triage after a bench run.
    """
    flag_names = {"REGRESSION": "REGRESSION", "improvement": "improvement",
                  "semantics changed": "semantics≠"}
    id_w = max((len(scenario_prefix(r["id"])) for r in rows), default=20)
    print(f"{'scenario':<{id_w}}  {'tier':>8}  {'v2 median':>11}  "
          f"{'v3 median':>11}  {'Δ':>8}  flag")
    print(f"{'-' * id_w}  {'-' * 8}  {'-' * 11}  {'-' * 11}  {'-' * 8}  {'-' * 11}")
    for r in sorted(rows, key=lambda r: (scenario_prefix(r["id"]), int(tier_of(r["id"])))):
        print(f"{scenario_prefix(r['id']):<{id_w}}  {tier_of(r['id']):>8}  "
              f"{fmt_ns(r['v2_ns']):>11}  {fmt_ns(r['v3_ns']):>11}  "
              f"{r['delta_pct']:+7.1f}%  {flag_names.get(r['flag'], '')}")

    n_reg = sum(1 for r in rows if r["flag"] == "REGRESSION")
    n_imp = sum(1 for r in rows if r["flag"] == "improvement")
    n_sem = sum(1 for r in rows if r["flag"] == "semantics changed")
    print()
    print(f"{len(rows)} comparable pairs: {n_reg} regressions, {n_imp} "
          f"improvements (beyond ±{threshold:.0f}%), {n_sem} semantics-changed.")

    v3_only = sorted(
        cid for cid in v3_query
        if scenario_prefix(cid) not in {scenario_prefix(r["id"]) for r in rows}
        and "_decrypt" not in cid
    )
    if v3_only:
        print()
        print("v3-only scenarios (no v2 counterpart):")
        for cid in v3_only:
            print(f"  {cid:<{id_w + 10}}  {fmt_ns(v3_query[cid]):>11}")
    print()
    print("Full report + charts: mise run report:build:v3-compare")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--threshold", type=float, default=10.0,
                    help="regression flag threshold in percent (default 10)")
    ap.add_argument("--cli", action="store_true",
                    help="print the v2/v3 side-by-side table to stdout and exit "
                         "(no Markdown report, no charts)")
    args = ap.parse_args()

    v2_query = load_criterion_dir(V2_QUERY_DIR)
    v3_query = load_criterion_dir(V3_QUERY_DIR)
    v3_meta = load_metadata_dir(V3_QUERY_DIR)
    v2_ingest = load_ingest_dir(V2_INGEST_DIR)
    v3_ingest = load_ingest_dir(V3_INGEST_DIR)

    if not v3_query:
        print("No v3 query results found under results/query/v3/ — run "
              "`mise run bench:v3:query:all` first.", file=sys.stderr)
        sys.exit(1)

    rows = build_regression_rows(v2_query, v3_query, args.threshold)

    if args.cli:
        print_cli_table(rows, v3_query, args.threshold)
        return

    tiers = sorted({tier_of(cid) for cid in v3_query}, key=int)

    try:
        for tier in tiers:
            chart_v3_vs_v2(rows, tier)
            chart_plaintext_overhead(v3_query, tier)
            chart_ore_vs_ope(v3_query, tier)
    except ImportError:
        print("matplotlib not available — skipping charts", file=sys.stderr)

    failures = write_report(rows, v3_query, v3_meta, v2_ingest, v3_ingest,
                            args.threshold, tiers)
    sys.exit(2 if failures else 0)


if __name__ == "__main__":
    main()
