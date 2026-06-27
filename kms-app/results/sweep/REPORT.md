# KMS comparison — batch-size sweep (first results)

Single-run characterization. **Directional, not publishable** — one sample per
cell, one machine, one region, one moment. See caveats.

- **Setup:** local Mac → AWS KMS / ZeroKMS in `ap-southeast-2`. Postgres on
  `:5400`. Each record has 3 encrypted fields, so a batch of size N = 3N values.
- **Backends, per-value-mediation (equal security):** `zerokms` uses
  `bulkEncryptModels`/`bulkDecryptModels` (one round-trip per batch); `aws-kms`
  and `aws-kms-envelope` (MAX_USES=1) make one KMS call per value (3N), fanned
  out concurrently (`Promise.all`, `AWS_MAX_ATTEMPTS=3`).
- **Load:** warmup 2s + steady 12s; arrival rate 10/5/2/1 for size 20/100/500/1000.

## Latency (p95 ms, p50/p99 in parens), `!N` = failed virtual users

### Insert (write)
| size | zerokms | aws-kms | aws-kms-envelope |
|---:|---:|---:|---:|
| 20 | 45 (33/69) | 120 (66/128) | 97 (65/111) |
| 100 | 72 (59/81) | 728 (478/837) | 743 (508/758) |
| 500 | 460 (416/478) | 6976 (4147/6976) **!14** | 7261 (4231/7261) **!15** |
| 1000 | 872 (854/872) | 6440 (4317/6440) **!8** | 6312 (4231/6312) **!8** |

### Query (read)
| size | zerokms | aws-kms | aws-kms-envelope |
|---:|---:|---:|---:|
| 20 | 321\* (53/392) | 113 (66/120) | 107 (65/118) |
| 100 | 123 (63/141) | 728 (460/821) | 714 (460/728) |
| 500 | 573 (392/672) | 7710 (4867/7710) **!13** | 7866 (4676/7866) **!10** |
| 1000 | 944 (872/963) | 7261 (5272/7261) **!7** | 6976 (4771/6976) **!7** |

\* zerokms query size-20 p95 is a cold-start outlier (first cell; p50 53ms is representative).

## What it shows
- **ZeroKMS: 0 failures at every size.** One round-trip per batch, so latency
  grows gently with batch size (45 → 872 ms p95 insert across 20 → 1000).
- **AWS KMS (both variants): clean at 20 and 100, but ~10× slower by 100 records
  — and breaks at 500+** (throttling: most requests in those cells fail after
  retries, p95 ~7 s). AWS has no bulk API, so a 500-record read is 1500
  concurrent KMS calls; the rate limit wins.
- **Representative gap (p50):** ~2× at size 20, ~7–8× at 100, ~10–12× at 500
  (where AWS also starts failing). The docs' "≈14×" sits in this range but is
  conditions-dependent — quote it with the batch size, not as a blanket figure.

## Caveats (must travel with these numbers)
- One run per cell. The `!N` failure counts and the p95 tails are noisy; rerun
  interleaved before quoting.
- AWS large-batch failures are throttling at default retry settings; a
  different region/quota/retry config would shift the cliff.
- ZeroKMS bulk latency also grows with batch (bigger payload + processing) — it
  is not free, just one round-trip and far flatter.
- An earlier sweep was discarded: a short-lived SSO session expired mid-run and
  produced fast 500s that masqueraded as throttling. These numbers are from a
  clean session.
