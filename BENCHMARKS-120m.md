# Benchmarks

## Table of Contents

- [Benchmark Results](#benchmark-results)
    - [Exact](#exact)

## Benchmark Results

### Exact

|                | `plaintext_linear`          | `plaintext_btree`                | `ore_linear`                     | `ore_btree`                       |
|:---------------|:----------------------------|:---------------------------------|:---------------------------------|:--------------------------------- |
| **`0`**        | `252.51 us` (✅ **1.00x**)   | `142.67 us` (✅ **1.77x faster**) | `1.25 ms` (❌ *4.96x slower*)     | `742.45 us` (❌ *2.94x slower*)    |
| **`1000`**     | `127.88 us` (✅ **1.00x**)   | `142.96 us` (❌ *1.12x slower*)   | `433.50 us` (❌ *3.39x slower*)   | `754.43 us` (❌ *5.90x slower*)    |
| **`5000000`**  | `229.21 us` (✅ **1.00x**)   | `143.07 us` (✅ **1.60x faster**) | `1.76 ms` (❌ *7.67x slower*)     | `715.12 us` (❌ *3.12x slower*)    |
| **`19000000`** | `176.13 us` (✅ **1.00x**)   | `140.96 us` (✅ **1.25x faster**) | `775.20 us` (❌ *4.40x slower*)   | `761.21 us` (❌ *4.32x slower*)    |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

