#!/usr/bin/env node
// Linear throughput chart: achieved values/sec (y) vs offered request rate (x),
// one line per backend, two panels (insert / query). Linear y on purpose, so
// the magnitude of the gap is visible. Achieved peaks then degrades as each
// backend saturates; failures get a red ring. Reads ${THRU_DIR:-results/throughput}/data.csv.
import { readFileSync, writeFileSync } from "node:fs";

const dir = process.env.THRU_DIR || "results/throughput";
const rows = readFileSync(`${dir}/data.csv`, "utf8").trim().split("\n").slice(1)
  .map((l) => l.split(","))
  .map(([kind, batch, rate, backend, ok, failed, achieved, offered]) => ({
    kind, backend, rate: +rate, achieved: +achieved, failed: +failed,
  }));

const BACKENDS = [
  { key: "zerokms", label: "ZeroKMS (bulk)", color: "#16a34a" },
  { key: "aws-kms", label: "AWS KMS (direct)", color: "#ea580c" },
  { key: "aws-kms-envelope", label: "AWS KMS (envelope)", color: "#7c3aed" },
];
const RATES = [...new Set(rows.map((r) => r.rate))].sort((a, b) => a - b);
const series = (kind, key) =>
  rows.filter((r) => r.kind === kind && r.backend === key).sort((a, b) => a.rate - b.rate);

const maxA = Math.max(1000, ...rows.map((r) => r.achieved));
const YMAX = Math.ceil(maxA / 5000) * 5000;

const W = 980, H = 500, PADL = 76, PADR = 20, PADT = 56, PADB = 96, GAP = 64;
const PANEL_W = (W - PADL - PADR - GAP) / 2;
const PLOT_H = H - PADT - PADB;
const yPix = (v) => PADT + PLOT_H * (1 - v / YMAX);
const xPix = (px, i) => px + 44 + (PANEL_W - 60) * (i / Math.max(1, RATES.length - 1));
const kfmt = (v) => (v >= 1000 ? v / 1000 + "k" : String(v));

let svg = "";
const add = (s) => (svg += s + "\n");
add(`<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" font-family="system-ui,-apple-system,sans-serif" font-size="12">`);
add(`<rect width="${W}" height="${H}" fill="white"/>`);
add(`<text x="${W / 2}" y="24" text-anchor="middle" font-size="16" font-weight="600">Throughput — values/sec achieved as offered load rises (batch = 100 records)</text>`);

const panels = [
  { kind: "insert", title: "Insert (encrypt)", x: PADL },
  { kind: "query", title: "Query (decrypt)", x: PADL + PANEL_W + GAP },
];
const yticks = []; for (let v = 0; v <= YMAX; v += YMAX / 4) yticks.push(Math.round(v));

for (const p of panels) {
  add(`<text x="${p.x + PANEL_W / 2}" y="${PADT - 16}" text-anchor="middle" font-weight="600">${p.title}</text>`);
  for (const v of yticks) {
    const y = yPix(v);
    add(`<line x1="${p.x + 44}" y1="${y}" x2="${p.x + PANEL_W}" y2="${y}" stroke="#eef0f2"/>`);
    add(`<text x="${p.x + 38}" y="${y + 4}" text-anchor="end" fill="#6b7280">${kfmt(v)}</text>`);
  }
  RATES.forEach((r, i) => add(`<text x="${xPix(p.x, i)}" y="${H - PADB + 20}" text-anchor="middle" fill="#374151">${r}</text>`));
  add(`<text x="${p.x - 56}" y="${PADT + PLOT_H / 2}" text-anchor="middle" fill="#374151" transform="rotate(-90 ${p.x - 56} ${PADT + PLOT_H / 2})">achieved (values/s)</text>`);
  add(`<text x="${p.x + PANEL_W / 2}" y="${H - PADB + 44}" text-anchor="middle" fill="#374151">offered load (requests/s)</text>`);
  for (const b of BACKENDS) {
    const pts = series(p.kind, b.key).map((d) => ({ ...d, i: RATES.indexOf(d.rate) }));
    if (pts.length > 1) add(`<polyline fill="none" stroke="${b.color}" stroke-width="2.5" points="${pts.map((d) => `${xPix(p.x, d.i).toFixed(1)},${yPix(d.achieved).toFixed(1)}`).join(" ")}"/>`);
    for (const d of pts) {
      add(`<circle cx="${xPix(p.x, d.i).toFixed(1)}" cy="${yPix(d.achieved).toFixed(1)}" r="4" fill="${b.color}"/>`);
      if (d.failed) add(`<circle cx="${xPix(p.x, d.i).toFixed(1)}" cy="${yPix(d.achieved).toFixed(1)}" r="8" fill="none" stroke="#dc2626" stroke-width="2"/>`);
    }
  }
}

let off = 0; const ly = H - 16;
for (const b of BACKENDS) {
  add(`<rect x="${PADL + off}" y="${ly - 9}" width="14" height="3" fill="${b.color}"/>`);
  add(`<text x="${PADL + off + 20}" y="${ly - 4}" fill="#374151">${b.label}</text>`);
  off += 34 + b.label.length * 7;
}
add(`<circle cx="${PADL + off + 6}" cy="${ly - 8}" r="6" fill="none" stroke="#dc2626" stroke-width="2"/>`);
add(`<text x="${PADL + off + 18}" y="${ly - 4}" fill="#374151">= had failures</text>`);
add("</svg>");

writeFileSync(`${dir}/throughput.svg`, svg);
console.log(`wrote ${dir}/throughput.svg`);
