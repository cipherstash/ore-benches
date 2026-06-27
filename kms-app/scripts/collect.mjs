#!/usr/bin/env node
// Collect the round-tagged sweep JSONs into one tidy CSV — the canonical data
// artifact the report and chart are built from. Usage: node scripts/collect.mjs
import { readFileSync, readdirSync, writeFileSync } from "node:fs";

const dir = "results/sweep";
const re = /^(insert|query)-s(\d+)-(.+)-r(\d+)\.json$/;
const header = [
  "kind", "size", "backend", "round",
  "requests", "ok", "failed", "p50_ms", "p95_ms", "p99_ms", "req_per_s",
];

const rows = readdirSync(dir)
  .map((f) => f.match(re))
  .filter(Boolean)
  .map((m) => {
    const [file, kind, size, backend, round] = m;
    const a = JSON.parse(readFileSync(`${dir}/${file}`, "utf8")).aggregate;
    const c = a.counters ?? {};
    const rt = a.summaries?.["http.response_time"] ?? {};
    const ok = (c["http.codes.200"] ?? 0) + (c["http.codes.201"] ?? 0);
    return [
      kind, Number(size), backend, Number(round),
      c["http.requests"] ?? 0, ok, c["vusers.failed"] ?? 0,
      r(rt.median ?? rt.p50), r(rt.p95), r(rt.p99),
      r(a.rates?.["http.request_rate"]),
    ];
  })
  .sort((a, b) =>
    a[0].localeCompare(b[0]) || a[1] - b[1] ||
    a[2].localeCompare(b[2]) || a[3] - b[3],
  );

writeFileSync(
  `${dir}/data.csv`,
  [header, ...rows].map((r) => r.join(",")).join("\n") + "\n",
);
console.log(`wrote ${dir}/data.csv (${rows.length} rows)`);

function r(n) {
  return typeof n === "number" ? Math.round(n) : "";
}
