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

## Fairness caveats (read before quoting numbers)

Pick the AWS comparison that matches the claim you're making:

- **`aws-kms` (naive direct KMS)** has a 4 KB plaintext limit and is
  **rate-limited per region**, so a saturation test mostly measures KMS API
  throttling. It's what teams reach for first, but it's not how AWS recommends
  encrypting bulk application data.
- **`aws-kms-envelope`** is the production-grade AWS pattern (KMS protects a
  local AES-256 data key; AES-GCM encrypts locally). With data-key caching
  (`ENVELOPE_DATA_KEY_MAX_USES`) it makes far fewer KMS calls, so it's the
  fairer high-throughput comparison. Set `ENVELOPE_DATA_KEY_MAX_USES=1` to see
  the no-caching worst case.
- **`zerokms`** issues a unique key per record and derives keys in the SDK; its
  cost profile is different by design. Report both write and read paths, not a
  single number.
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

Tune the load in `load/users.yml` (`phases`). The defaults are a gentle local
ramp; raise `arrivalRate` to push toward saturation.

## Layout

```
kms-app/
  app/api/users/          POST (create+encrypt), GET (read+decrypt)
  app/api/health/         readiness probe
  lib/encryption/         backend abstraction + zerokms / aws-kms / aws-kms-envelope impls
  lib/db.ts               Postgres pool
  load/users.yml          Artillery profile (+ processor.js payload generator)
  scripts/summarize.mjs   Artillery JSON → comparison table
  sql/schema.sql          users table
  results/                load-test outputs (gitignored)
```

## TODO / next steps

- [ ] Capture the original Artillery Cloud scenario (share `sh_75edb…`) and
      reconcile this profile against it
- [ ] Add a `report:build` that writes a Markdown report into `results/` for
      committing alongside the EQL benchmark reports
