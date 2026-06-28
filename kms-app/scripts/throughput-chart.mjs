#!/usr/bin/env node
// Linear throughput chart from results/throughput/data.csv: achieved values/sec
// (y) vs offered values/sec (x), one line per backend, two panels (insert /
// query). A dashed diagonal marks perfect linear scaling — backends that keep
// up track it; backends that saturate fall below it. Linear axes on purpose, so
// the magnitude of the gap is visible (unlike the log latency chart).
import { readFileSync, writeFileSync } from "node:fs";

const dir = "results/throughput";
const rows = readFileSync(`${dir}/data.csv`, "utf8").trim().split("\n").slice(1)
  .map((l) => l.split(","))
  .map(([kind, batch, rate, backend, ok, failed, achieved, offered]) => ({
    kind, backend, rate: +rate,
    achieved: +achieved, offered: +offered, failed: +failed,
  }));

const BACKENDS = [
  { key: "zerokms", label: "ZeroKMS (bulk)", color: "#16a34a" },
  { key: "aws-kms", label: "AWS KMS (direct)", color: "#ea580c" },
  { key: "aws-kms-envelope", label: "AWS KMS (envelope)", color: "#7c3aed" },
];
const series = (kind, key) =>
  rows.filter((r) => r.kind === kind && r.backend === key).sort((a, b) => a.offered - b.offered);

const maxV = Math.max(1, ...rows.map((r) => Math.max(r.offered, r.achieved)));
const AXMAX = Math.ceil(maxV / 5000) * 5000; // round up to a tidy 5k

const W = 980, H = 500, PADL = 72, PADR = 20, PADT = 56, PADB = 100, GAP = 64;
const PANEL_W = (W - PADL - PADR - GAP) / 2;
const PLOT_H = H - PADT - PADB;
const yPix = (v) => PADT + PLOT_H * (1 - v / AXMAX);
const xPix = (px, v) => px + (PANEL_W) * (v / AXMAX);
const kfmt = (v) => (v >= 1000 ? v / 1000 + "k" : String(v));

let svg = "";
const add = (s) => (svg += s + "\n");
add(`<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" font-family="system-ui,-apple-system,sans-serif" font-size="12">`);
add(`<rect width="${W}" height="${H}" fill="white"/>`);
add(`<text x="${W / 2}" y="24" text-anchor="middle" font-size="16" font-weight="600">Throughput — values encrypted/decrypted per second (batch = 100 records)</text>`);

const panels = [
  { kind: "insert", title: "Insert (encrypt)", x: PADL },
  { kind: "query", title: "Query (decrypt)", x: PADL + PANEL_W + GAP },
];
const ticks = [];
for (let v = 0; v <= AXMAX; v += AXMAX / 3) ticks.push(Math.round(v));

for (const p of panels) {
  add(`<text x="${p.x + PANEL_W / 2}" y="${PADT - 16}" text-anchor="middle" font-weight="600">${p.title}</text>`);
  // gridlines + axis ticks
  for (const v of ticks) {
    const y = yPix(v);
    add(`<line x1="${p.x}" y1="${y}" x2="${p.x + PANEL_W}" y2="${y}" stroke="#eef0f2"/>`);
    add(`<text x="${p.x - 8}" y="${y + 4}" text-anchor="end" fill="#6b7280">${kfmt(v)}</text>`);
    add(`<text x="${xPix(p.x, v)}" y="${H - PADB + 20}" text-anchor="middle" fill="#6b7280">${kfmt(v)}</text>`);
  }
  add(`<text x="${p.x - 52}" y="${PADT + PLOT_H / 2}" text-anchor="middle" fill="#374151" transform="rotate(-90 ${p.x - 52} ${PADT + PLOT_H / 2})">achieved (values/s)</text>`);
  add(`<text x="${p.x + PANEL_W / 2}" y="${H - PADB + 44}" text-anchor="middle" fill="#374151">offered (values/s)</text>`);
  // perfect-scaling diagonal
  add(`<line x1="${p.x}" y1="${yPix(0)}" x2="${xPix(p.x, AXMAX)}" y2="${yPix(AXMAX)}" stroke="#9ca3af" stroke-dasharray="4 4"/>`);
  add(`<text x="${xPix(p.x, AXMAX) - 6}" y="${yPix(AXMAX) + 14}" text-anchor="end" fill="#9ca3af" font-size="11">perfect scaling</text>`);
  // backend lines
  for (const b of BACKENDS) {
    const pts = series(p.kind, b.key);
    if (pts.length > 1) {
      add(`<polyline fill="none" stroke="${b.color}" stroke-width="2.5" points="${pts.map((d) => `${xPix(p.x, d.offered).toFixed(1)},${yPix(d.achieved).toFixed(1)}`).join(" ")}"/>`);
    }
    for (const d of pts) {
      add(`<circle cx="${xPix(p.x, d.offered).toFixed(1)}" cy="${yPix(d.achieved).toFixed(1)}" r="4" fill="${b.color}"/>`);
      if (d.failed) add(`<circle cx="${xPix(p.x, d.offered).toFixed(1)}" cy="${yPix(d.achieved).toFixed(1)}" r="8" fill="none" stroke="#dc2626" stroke-width="2"/>`);
    }
  }
}

// legend
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
