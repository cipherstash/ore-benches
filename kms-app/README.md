# KMS comparison harness

A thin Next.js CRUD app for load-testing **field-level encryption** with
swappable key-management backends, driven by [Artillery](https://www.artillery.io/).
It exists to compare **ZeroKMS** (via the CipherStash Encryption SDK) against
AWS KMS under a realistic HTTP workload — the same app, same database, same
load profile, only the encryption backend changes. Three backends ship:

| `ENCRYPTION_BACKEND` | What it does |
|---|---|
| `zerokms` | CipherStash Encryption SDK; unique key per record |
| `aws-kms` | Naive direct KMS Encrypt/Decrypt per value |
| `aws-kms-envelope` | Production AWS pattern: KMS-wrapped AES-256 data key + local AES-GCM, with data-key caching |

> Replaces the older, lost Next.js + Artillery comparison. Lives in `benches`
> so the methodology and results sit alongside the EQL/PostgreSQL benchmarks.

## How it works

Each request encrypts or decrypts **a batch of values** — that bulk
amortization is the whole point. A realistic request handles, say, 20 records ×
3 encrypted fields = 60 values:

```
Artillery ──HTTP──▶ Next.js API ──batch encrypt/decrypt──▶ [ ZeroKMS | AWS KMS ]
                         │
                         └──store ciphertext──▶ Postgres (records table)
```

- `POST /api/records/insert` `{ count }` — generate `count` records, **bulk
  encrypt** all `count×3` fields, multi-row insert (the **write** benchmark)
- `GET /api/records/query?limit=N` — read a random window of `N` existing rows,
  **bulk decrypt** all `N×3` fields (the **read** benchmark)
- `GET /api/health` — checks DB + that the selected backend initializes

The backend is chosen per server process by `ENCRYPTION_BACKEND`. The decisive
difference: **ZeroKMS does a whole batch in one network round-trip**
(`bulkEncryptModels`/`bulkDecryptModels`, up to 10,000 keys per call), while
**AWS KMS has no bulk API** — under per-value mediation it makes one call per
value (`count×3` per request). All three store a serialized ciphertext string
per field, so the table shape is identical.

## Fairness: compare under equal security constraints

A latency comparison is only fair if both systems provide the **same security
guarantee**. ZeroKMS gives every value a unique key and mediates every
encrypt/decrypt individually, so each value's access is independently auditable
and revocable. The AWS side must hold that same constraint to be comparable —
which means **per-value KMS operations, no data-key caching**:

- **`aws-kms` (direct)** and **`aws-kms-envelope` with `ENVELOPE_DATA_KEY_MAX_USES=1`
  (the default)** both make one KMS call per value, preserving per-value
  mediation/audit. These are the apples-to-apples comparisons against ZeroKMS.
- **`aws-kms-envelope` with caching (`MAX_USES > 1`) is a *different, weaker*
  security model**, not a faster version of the same one. A cached data key
  covers many records with its plaintext held in app memory, so you lose the
  ability to identify, audit, or revoke access to individual values. It's
  faster because it does less — included only to show the trade-off, **not** a
  fair comparison against ZeroKMS.
- Report write and read paths separately; run the app + Postgres on the same
  hardware/region for each backend; warm up first. Numbers are comparative for
  *this* workload, not absolute KMS benchmarks.

## Prerequisites

- Node.js 20+
- Postgres running. The benches native cluster works out of the box:
  from the repo root, `mise run postgres` (listens on `:5400`).
- Credentials for whichever backend(s) you test (see `.env.example`).

## Setup

```sh
cd kms-app
npm install
cp .env.example .env.local   # AWS key id + region; ZeroKMS uses your `stash` profile
npm run db:setup             # create the records table
```

## Run a comparison

There are two benchmarks — **insert** (write) and **query** (read) — and three
backends. The query benchmark reads existing rows, so **seed each backend by
running its insert benchmark first**. Build once, then per backend:

```sh
npm run build

# example for one backend (repeat for aws-kms, aws-kms-envelope):
ENCRYPTION_BACKEND=zerokms npm start &
curl -s localhost:3000/api/health                       # {"ok":true,"backend":"zerokms"}

npm run load:insert -- -o results/insert-zerokms.json   # write benchmark (also seeds)
npm run load:query  -- -o results/query-zerokms.json    # read benchmark
kill %1
```

Then compare (each skips any backend you didn't run):

```sh
npm run report:insert   # write-path latency/throughput across backends
npm run report:query    # read-path latency/throughput across backends
```

Tune batch size in `load/insert.yml` (`count`) and `load/query.yml` (`limit`).
Larger batches push ZeroKMS toward its 10k-keys-per-call ceiling while AWS's
per-value call count grows linearly.

### The load profiles

Both default to a **sustained, fixed-rate** test (short warmup, then one arrival
rate held for 2 minutes) — stable percentiles at a known offered load. Keep
`arrivalRate × count × 3` (the per-second KMS call rate) under your AWS region
quota for the `aws-kms` backend, or you'll measure throttling, not crypto. Each
profile's `ensure` block fails the run if any virtual user errors.

## Layout

```
kms-app/
  app/api/records/insert/  POST { count } — bulk encrypt + insert
  app/api/records/query/   GET ?limit=N  — bulk read + decrypt
  app/api/health/          readiness probe
  lib/encryption/          batch backend interface + zerokms / aws-kms / aws-kms-envelope
  lib/records.ts           synthetic record generator
  lib/db.ts                Postgres pool
  load/insert.yml          write benchmark   load/query.yml  read benchmark
  scripts/summarize.mjs    Artillery JSON → comparison table
  sql/schema.sql           records table (3 encrypted columns)
  results/                 load-test outputs (gitignored)
```

## TODO / next steps

- [ ] Add a `report:build` that writes a Markdown report into `results/` for
      committing alongside the EQL benchmark reports
- [ ] Capture baseline numbers per backend across a couple of batch sizes;
      run interleaved repeats before quoting figures
