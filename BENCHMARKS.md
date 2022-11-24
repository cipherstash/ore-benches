# Benchmarks

## Table of Contents

- [Benchmark Results](#benchmark-results)
    - [Exact](#exact)

## Benchmark Results

### Exact

|                | `plaintext_linear`          | `plaintext_btree`                | `ore_linear`                     | `ore_btree`                       |
|:---------------|:----------------------------|:---------------------------------|:---------------------------------|:--------------------------------- |
| **`0`**        | `119.54 us` (✅ **1.00x**)   | `117.54 us` (✅ **1.02x faster**) | `115.82 us` (✅ **1.03x faster**) | `684.23 us` (❌ *5.72x slower*)    |
| **`1000`**     | `176.59 us` (✅ **1.00x**)   | `127.14 us` (✅ **1.39x faster**) | `1.65 ms` (❌ *9.33x slower*)     | `683.09 us` (❌ *3.87x slower*)    |
| **`5000000`**  | `262.09 us` (✅ **1.00x**)   | `130.15 us` (🚀 **2.01x faster**) | `822.92 us` (❌ *3.14x slower*)   | `711.04 us` (❌ *2.71x slower*)    |
| **`19000000`** | `194.78 us` (✅ **1.00x**)   | `120.15 us` (✅ **1.62x faster**) | `1.09 ms` (❌ *5.60x slower*)     | `695.32 us` (❌ *3.57x slower*)    |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

