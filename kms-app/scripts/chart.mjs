#!/usr/bin/env node
// Render the latency sweep into a dependency-free SVG: two LINEAR panels (insert /
// query) of median p95 latency vs batch size, one line per backend. Failure
// points get a hollow red ring. Reads the aggregated data.csv so it reproduces
// from committed results (laptop or in-region). Usage:
//   node scripts/chart.mjs                    # results/sweep (laptop)
//   SWEEP_DIR=results-ec2/sweep node scripts/chart.mjs   # in-region
import { readFileSync, writeFileSync } from "node:fs";

const dir = process.env.SWEEP_DIR || "results/sweep";
const SIZES = [20, 100, 500]; // 1000 omitted: app-instance CPU/IO saturation, not a key-service signal
const BACKENDS = [
  { key: "zerokms", label: "ZeroKMS (bulk)", color: "#16a34a" },
  { key: "aws-kms", label: "AWS KMS (direct)", color: "#ea580c" },
  { key: "aws-kms-envelope", label: "AWS KMS (envelope)", color: "#7c3aed" },
];

// Parse data.csv: kind,size,backend,round,requests,ok,failed,p50_ms,p95_ms,p99_ms,req_per_s
const rows = readFileSync(`${dir}/data.csv`, "utf8")
  .trim().split("\n").slice(1)
  .map((l) => {
    const c = l.split(",");
    return { kind: c[0], size: +c[1], backend: c[2], failed: +c[6], p95: +c[8] };
  });

// median p95 + any-failure flag per (kind,size,backend) across rounds
function stat(kind, size, backend) {
  const m = rows.filter((r) => r.kind === kind && r.size === size && r.backend === backend);
  const p95s = m.map((r) => r.p95).filter((p) => Number.isFinite(p)).sort((a, b) => a - b);
  const failed = m.reduce((s, r) => s + (r.failed || 0), 0);
  if (!p95s.length && !failed) return null;
  return { p95: p95s.length ? p95s[Math.floor((p95s.length - 1) / 2)] : null, failed };
}

// --- geometry ---
const W = 960, H = 480, PADL = 64, PADR = 20, PADT = 56, PADB = 96, GAP = 56;
const PANEL_W = (W - PADL - PADR - GAP) / 2;
const PLOT_H = H - PADT - PADB;
const YMAX = 8000; // linear, milliseconds
const TICKS = [0, 2000, 4000, 6000, 8000];
const yPix = (v) => PADT + PLOT_H * (1 - Math.min(Math.max(v, 0), YMAX) / YMAX);
const xPix = (panelX, i) => panelX + 40 + (PANEL_W - 60) * (i / (SIZES.length - 1));
const tickLabel = (v) => (v === 0 ? "0" : v >= 1000 ? v / 1000 + "s" : v + "ms");

const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;");
let svg = "";
const add = (s) => (svg += s + "\n");

add(`<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" font-family="system-ui,-apple-system,sans-serif" font-size="12">`);
add(`<rect width="${W}" height="${H}" fill="white"/>`);
add(`<text x="${W / 2}" y="24" text-anchor="middle" font-size="16" font-weight="600">ZeroKMS vs AWS KMS — median p95 latency by batch size (3 fields/record)</text>`);

const panels = [
  { kind: "insert", title: "Insert (encrypt + write)", x: PADL },
  { kind: "query", title: "Query (read + decrypt)", x: PADL + PANEL_W + GAP },
];

for (const panel of panels) {
  add(`<text x="${panel.x + PANEL_W / 2}" y="${PADT - 16}" text-anchor="middle" font-weight="600">${panel.title}</text>`);
  // y gridlines + labels (linear)
  for (const v of TICKS) {
    const y = yPix(v);
    add(`<line x1="${panel.x + 40}" y1="${y}" x2="${panel.x + PANEL_W}" y2="${y}" stroke="#e5e7eb"/>`);
    add(`<text x="${panel.x + 34}" y="${y + 4}" text-anchor="end" fill="#6b7280">${tickLabel(v)}</text>`);
  }
  // x labels
  SIZES.forEach((s, i) => {
    const x = xPix(panel.x, i);
    add(`<text x="${x}" y="${H - PADB + 20}" text-anchor="middle" fill="#374151">${s}</text>`);
  });
  add(`<text x="${panel.x + PANEL_W / 2}" y="${H - PADB + 40}" text-anchor="middle" fill="#6b7280">records per request</text>`);
  // lines
  for (const b of BACKENDS) {
    const pts = SIZES.map((s, i) => ({ ...stat(panel.kind, s, b.key), x: xPix(panel.x, i), i, s }))
      .filter((p) => p.p95 != null);
    if (pts.length > 1) {
      add(`<polyline fill="none" stroke="${b.color}" stroke-width="2.5" points="${pts.map((p) => `${p.x.toFixed(1)},${yPix(p.p95).toFixed(1)}`).join(" ")}"/>`);
    }
    for (const p of pts) {
      const y = yPix(p.p95);
      add(`<circle cx="${p.x.toFixed(1)}" cy="${y.toFixed(1)}" r="4" fill="${b.color}"/>`);
      if (p.failed) add(`<circle cx="${p.x.toFixed(1)}" cy="${y.toFixed(1)}" r="8" fill="none" stroke="#dc2626" stroke-width="2"/>`);
    }
  }
}

// legend
const lx = PADL + 40, lyy = H - 14;
let off = 0;
for (const b of BACKENDS) {
  add(`<rect x="${lx + off}" y="${lyy - 9}" width="14" height="3" fill="${b.color}"/>`);
  add(`<text x="${lx + off + 20}" y="${lyy - 4}" fill="#374151">${esc(b.label)}</text>`);
  off += 30 + b.label.length * 7;
}
add(`<circle cx="${lx + off + 6}" cy="${lyy - 8}" r="6" fill="none" stroke="#dc2626" stroke-width="2"/>`);
add(`<text x="${lx + off + 18}" y="${lyy - 4}" fill="#374151">= had failures (throttling)</text>`);
add("</svg>");

writeFileSync(`${dir}/latency.svg`, svg);
console.log(`wrote ${dir}/latency.svg`);
