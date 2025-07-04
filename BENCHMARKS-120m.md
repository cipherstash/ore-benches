# Benchmarks

## Table of Contents

- [Benchmark Results](#benchmark-results)
    - [Exact](#exact)

## Benchmark Results

The following tests were conducted on a single table in Postgres containing 120 million rows of data.

### Exact

|                | `plaintext_linear`          | `plaintext_btree`                | `ore_linear`                     | `ore_btree`                       |
|:---------------|:----------------------------|:---------------------------------|:---------------------------------|:--------------------------------- |
| **`0`**        | `252.51 us`   | `142.67 us`  | `1.25 ms`      | `742.45 us`    |
| **`1000`**     | `127.88 us`    | `142.96 us`    | `433.50 us`    | `754.43 us`     |
| **`5000000`**  | `229.21 us`    | `143.07 us`  | `1.76 ms`      | `715.12 us`     |
| **`19000000`** | `176.13 us`    | `140.96 us`  | `775.20 us`    | `761.21 us`    |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

