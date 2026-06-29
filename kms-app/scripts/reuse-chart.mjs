#!/usr/bin/env node
// Render the data-key REUSE experiment into a dependency-free SVG: two panels of
// grouped bars (sequential vs scattered) over the three backends. Left panel =
// KMS calls per 50-record query (the mechanism); right panel = p95 latency. The
// story: reuse is cheap sequentially but collapses to ~one KMS call per record
// when the read pattern scatters; ZeroKMS is flat. Usage: node scripts/reuse-chart.mjs
import { readFileSync, writeFileSync } from "node:fs";

const DIR = process.env.REUSE_DIR || "results-ec2/reuse";
const rows = readFileSync(`${DIR}/data.csv`, "utf8")
  .trim().split("\n").slice(1)
  .map((l) => {
    const [phase, backend, maxuses, pattern, p95, failed, kms] = l.split(",");
    return { phase, backend, maxuses, pattern, p95: +p95, failed: +failed, kms: +kms };
  });

// Three query "backends" in display order; reuse vs per-value are both envelope.
const SERIES = [
  { label: "ZeroKMS", color: "#16a34a", match: (r) => r.backend === "zerokms" },
  { label: "AWS envelope\n+ reuse", color: "#7c3aed", match: (r) => r.backend === "aws-kms-envelope" && r.maxuses === "300" },
  { label: "AWS envelope\nper-value", color: "#ea580c", match: (r) => r.backend === "aws-kms-envelope" && r.maxuses === "1" },
];
const PATTERNS = [
  { key: "sequential", tint: 1.0 },
  { key: "scattered", tint: 0.55 },
];
const q = (s, pat) => rows.find((r) => r.phase === "query" && r.pattern === pat && s.match(r));

const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;");
function lighten(hex, f) {
  const n = parseInt(hex.slice(1), 16);
  const r = (n >> 16) & 255, g = (n >> 8) & 255, b = n & 255;
  const mix = (c) => Math.round(c + (255 - c) * (1 - f));
  return `rgb(${mix(r)},${mix(g)},${mix(b)})`;
}

// --- geometry ---
const W = 960, H = 470, PADL = 60, PADR = 24, PADT = 82, PADB = 92, GAP = 64;
const PANEL_W = (W - PADL - PADR - GAP) / 2;
const PLOT_H = H - PADT - PADB;

let svg = "";
const add = (s) => (svg += s + "\n");
add(`<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" font-family="system-ui,-apple-system,sans-serif" font-size="12">`);
add(`<rect width="${W}" height="${H}" fill="white"/>`);
add(`<text x="${W / 2}" y="26" text-anchor="middle" font-size="16" font-weight="600">Data-key reuse: sequential vs scattered reads (50-record query, 3 fields each)</text>`);
add(`<text x="${W / 2}" y="46" text-anchor="middle" font-size="12" fill="#6b7280">Reuse amortises sequential reads but collapses to one KMS call per record when the read pattern scatters. ZeroKMS is flat.</text>`);

const panels = [
  { x: PADL, title: "KMS calls per query", unit: (v) => v, ymax: 160, ticks: [0, 50, 100, 150], fmt: (v) => v, valOf: (r) => (r ? r.kms : null) },
  { x: PADL + PANEL_W + GAP, title: "p95 latency per query", unit: (v) => v, ymax: 200, ticks: [0, 50, 100, 150, 200], fmt: (v) => v + "ms", valOf: (r) => (r ? r.p95 : null) },
];

for (const panel of panels) {
  add(`<text x="${panel.x + PANEL_W / 2}" y="${PADT - 14}" text-anchor="middle" font-weight="600">${panel.title}</text>`);
  const yPix = (v) => PADT + PLOT_H * (1 - Math.min(v, panel.ymax) / panel.ymax);
  for (const t of panel.ticks) {
    const y = yPix(t);
    add(`<line x1="${panel.x + 36}" y1="${y}" x2="${panel.x + PANEL_W}" y2="${y}" stroke="#e5e7eb"/>`);
    add(`<text x="${panel.x + 30}" y="${y + 4}" text-anchor="end" fill="#6b7280">${panel.fmt(t)}</text>`);
  }
  const groupW = (PANEL_W - 44) / SERIES.length;
  const barW = groupW * 0.32;
  SERIES.forEach((s, gi) => {
    const gx = panel.x + 44 + groupW * gi + groupW / 2;
    PATTERNS.forEach((pat, pi) => {
      const r = q(s, pat.key);
      const raw = panel.valOf(r);
      const clipped = raw == null ? 0 : Math.min(raw, panel.ymax);
      const over = raw != null && raw > panel.ymax; // bar capped (failed/off-scale)
      const bx = gx + (pi - 0.5) * (barW + 6) - barW / 2;
      const by = yPix(clipped);
      const bh = PADT + PLOT_H - by;
      const fill = lighten(s.color, pat.tint);
      add(`<rect x="${bx.toFixed(1)}" y="${by.toFixed(1)}" width="${barW.toFixed(1)}" height="${bh.toFixed(1)}" fill="${fill}"${over ? ' stroke="#dc2626" stroke-width="1.5" stroke-dasharray="3 2"' : ""}/>`);
      const lbl = raw == null ? "" : over ? (r.failed >= 1000 ? "fail" : raw + "+") : panel.fmt(raw);
      add(`<text x="${(bx + barW / 2).toFixed(1)}" y="${(by - 5).toFixed(1)}" text-anchor="middle" font-size="11" fill="#374151" font-weight="600">${lbl}</text>`);
    });
    // backend label (two lines)
    s.label.split("\n").forEach((line, li) => {
      add(`<text x="${gx.toFixed(1)}" y="${H - PADB + 20 + li * 14}" text-anchor="middle" fill="#374151">${esc(line)}</text>`);
    });
  });
}

// legend (sequential = solid, scattered = lighter)
const lx = PADL + 40, lyy = H - 10;
add(`<rect x="${lx}" y="${lyy - 10}" width="14" height="11" fill="#475569"/>`);
add(`<text x="${lx + 20}" y="${lyy}" fill="#374151">sequential read</text>`);
add(`<rect x="${lx + 130}" y="${lyy - 10}" width="14" height="11" fill="${lighten("#475569", 0.55)}"/>`);
add(`<text x="${lx + 150}" y="${lyy}" fill="#374151">scattered read (realistic)</text>`);
add(`<rect x="${lx + 320}" y="${lyy - 10}" width="14" height="11" fill="none" stroke="#dc2626" stroke-width="1.5" stroke-dasharray="3 2"/>`);
add(`<text x="${lx + 340}" y="${lyy}" fill="#374151">capped / throttled to failure</text>`);
add("</svg>");

writeFileSync(`${DIR}/reuse.svg`, svg);
console.log(`wrote ${DIR}/reuse.svg`);
