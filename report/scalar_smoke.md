# SCALAR_SMOKE Queries

[← Back to overview](./BENCHMARK_REPORT.md)

Per-tier query performance. Each scenario lists its SQL, the indexes available on the target table, the indexes the planner actually picked per tier, the timing table, and the full EXPLAIN plan in a collapsed block.

## bigint_ord/range_gt_10

**Description:** Unknown query

****

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | — | 777.22μs | N/A |

_Rows are the planner's estimate from `EXPLAIN` captured before the bench loop; re-run the bench with the current source to capture actual row counts._

## bigint_ord/range_gt_ordered_10

**Description:** Unknown query

****

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | — | 580.85μs | N/A |

_Rows are the planner's estimate from `EXPLAIN` captured before the bench loop; re-run the bench with the current source to capture actual row counts._

## boolean/select_back

**Description:** Unknown query

****

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | — | 98.49μs | N/A |

_Rows are the planner's estimate from `EXPLAIN` captured before the bench loop; re-run the bench with the current source to capture actual row counts._

## date_ord/range_gt_10

**Description:** Unknown query

****

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | — | 1.48ms | N/A |

_Rows are the planner's estimate from `EXPLAIN` captured before the bench loop; re-run the bench with the current source to capture actual row counts._

## date_ord/range_gt_ordered_10

**Description:** Unknown query

****

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | — | 1.34ms | N/A |

_Rows are the planner's estimate from `EXPLAIN` captured before the bench loop; re-run the bench with the current source to capture actual row counts._

## double_ord/range_gt_10

**Description:** Unknown query

****

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | — | 1.07ms | N/A |

_Rows are the planner's estimate from `EXPLAIN` captured before the bench loop; re-run the bench with the current source to capture actual row counts._

## double_ord/range_gt_ordered_10

**Description:** Unknown query

****

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | — | 617.01μs | N/A |

_Rows are the planner's estimate from `EXPLAIN` captured before the bench loop; re-run the bench with the current source to capture actual row counts._

## numeric_ord/range_gt_10

**Description:** Unknown query

****

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | — | 1.02ms | N/A |

_Rows are the planner's estimate from `EXPLAIN` captured before the bench loop; re-run the bench with the current source to capture actual row counts._

## numeric_ord/range_gt_ordered_10

**Description:** Unknown query

****

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | — | 744.64μs | N/A |

_Rows are the planner's estimate from `EXPLAIN` captured before the bench loop; re-run the bench with the current source to capture actual row counts._

## timestamp_ord/range_gt_10

**Description:** Unknown query

****

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | — | 949.15μs | N/A |

_Rows are the planner's estimate from `EXPLAIN` captured before the bench loop; re-run the bench with the current source to capture actual row counts._

## timestamp_ord/range_gt_ordered_10

**Description:** Unknown query

****

| Data Set Size | Rows (est.) | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|-------------|-------------------------|---------------------------|
| 10,000 | — | 678.64μs | N/A |

_Rows are the planner's estimate from `EXPLAIN` captured before the bench loop; re-run the bench with the current source to capture actual row counts._

