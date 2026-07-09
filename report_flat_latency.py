#!/usr/bin/env python3
"""CIP-3361: the cipherstash/stack README "flat latency" chart.

Generates a theme-aware light/dark SVG pair showing median encrypted-query
latency vs table size — three flat lines near the floor plus a subtle
plaintext-baseline band. Palette matches the stack README architecture
diagrams (lime + orange accents, warm ink/surfaces).

Data source: EQL v3 criterion results in results/query/v3/. Tiers are
auto-detected (the callout adapts: "10k → 1M" vs "10k → 10M" once the 10M
tier lands). Series:

    Equality             EXACT/exact/eql_cast            orange, circle
    JSON field equality  JSON/json/field_eq/functional   lime,   square
    Range + ORDER BY     ORE/ore/range_lt_ordered_10     ink,    triangle, dashed

Accessibility notes (brand palette is fixed; the validator flags the
lime↔orange pair in the CVD floor band and the ink line reads gray): every
series carries a DISTINCT marker shape and a direct end-of-line label, and
the ink line is dashed — identity never rides on hue alone. The plaintext
band is a fill with its own label.

Usage:
    python3 report_flat_latency.py [--out-dir report/v3]

Outputs perf-latency-light.svg / perf-latency-dark.svg. Copy into
cipherstash/stack docs/images/ as perf-latency-{light,dark}.svg with the
<picture> embed per docs/plans/readme-visual-assets.md (Asset 3).

Alt text (from the spec):
    Line chart of median encrypted-query latency versus table size.
    Equality, range, and JSON queries hold steady at well under one
    millisecond as row counts grow from ten thousand to ten million.
"""

import argparse
import json
import math
import sys
from pathlib import Path

V3_DIR = Path(__file__).parent / "results" / "query" / "v3"

SERIES = [
    # (label, criterion id prefix, color role, marker, dashed)
    ("Equality", "EXACT/exact/eql_cast", "accent2", "circle", False),
    ("JSON field equality", "JSON/json/field_eq/functional", "accent1", "square", False),
    ("Range + ORDER BY", "ORE/ore/range_lt_ordered_10", "ink", "triangle", True),
]

# Band = the two btree-indexed plaintext baselines (both flat ~0.1 ms). The
# json plaintext baseline is deliberately excluded: it is an unindexed
# LIMIT-scan whose growth with table size is an artifact of the baseline
# shape, not a fair "native Postgres" reference for this chart.
PLAINTEXT_PREFIXES = [
    "PLAINTEXT/plaintext/exact_eq",
    "PLAINTEXT/plaintext/range_lt_ordered_10",
]

THEMES = {
    "light": {
        "surface": "#f4f3ec",
        "ink": "#060606",
        "muted": "#6b6a63",
        "grid": "#e2e0d3",
        "accent1": "#5f7404",   # lime/olive
        "accent2": "#cc3d00",   # orange
        "band": "#e2e0d3",
    },
    "dark": {
        "surface": "#0d0d0d",
        "ink": "#eae8dd",
        "muted": "#8f8e85",
        "grid": "#242420",
        "accent1": "#c8f031",
        "accent2": "#ff5b1f",
        "band": "#1c1c18",
    },
}

TIER_LABELS = {10_000: "10k", 100_000: "100k", 1_000_000: "1M", 10_000_000: "10M"}


def load_medians():
    """{criterion_id: median_ms} across every v3 result file."""
    out = {}
    for f in sorted(V3_DIR.glob("*_rows_*.json")):
        for line in f.read_text().splitlines():
            if '"benchmark-complete"' not in line:
                continue
            try:
                d = json.loads(line)
            except json.JSONDecodeError:
                continue
            m = d.get("median", {}).get("estimate")
            if m is not None:
                out[d["id"]] = m / 1e6
    return out


def collect(medians):
    """Return (tiers, series_points, band) where series_points[label] and
    band are {tier: ms}. Tiers = those present in EVERY line series."""
    tiers = None
    series_points = {}
    for label, prefix, *_ in SERIES:
        pts = {}
        for t in TIER_LABELS:
            v = medians.get(f"{prefix}/{t}")
            if v is not None:
                pts[t] = v
        series_points[label] = pts
        tiers = set(pts) if tiers is None else tiers & set(pts)
    tiers = sorted(tiers or [])

    band = {}
    for t in tiers:
        vals = [medians[f"{p}/{t}"] for p in PLAINTEXT_PREFIXES if f"{p}/{t}" in medians]
        if vals:
            band[t] = (min(vals), max(vals))
    return tiers, series_points, band


def marker_svg(shape, x, y, color, surface):
    """Distinct marker per series (CVD relief), with a surface ring."""
    ring = f'stroke="{surface}" stroke-width="2"'
    if shape == "circle":
        return f'<circle cx="{x:.1f}" cy="{y:.1f}" r="4.5" fill="{color}" {ring}/>'
    if shape == "square":
        return (f'<rect x="{x - 4:.1f}" y="{y - 4:.1f}" width="8" height="8" '
                f'fill="{color}" {ring}/>')
    # triangle
    return (f'<path d="M {x:.1f} {y - 5:.1f} L {x + 5:.1f} {y + 4:.1f} '
            f'L {x - 5:.1f} {y + 4:.1f} Z" fill="{color}" {ring}/>')


def render(theme_name, tiers, series_points, band, out_path):
    c = THEMES[theme_name]
    W, H = 760, 420
    ML, MR, MT, MB = 64, 226, 76, 52
    plot_w, plot_h = W - ML - MR, H - MT - MB

    xs = {t: ML + plot_w * i / (len(tiers) - 1) for i, t in enumerate(tiers)}

    y_max = max(v for pts in series_points.values() for v in pts.values())
    if band:
        y_max = max(y_max, max(hi for _, hi in band.values()))
    y_max *= 1.35

    def Y(v):
        return MT + plot_h * (1 - v / y_max)

    top_label = TIER_LABELS[tiers[-1]]
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" '
        f'font-family="ui-sans-serif, -apple-system, \'Helvetica Neue\', Arial, sans-serif">',
        f'<rect width="{W}" height="{H}" fill="{c["surface"]}" rx="8"/>',
        # Callout (the headline claim)
        f'<text x="{ML}" y="34" font-size="19" font-weight="700" fill="{c["ink"]}">'
        f'Latency stays flat from 10k → {top_label} rows.</text>',
        f'<text x="{ML}" y="56" font-size="13" fill="{c["muted"]}">'
        f'Median encrypted query latency · EQL v3 · PostgreSQL 17</text>',
    ]

    # y gridlines + labels (recessive)
    step = 0.2 if y_max <= 1.2 else 0.5
    v = 0.0
    while v <= y_max + 1e-9:
        y = Y(v)
        parts.append(f'<line x1="{ML}" y1="{y:.1f}" x2="{ML + plot_w}" y2="{y:.1f}" '
                     f'stroke="{c["grid"]}" stroke-width="1"/>')
        parts.append(f'<text x="{ML - 8}" y="{y + 4:.1f}" font-size="12" '
                     f'text-anchor="end" fill="{c["muted"]}">{v:g}</text>')
        v += step
    parts.append(f'<text x="{ML - 44}" y="{MT + plot_h / 2:.1f}" font-size="12" '
                 f'fill="{c["muted"]}" transform="rotate(-90 {ML - 44} {MT + plot_h / 2:.1f})" '
                 f'text-anchor="middle">latency (ms)</text>')

    # x labels
    for t in tiers:
        parts.append(f'<text x="{xs[t]:.1f}" y="{MT + plot_h + 22}" font-size="13" '
                     f'text-anchor="middle" fill="{c["muted"]}">{TIER_LABELS[t]}</text>')
    parts.append(f'<text x="{ML + plot_w / 2:.1f}" y="{MT + plot_h + 40}" font-size="12" '
                 f'text-anchor="middle" fill="{c["muted"]}">rows (log scale)</text>')

    # plaintext band
    if band:
        top_pts = " ".join(f'{xs[t]:.1f},{Y(band[t][1]):.1f}' for t in tiers)
        bot_pts = " ".join(f'{xs[t]:.1f},{Y(band[t][0]):.1f}' for t in reversed(tiers))
        parts.append(f'<polygon points="{top_pts} {bot_pts}" fill="{c["band"]}" '
                     f'opacity="0.9"/>')
        # Label the band at its left edge, BELOW the band — above it sits the
        # equality line, and the right edge is where the series' direct
        # labels stack.
        t_first = tiers[0]
        parts.append(f'<text x="{xs[t_first] + 6:.1f}" y="{Y(band[t_first][0]) + 16:.1f}" '
                     f'font-size="12" fill="{c["muted"]}">plaintext Postgres</text>')

    # series lines + markers + direct labels
    label_ys = []
    for label, _, role, marker, dashed in SERIES:
        pts = series_points[label]
        color = c[role]
        dash = ' stroke-dasharray="7 5"' if dashed else ""
        path = " ".join(f'{xs[t]:.1f},{Y(pts[t]):.1f}' for t in tiers)
        parts.append(f'<polyline points="{path}" fill="none" stroke="{color}" '
                     f'stroke-width="2.5" stroke-linecap="round"{dash}/>')
        for t in tiers:
            parts.append(marker_svg(marker, xs[t], Y(pts[t]), color, c["surface"]))
        # direct label, collision-nudged
        t_last = tiers[-1]
        ly = Y(pts[t_last]) + 4
        while any(abs(ly - prev) < 16 for prev in label_ys):
            ly += 16
        label_ys.append(ly)
        parts.append(f'<text x="{xs[t_last] + 10:.1f}" y="{ly:.1f}" font-size="13" '
                     f'font-weight="600" fill="{color}">{label} '
                     f'({pts[t_last]:.2f} ms)</text>')

    # attribution caption
    parts.append(f'<text x="{W - 14}" y="{H - 12}" font-size="11" text-anchor="end" '
                 f'fill="{c["muted"]}">github.com/cipherstash/benches</text>')
    parts.append("</svg>")

    out_path.write_text("\n".join(parts))
    print(f"chart: {out_path}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out-dir", type=Path, default=Path("report/v3"))
    args = ap.parse_args()

    medians = load_medians()
    if not medians:
        sys.exit("no v3 results under results/query/v3/")
    tiers, series_points, band = collect(medians)
    if len(tiers) < 2:
        sys.exit(f"need >= 2 complete tiers across all series, found {tiers}")

    args.out_dir.mkdir(parents=True, exist_ok=True)
    for theme in THEMES:
        render(theme, tiers, series_points, band,
               args.out_dir / f"perf-latency-{theme}.svg")


if __name__ == "__main__":
    main()
