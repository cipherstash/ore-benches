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

```
Artillery ──HTTP──▶ Next.js API ──encrypt/decrypt──▶ [ ZeroKMS | AWS KMS ]
                         │
                         └──store ciphertext──▶ Postgres (users table)
```

- `POST /api/users` — encrypts `email` + `name`, inserts the ciphertext (write path)
- `GET /api/users/:id` — reads the row, decrypts both fields (read path)
- `GET /api/health` — checks DB + that the selected backend initializes

The backend is chosen per server process by `ENCRYPTION_BACKEND`. All three
store a serialized ciphertext string per field, so the table shape is
identical; only the key-management work differs.

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
cp .env.example .env.local   # fill in credentials
npm run db:setup             # create the users table
```

## Run a comparison

Each backend is a separate server process. Build once, then for each backend:
start the server with that backend, run the load profile, save the output.

```sh
npm run build

# --- ZeroKMS ---
ENCRYPTION_BACKEND=zerokms npm start &        # or: npm run serve:zerokms
curl -s localhost:3000/api/health             # expect {"ok":true,"backend":"zerokms"}
npm run load:zerokms                           # writes results/zerokms.json
kill %1

# --- AWS KMS (naive direct) ---
ENCRYPTION_BACKEND=aws-kms npm start &
curl -s localhost:3000/api/health             # expect {"ok":true,"backend":"aws-kms"}
npm run load:aws-kms                           # writes results/aws-kms.json
kill %1

# --- AWS KMS (envelope, production pattern) ---
ENCRYPTION_BACKEND=aws-kms-envelope npm start &
curl -s localhost:3000/api/health             # expect {"ok":true,"backend":"aws-kms-envelope"}
npm run load:aws-kms-envelope                  # writes results/aws-kms-envelope.json
kill %1

# --- compare (skips any backend you didn't run) ---
npm run report     # side-by-side latency percentiles + throughput
```

`npm run report` prints an overall table plus a **per-endpoint** breakdown
(write = `create`, read = `read`), so you can see encrypt vs decrypt cost
separately rather than blended.

### The load profile (`load/users.yml`)

The default is a **sustained, fixed-rate** test: a short warmup, then one
arrival rate held for 2 minutes. Holding a steady rate gives stable
p50/p95/p99 at a known offered load, which is what you want when comparing
backends. To find the saturation knee instead, comment out the `steady` phase
and uncomment the `ramp` phase.

Two things to keep honest:

- **AWS KMS rate limits.** For the `aws-kms` (direct) backend, keep
  `arrivalRate` well under your account's per-region KMS quota, or you'll be
  measuring KMS throttling, not crypto cost. `aws-kms-envelope` with data-key
  caching makes far fewer KMS calls and tolerates higher rates.
- **`ensure` thresholds.** The profile fails the run if any virtual user errors
  or p95 exceeds 1s, so a throttled/erroring backend can't quietly look "fast".
  Adjust to your SLO.

## Layout

```
kms-app/
  app/api/users/          POST (create+encrypt), GET (read+decrypt)
  app/api/health/         readiness probe
  lib/encryption/         backend abstraction + zerokms / aws-kms / aws-kms-envelope impls
  lib/db.ts               Postgres pool
  load/users.yml          Artillery profile (+ processor.cjs payload generator)
  scripts/summarize.mjs   Artillery JSON → comparison table
  sql/schema.sql          users table
  results/                load-test outputs (gitignored)
```

## TODO / next steps

- [ ] Add a `report:build` that writes a Markdown report into `results/` for
      committing alongside the EQL benchmark reports
- [ ] First real run: `npm install`, confirm the `@cipherstash/stack` API
      surface, and capture baseline numbers per backend
