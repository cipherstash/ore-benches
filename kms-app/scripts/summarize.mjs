#!/usr/bin/env node
// Summarize one or more Artillery JSON outputs into a compact comparison
// table. Usage:
//   node scripts/summarize.mjs results/zerokms.json results/aws-kms.json
//
// Reads the `aggregate` block Artillery writes with `--output`. Prints latency
// percentiles (ms) and request throughput per file, side by side.

import { readFileSync, existsSync } from "node:fs";
import { basename } from "node:path";

const requested = process.argv.slice(2);
if (requested.length === 0) {
  console.error("usage: node scripts/summarize.mjs <artillery.json> [more.json ...]");
  process.exit(1);
}

// Skip files that weren't produced (e.g. only two of three backends were run).
const files = requested.filter((f) => {
  if (existsSync(f)) return true;
  console.error(`skipping ${f} (not found)`);
  return false;
});
if (files.length === 0) {
  console.error("no result files found — run a load test first");
  process.exit(1);
}

const cols = files.map((file) => {
  const data = JSON.parse(readFileSync(file, "utf-8"));
  const agg = data.aggregate ?? data;
  const summaries = agg.summaries ?? {};
  const rt = summaries["http.response_time"] ?? {};
  const counters = agg.counters ?? {};
  const rates = agg.rates ?? {};

  // Per-endpoint latency from the metrics-by-endpoint plugin (named requests:
  // create / read). Key format varies by Artillery version, so match any
  // response_time summary that isn't the overall http.response_time and label
  // it by its trailing segment.
  const endpoints = {};
  for (const [key, val] of Object.entries(summaries)) {
    if (key === "http.response_time") continue;
    if (!/response_time/i.test(key)) continue;
    const label = key.split(/[./]/).pop();
    endpoints[label] = val;
  }

  return {
    label: basename(file).replace(/\.json$/, ""),
    requests: counters["http.requests"] ?? 0,
    errors:
      (counters["vusers.failed"] ?? 0) +
      Object.entries(counters)
        .filter(([k]) => k.startsWith("errors."))
        .reduce((s, [, v]) => s + v, 0),
    rps: rates["http.request_rate"] ?? 0,
    min: rt.min,
    median: rt.median ?? rt.p50,
    p95: rt.p95,
    p99: rt.p99,
    max: rt.max,
    endpoints,
  };
});

const rows = [
  ["metric", ...cols.map((c) => c.label)],
  ["requests", ...cols.map((c) => String(c.requests))],
  ["errors", ...cols.map((c) => String(c.errors))],
  ["req/s (mean)", ...cols.map((c) => fmt(c.rps))],
  ["latency min (ms)", ...cols.map((c) => fmt(c.min))],
  ["latency p50 (ms)", ...cols.map((c) => fmt(c.median))],
  ["latency p95 (ms)", ...cols.map((c) => fmt(c.p95))],
  ["latency p99 (ms)", ...cols.map((c) => fmt(c.p99))],
  ["latency max (ms)", ...cols.map((c) => fmt(c.max))],
];

printTable(rows);

// Per-endpoint breakdown (write vs read), if metrics-by-endpoint data exists.
const endpointLabels = [
  ...new Set(cols.flatMap((c) => Object.keys(c.endpoints))),
].sort();
if (endpointLabels.length > 0) {
  const eRows = [["endpoint / pctl", ...cols.map((c) => c.label)]];
  const pctls = [
    ["p50", (s) => s.median ?? s.p50],
    ["p95", (s) => s.p95],
    ["p99", (s) => s.p99],
  ];
  for (const ep of endpointLabels) {
    for (const [pctl, get] of pctls) {
      eRows.push([
        `${ep} ${pctl} (ms)`,
        ...cols.map((c) => fmt(c.endpoints[ep] ? get(c.endpoints[ep]) : undefined)),
      ]);
    }
  }
  console.log("\nPer-endpoint latency (write = create, read = read):");
  printTable(eRows);
}

function printTable(table) {
  const w = table[0].map((_, i) => Math.max(...table.map((r) => String(r[i]).length)));
  const fmtRow = (r) => r.map((cell, i) => String(cell).padEnd(w[i])).join("  ");
  console.log(fmtRow(table[0]));
  console.log(w.map((n) => "-".repeat(n)).join("  "));
  table.slice(1).forEach((r) => console.log(fmtRow(r)));
}

function fmt(n) {
  return typeof n === "number" ? n.toFixed(1) : "—";
}
