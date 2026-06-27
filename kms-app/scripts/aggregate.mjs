#!/usr/bin/env node
// Aggregate round-tagged sweep results: for each (kind, size, backend), report
// median p95 (ms) across rounds, the min–max spread, and total failed vusers.
// Usage: node scripts/aggregate.mjs [rounds]   (default: auto-detect up to 9)
import { readFileSync, existsSync } from "node:fs";

const ROUNDS = Number(process.argv[2] ?? 9);
const SIZES = [20, 100, 500, 1000];
const BACKENDS = ["zerokms", "aws-kms", "aws-kms-envelope"];
const dir = "results/sweep";

function cell(kind, size, backend) {
  const p95s = [];
  let failed = 0, n = 0;
  for (let r = 1; r <= ROUNDS; r++) {
    const f = `${dir}/${kind}-s${size}-${backend}-r${r}.json`;
    if (!existsSync(f)) continue;
    const a = JSON.parse(readFileSync(f, "utf8")).aggregate;
    const p95 = a.summaries?.["http.response_time"]?.p95;
    if (typeof p95 === "number") p95s.push(p95);
    failed += a.counters?.["vusers.failed"] ?? 0;
    n++;
  }
  if (n === 0) return null;
  p95s.sort((x, y) => x - y);
  const med = p95s.length ? p95s[Math.floor((p95s.length - 1) / 2)] : null;
  return { med, lo: p95s[0], hi: p95s[p95s.length - 1], failed, n };
}

const fmt = (n) => (typeof n === "number" ? String(Math.round(n)) : "—");

for (const kind of ["insert", "query"]) {
  console.log(`\n### ${kind.toUpperCase()} — median p95 ms [min–max], !failed, (rounds) ###`);
  console.log(["size".padEnd(6), ...BACKENDS.map((b) => b.padEnd(26))].join(""));
  for (const s of SIZES) {
    const cells = BACKENDS.map((b) => {
      const c = cell(kind, s, b);
      if (!c) return "—".padEnd(26);
      return `${fmt(c.med)} [${fmt(c.lo)}–${fmt(c.hi)}]${c.failed ? ` !${c.failed}` : ""} (${c.n})`.padEnd(26);
    });
    console.log([String(s).padEnd(6), ...cells].join(""));
  }
}
