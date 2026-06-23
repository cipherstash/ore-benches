# KMS comparison harness

A thin Next.js CRUD app for load-testing **field-level encryption** with
swappable key-management backends, driven by [Artillery](https://www.artillery.io/).
It exists to compare **ZeroKMS** (via the CipherStash Encryption SDK) against
**AWS KMS** under a realistic HTTP workload — the same app, same database,
same load profile, only the encryption backend changes.

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

The backend is chosen per server process by `ENCRYPTION_BACKEND`
(`zerokms` | `aws-kms`). Both store a serialized ciphertext string per field,
so the table shape is identical; only the key-management work differs.

## Fairness caveats (read before quoting numbers)

This harness deliberately measures the **naive direct-KMS** pattern, which is
what most teams reach for first — but it is not the only AWS pattern:

- **Direct KMS Encrypt/Decrypt per value** has a 4 KB plaintext limit and is
  **rate-limited per region**, so a saturation test mostly measures KMS API
  throttling. The production-grade AWS approach is **envelope encryption**
  (KMS protects a local data key; AES-GCM encrypts the data locally). A
  `aws-kms-envelope` backend variant should be added to compare that path —
  see the TODO in `lib/encryption/aws-kms.ts`.
- **ZeroKMS** issues a unique key per record and does the key derivation in the
  SDK; its cost profile is different by design. Report both write and read
  paths, not a single number.
- Run the app and Postgres on the **same hardware/region** for each backend,
  and warm up before measuring. Numbers are comparative for *this* workload,
  not absolute KMS benchmarks.

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

# --- AWS KMS ---
ENCRYPTION_BACKEND=aws-kms npm start &
curl -s localhost:3000/api/health             # expect {"ok":true,"backend":"aws-kms"}
npm run load:aws-kms                           # writes results/aws-kms.json
kill %1

# --- compare ---
npm run report     # side-by-side latency percentiles + throughput
```

Tune the load in `load/users.yml` (`phases`). The defaults are a gentle local
ramp; raise `arrivalRate` to push toward saturation.

## Layout

```
kms-app/
  app/api/users/          POST (create+encrypt), GET (read+decrypt)
  app/api/health/         readiness probe
  lib/encryption/         backend abstraction + zerokms / aws-kms impls
  lib/db.ts               Postgres pool
  load/users.yml          Artillery profile (+ processor.js payload generator)
  scripts/summarize.mjs   Artillery JSON → comparison table
  sql/schema.sql          users table
  results/                load-test outputs (gitignored)
```

## TODO / next steps

- [ ] Add an `aws-kms-envelope` backend for the production AWS pattern
- [ ] Capture the original Artillery Cloud scenario (share `sh_75edb…`) and
      reconcile this profile against it
- [ ] Add a `report:build` that writes a Markdown report into `results/` for
      committing alongside the EQL benchmark reports
