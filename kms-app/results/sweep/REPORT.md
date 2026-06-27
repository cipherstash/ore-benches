# KMS comparison — batch-size sweep

3 interleaved rounds (backends rotated each round to share temporal AWS
variance). Local Mac → AWS KMS / ZeroKMS in `ap-southeast-2`; Postgres `:5400`.
Each record = 3 encrypted fields, so batch size N = 3N values. AWS uses
least-privilege static creds (no SSO expiry). Per-value mediation for all
backends (ZeroKMS bulk = one round-trip; AWS = one KMS call per value,
`AWS_MAX_ATTEMPTS=3`). Load: warmup 2s + steady 12s; arrival 10/5/2/1 for
size 20/100/500/1000.

## Latency — median p95 ms across 3 rounds (zerokms vs aws-kms ≈ ratio)

### Insert (write)
| size | zerokms | aws-kms | aws-kms-envelope | gap |
|---:|---:|---:|---:|---:|
| 20 | 70 | 95 | 116 | 1.4× |
| 100 | 70 | 714 | 714 | ~10× |
| 500 | 450 | 7557 | 7408 | ~17× |
| 1000 | 907 | 6312 | 6312 | ~7× |

### Query (read)
| size | zerokms | aws-kms | aws-kms-envelope | gap |
|---:|---:|---:|---:|---:|
| 20 | 60 | 125 | 82 | ~2× |
| 100 | 82 | 728 | 518 | ~9× |
| 500 | 433 | 7408 | 5488 | ~17× |
| 1000 | 889 | 6838 | 5945 | ~8× |

## Reliability
- **ZeroKMS: effectively zero failures at every size** (a couple of stray
  cold-start blips), one round-trip per batch.
- **AWS KMS fails at large batches.** size-1000 insert failed 8/14, 10/14,
  14/14 across the three rounds (cumulative throttling). size-500 also fails
  heavily. A 500-record read = 1500 concurrent KMS calls; the rate limit wins.
- **Caveat on failure *counts*:** an AWS large-batch meltdown saturates the
  Node server and spills timeouts into the *next* cell (e.g. round-3 query-20
  timed out entirely because the round-3 size-1000 insert hadn't drained). So
  treat the AWS failure *counts* as inflated by inter-cell spillover (drain
  time too short); the **latency medians exclude fully-failed rounds and are
  clean**. The qualitative result — AWS throttle-fails past a few hundred
  values — is robust.

## What it means for the docs "≈14×" claim
The gap is a function of batch size: ~1–2× at 20, **~10× at 100, ~17× at 500**,
~7–8× at 1000 (where ZeroKMS's own latency grows and AWS is failing). The
"≈14×" figure is **defensible specifically as a bulk-operation, mid-batch
(~100–500 record) number** — quote it with that condition, not as a blanket
multiple. The deeper, more honest point: ZeroKMS does a whole batch in one
network round-trip (up to 10k keys), while AWS KMS has no bulk API at all, so
the gap widens with batch size until AWS simply throttle-fails.

## Known limitations (before publishing)
- One machine, one region, 3 rounds, 12s steady cells — still modest.
- Inter-cell drain too short (inflates AWS failure counts; see above). A clean
  failure-rate run needs longer settle or per-cell process isolation.
- ZeroKMS bulk latency also grows with batch (bigger payload + processing) —
  it is one round-trip, not free.
