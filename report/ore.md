# ORE Queries

[← Back to overview](./BENCHMARK_REPORT.md)

Per-tier query performance. Each scenario lists its SQL, the indexes available on the target table, the indexes the planner actually picked per tier, the timing table, and the full EXPLAIN plan in a collapsed block.

## range_gt_10

**Description:** Range query (greater than) returning 10 results

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 10
```

**Parameter:** `5000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. The bare-form `<` / `>` operators inline to `eql_v2.ore_block_u64_8_256(a) op eql_v2.ore_block_u64_8_256(b)` post-2.3, so the index engages without query rewriting. Query: WHERE value > 5000 LIMIT 10.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `integer_encrypted_v3_10000_ord_index`
- 100,000: `integer_encrypted_v3_100000_ord_index`
- 1,000,000: `integer_encrypted_v3_1000000_ord_index`
- 10,000,000: `integer_encrypted_v3_10000000_ord_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 502.66μs | 26.89ms |
| 100,000 | 10 | 541.35μs | 26.12ms |
| 1,000,000 | 10 | 541.58μs | 25.77ms |
| 10,000,000 | 10 | 549.02μs | 26.25ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_10000_ord_index on integer_encrypted_v3_10000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_10000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) > eql_v3_internal.ore_block_256((('{\"c\": \"mBbLWBsRb)6eQXj+CUGLmauNb7J=5#Ij<k_Z5aA)Yl?GyMFSlq;y0qiAZ5m>bu(MfY$S*X$T)tFQsFESJ|`4Gvy~+(%0~u-L8W$KY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a22490886344f6e1d4cddadfb8776875fcf3c3e0abe2e76343ec6e50141fd235f9c9ee19780966731ee39b0b511d16e8bb36edee7dce9b67b0b6a4a38a49a1eae5d1e0bb93557f581daff17ecc75f9f2f9c17eccf907e6a38a8d409ae85f82542a420c88259d8565b0f1d4b813bd65dd1f0022d0f0826b18d7b3cfe226c5c6d47de8ebe6ffc4f235c645adeb69f6a6eee70b7cfd659a480ae4913ea1b3102db337a702401a344c7ef95836dce6d4cc3bb02f291ac164dac8a208fd0584e4d2574908de61026f54a98f534f85c85664d0b28518e3a52e39c88b8cc5eb124e1ff85b3f332a633b03b0541fbf45e07b67e1c5b96b63fcc4820f888b86a188b44c481aad4a47b515547b7fcde919d2163c57757\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_10000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4850,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.79,
          "Total Cost": 7321.57
        }
      ],
      "Startup Cost": 0.79,
      "Total Cost": 15.88
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_100000_ord_index on integer_encrypted_v3_100000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_100000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) > eql_v3_internal.ore_block_256((('{\"c\": \"mBbM7>vz?)OlK0@C8sAWl#gx17Gc|N!NlqitFJr$Tj%C!2JHMAXiB%lAlQHv^j3n!Upn$Ih$!<m8EM==D->;%{yCqgfO1vQ%cXW<Y;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_100000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a224d70c5ea374cca13c04aaed3707721fe675e47b0fd068818959436da37ff1646c9c8e3fb17c006b23834b591f29bc5459622027791bfcbb080e9693c4c132dbeef804e87f52b24950fffe745a206dd6712f10cee0427ad228e3a61039afc59b0e6d82848f5c189a1328f9b9e9b8e5410af593917ff9f371c7f6d354b9ea47a16df425b34404c29e3d23b6049bf25fe8bd67c1f095171907a69ed1f9608aa1e2e424020526692f76d77e05771a91e0d7ca1de6ea87b061169deb3d797d7f43bf8dea32d21ff12e4aabdb2956c225defe9c17379d7e69b042353c5239a8673e11890048c4a1ecfa0a54fac43b4e5791ad2db7b8d56b1b24d3ef80a8e47e59909aac4c6a147403ae29cbbad6ceae9065ad4\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_100000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49500,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.92,
          "Total Cost": 73448.67
        }
      ],
      "Startup Cost": 0.92,
      "Total Cost": 15.76
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_1000000_ord_index on integer_encrypted_v3_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_1000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) > eql_v3_internal.ore_block_256((('{\"c\": \"mBbLmQ9%&7tznj=CG@e~bhK5(7L>6%{yVA%!jbf*8KAiiF~piFmi0BnAXzKt+KSI#^4ieD+h6Cz5E<*V_e%AWn%0hE17B_Wtfh8gY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_1000000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a225cd2430565ba1a784083ff052802607d00cb7d7e94883d348720cb39e670c30fe11a955210cf6309c20887f0d02efac0806cbde21d80f3965bdb17c9cd42382e52d9d97d1017784e7b3f1e1dcfe15c224966ce98b12913cf9b4a0dc643ad00862d3868cd9e6e73b189b7be4ed120b51fa95d425a6c4222656b73726cc9a6e634315437b04a978949e948690d6efb3328493f6819bebffbc178bdc345a84481c2faa7ef50ce83c15c69a91aa4c7beef189211dd12c6ad83842b928a403171bef4aac1da9bfa84583a955108b85076ee2a8f2415813fec8d712531a24648e3450942f015bc12b79154b51dabd1502ff4dc999662015a348a93db7b46deddc37fd5649b5869208c225ffd72b8a5bb448521\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_1000000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 495003,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 1.05,
          "Total Cost": 734361.89
        }
      ],
      "Startup Cost": 1.05,
      "Total Cost": 15.89
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_10000000_ord_index on integer_encrypted_v3_10000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_10000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) > eql_v3_internal.ore_block_256((('{\"c\": \"mBbLPsC<VAeZ0*S4!6;h@q{PD76HP7$YgbIPLrA=ZGXB?oKZ?$-Mys5AoCE(H~Fsx5q=sfnPs#6P%$XTG3?tRZ@Q<^dG=M%bftD-Y;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a226aa4c7f033219508c87aafbb5d35ac5e55474798d0c0a4ad729691cdadbbfafb2d9ccbe3fab97a5c0ab07fc6c807de0cae7900f000cfb3cc09fb33996117d5b10eda8defd7533f233431030c6afe1b311e3685e493b26dae96452712d2da99d5e0c40f46e3813c84d883dd679ee03bb06322bf406c4625103e6e81c5c5ce351e4fcf8549ec05b2ec041708d73d3509ce5b537618b251502c8919138010dce9928fa9cb5e69185c52e4734a30b1dc2a44f73579b4c06f26a803f8e00e10d820aad10669b675796c4979906b8fa070780fba8fe01ef68b861512e52a8ce287691aefdf26c343cb59beccbde14b3db602a9cccfb1cf2d0a0f39a9f568882c6bfd9a93de2fdf945cc53e969581b00dd11d81\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_10000000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5050002,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 1.19,
          "Total Cost": 7376457.78
        }
      ],
      "Startup Cost": 1.19,
      "Total Cost": 15.79
    }
  }
]
```

</details>

![Query Performance - ORE/range_gt_10](query_ore_range_gt_10_chart.png)

## range_gt_100

**Description:** Range query (greater than) returning 100 results

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value > $1 LIMIT 100
```

**Parameter:** `5000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value > 5000 LIMIT 100.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `integer_encrypted_v3_10000_ord_index`
- 100,000: `integer_encrypted_v3_100000_ord_index`
- 1,000,000: `integer_encrypted_v3_1000000_ord_index`
- 10,000,000: `integer_encrypted_v3_10000000_ord_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 100 | 974.64μs | 42.60ms |
| 100,000 | 100 | 977.05μs | 34.16ms |
| 1,000,000 | 100 | 976.53μs | 35.87ms |
| 10,000,000 | 100 | 1.00ms | 37.31ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_10000_ord_index on integer_encrypted_v3_10000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_10000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) > eql_v3_internal.ore_block_256((('{\"c\": \"mBbKU56O`Yy;m%(Dx9`_dev3L7Ufb$9A@2;5Kibtu6{_?$YJib)<G`BAV#Xb<72ZJ+xTyepk~LN$6p$$Y{ZZA1UF`<dA4cnHl=o9Y;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a22f7940434ba5d8bd58e8012390a42164589e167363538df78ab9fe9fb7a378cab07a6534c6dff35c861f28333799c5ad21d84f5b40a6958bd229511ba57f0eece264ff386bfb6b05b1a834119b7a9086cd8f746b3376124ec58b137caea8bf427c238a49dde1ad7c983f5adf1acd84a8354c3480c9fcb5e7dd2a90e9ebc730faf80a83d279cdf6e7d064ba83bab30f7e9199e46b87dd235dd029450a878a33247724fc340886d9510d285e5bab097735b2cf2c6f92566da2c75d919af14c6eab5f7e7f62bc2e9324af2fa2203a082e15526501a5c945e07bf8859c93597e63d49119d63a14303505d65b9917b92aa03d54bbebf74bd1d1e2891d20675e292fec863bf1d14093ff9c5052a0a1afbcb8d93\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_10000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4850,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.79,
          "Total Cost": 7321.57
        }
      ],
      "Startup Cost": 0.79,
      "Total Cost": 151.73
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_100000_ord_index on integer_encrypted_v3_100000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_100000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) > eql_v3_internal.ore_block_256((('{\"c\": \"mBbK#i|jR0mIE6R#Vn9^v*k_176|=1zfZjEt{rEpMy*}GbhPXwt~FW2Ah@YpZ(4)<M;iBWfr#%j_UnCl9#xhs4c()bMzCw%G^KW7Y;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_100000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a229befb2abf146a7b30e21e3e247c39b3a20af8bfc5fb5ff1f5bc9638fe19d3b793e9a2c623eb3e4647a72cc8cf24f5290e8c8f4c4b28518aebb48b56b74d35db66a4f09d7aa7bb1ff88da81ae5e48d56292d219fa4f52e21c151a073c66debaaeea91f181e1faddb6568ef19cd4baf5e47d34bbaaf1603d137d60f36d1529f69946d7d3fcbdaec09f018958356e59ee39cd6c23476429a5e1c81cab96b4033d50bf4559505de93c207bee3fa62e5e8c8832346b5787ad5a8cca98cfe989f5d5c80611bfb87fcf92f79e1528b77bb9b4b5b4311afd90773349d70677517c7c316470d28a475ce4dd2043af0f2e786b8ad5652194ea0c829248f3b38db8178319e900b6973b5bd4c351c119d9226e684232\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_100000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 49500,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.92,
          "Total Cost": 73448.67
        }
      ],
      "Startup Cost": 0.92,
      "Total Cost": 149.3
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_1000000_ord_index on integer_encrypted_v3_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_1000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) > eql_v3_internal.ore_block_256((('{\"c\": \"mBbK2YcyBewbxQPWU}V@%St1}7Qzqlz}9)a$Ml_>v4_uxcrOf@r);gnAfoJNLqP7KH!)wfuPw1DdMBws&RD7UEU(=;M&(kcfTea}Y;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_1000000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a22e1cd5d41661cb22e5b2385b8b97e1a9f317d623b4bab1f6d4528b82da90b25ff919d14d7bec2bdaef50adc8c7e4951676f362392a11633a0986dfac9755d53509737858ad643fcd0b2fbb1b2be383fa693e33394ff427ac2592f63ee7cffecb58a916253bf9f55bd00a5fdeb73e037949477ef2458af2aff0106164139462075279af880b889e3045b66560012de8e1457d4a47d110dccf889f37f966b10a67907bc369a9f37195d17d9c9f4ba5b6170835fa1c15827b00a21ee00772448b0068244346a4acd273443577ec2404e1dd257799ad9fda26714c006f0968b24ef25a2b16b04b6f10e2ffb3fd87e52c57d7e131c76aba1efbd0dd997dcfda1fdaa7321e4cc1d15259472f6595680c965f56e\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_1000000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 495003,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 1.05,
          "Total Cost": 734361.89
        }
      ],
      "Startup Cost": 1.05,
      "Total Cost": 149.41
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_10000000_ord_index on integer_encrypted_v3_10000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_10000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) > eql_v3_internal.ore_block_256((('{\"c\": \"mBbL}UZVLggZ2PgHzVLUw9uQx7S$IQ{eMo*Jdj@d$_PAfiVqCvKeA}VAONW+@M$?#cOZn_9w9#KG~mY~%Ah`77lQj&l{H{8_oa4WY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a2287e1ec8ee908056d639839e3ad5f07a78213def4d88544e05668bf927ebca7d36432b9044e4d898a0917da9f4bb6bfa716e059f123b7aac2d6214959e435ee36249a1a0a7a496d092a98d59f6a7dd30044ccddfb9c749c1bf888d0a311e193968b604e15a56bf3fd33c508bd0b28d5c8016bcc1d495036a87b5f2329fd57d2447a095a3b3c7a9a485386451add7815f96ff733be3f565e9aa0a41fd29f56ca2cb6f2e7d29b25d4775006e5c5da5f07a7ee28bccceeb145ea7e5b6022eada5ef31d1cdfb9d7b49a8da68b8ecc9b2d85b31e895b0e7e8c60df2f8947f11c9ede1fb2d091fda1c1c009292f0faf0592b7651514a67ab9bff6003651c5a885757d2c5c40b582eed3792645563a0bb2c1acf0\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_10000000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5050002,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 1.19,
          "Total Cost": 7376457.78
        }
      ],
      "Startup Cost": 1.19,
      "Total Cost": 147.26
    }
  }
]
```

</details>

![Query Performance - ORE/range_gt_100](query_ore_range_gt_100_chart.png)

## range_lt_10

**Description:** Range query (less than) returning 10 results

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 LIMIT 10
```

**Parameter:** `5000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value < 5000 LIMIT 10.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `integer_encrypted_v3_10000_ord_index`
- 100,000: `integer_encrypted_v3_100000_ord_index`
- 1,000,000: `integer_encrypted_v3_1000000_ord_index`
- 10,000,000: `integer_encrypted_v3_10000000_ord_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 459.12μs | 25.99ms |
| 100,000 | 10 | 517.92μs | 25.91ms |
| 1,000,000 | 10 | 512.07μs | 26.72ms |
| 10,000,000 | 10 | 480.69μs | 26.00ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_10000_ord_index on integer_encrypted_v3_10000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_10000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbLl{C%Ofl0~-A-(IfQl0o0Z7TILl#*xnD^%i>Yz!Km6L-a=g&)uxVAhnj6unq99%|*}#SxN<x-r@zC+H;@rWPKyoii9Ij*rj%1Y;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a228a2443aca787f0fb57be71172c456978a43b8a5b775baf4830ae4e374c2351d8e3ded9ac485f5dbe9599ddc2bed03a2e44ba12bc2e5e2c82790bdb96b3e13b4141a0f3dc497958fb679b5d8063b72f903e577640446e5fd25c572ab64a5a0ef6697cad7017fd06421facb751a5572c4c0357519d4a629c7311eecf341e6de0ee55b84c6bfd20dbe3778d4e22412dac66532ff6de23b2451a0f49da3b97513fcab360994e62883f02f860799e6954f70dae27ace2887d7f42f698a481bc62de663332c06d7a6a3abb6a11d71ad827dda203b92695883a2043c2e0a0996bb71429c42a70c80d25904af5441ebb808adecd349bb24aacf15eee3861486f41aaae6558aad6baaa206a9f0af6a6c09e95ac1a\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_10000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5149,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.79,
          "Total Cost": 7418.8
        }
      ],
      "Startup Cost": 0.79,
      "Total Cost": 15.19
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_100000_ord_index on integer_encrypted_v3_100000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_100000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbKFTD{B2qpFwEU<I4JC#k8#7NrvK*_&%#Gl-XheaX}cL^YO*{~7qiAk~ErwLNeRo)xz<T*TRxYjPI;DY3sF%>S!1SXX<L^QCrSY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_100000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a22c88c465fd65d2916b9ff115fe0cecc3ebefe1e965e1679461b1999d48c334982310fc4af06ed982586b1175159c160ef63859e5ea14e4dda15a488e29f78594c84ae95a1f82140f4e5c90d60a5a6c57be60faeaa4282bef523e469219b5fcf8df1b97d56128592c48d66a6b617645e999e0fbed42a1da276fac8cb4b04f158b1afed5aa27940fb92f2ffc0085a45bcd60ed05cb1f35f1c07102b19fb57ae85f789a1a6562e261ba3df1571a4e76a5ddaee311599814c9e014f30800049b05dbe42438bf7ccaa41f77266e6bea8bd9e9cb6ed92c66c799bc80d78b4ba9952fffb4e00f0ef3fe3ea51a39022bc4043d948985f0b2d5aa317dd2176575f4adb2fc089b455892fcdafe625c4aa35b7480432\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_100000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50499,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.92,
          "Total Cost": 73774.16
        }
      ],
      "Startup Cost": 0.92,
      "Total Cost": 15.53
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_1000000_ord_index on integer_encrypted_v3_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_1000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbKXO((pb&-TKHzogfx6Th{@7JJGi@x~wAt5vp+M86W|kW9++Y$a;MAPDcUn`+#~$*tzvqFn?nAbMI@dhBP6=BAH}DSv3-zom9zY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_1000000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a22ead7035b93898b22b6be67d1fb64aca6f2caede02374ac8c6a1cbc6eb472121a86e1d001c89ec9c70d69c0542eb9761829d9e500050d2e8f3820878165c30c7326f717c2e803e21f01f27b42951963ee0998348cb3639ccd316c0bf71f2b30eb0a8473661604cfe0eb298de6b5d926ed306715bd34e5d97478616eac80e414ef32b7b5382a5aca4bfb9ca8dfea00e29af9ab50296c6e474ea61730c54ea0ff77274cc042dff2bbf92219228663fa9e29d8dd4d606883f92c3b0afaa071e381e9e811a6b4100744f7b818de57da629d1acac2a5c28eb1a93d6d64f22264e964eaa521bc724f5ad3a4ce651e54c334bc265be6dab56524fd1b438ef1697a76e85cb7cdce7134d5af04970e9ec0a0496eb9\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_1000000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 505002,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 1.05,
          "Total Cost": 737652.95
        }
      ],
      "Startup Cost": 1.05,
      "Total Cost": 15.66
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_10000000_ord_index on integer_encrypted_v3_10000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_10000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbLd@jaGvOfiXZh6S2Kt>Ht&7Gn>yay5es2-X+@8~q*d&W)-0_A$c5AQxYuS-dW93T%B+$3kDPh{Y|K-DWhu3obrMC^4{BwxxDqY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a22c64e22de1875dbc1aa91b3918bdf16eda0eb462640a7a718dbcac6d51970b717b9f3a882d5b7ea24d6ae323f6766b239a6c08dcd7c31fab33f4e2e8fed569b0813d0d4915f0bfd0542c1fd57357c06b338df05d167b3fe2afcbd4adc50edf32bb0abb3dfa46664e22bfc90a150e9de133510098754506b2ebcc3787a999aa89773a1bcafa354cc0f90472a14c0a6af39d946162aec5622b10f246f0a24dc3fde6bbdc5fe5110e19fe6f0856135077c8293a38b8df9a6d6dc1a6b5049f22381d8edfff5db644320384bfa8fe3752a2ccaa82c4d32e0a69ffbb6a5fbaa43d875c8754fb7b0421accd1d3a7325bce897ac88f16f92b8c84aa859392632231b1221088005830210e3b03c97150d36eed201a\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_10000000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4950001,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 1.19,
          "Total Cost": 7343539.07
        }
      ],
      "Startup Cost": 1.19,
      "Total Cost": 16.02
    }
  }
]
```

</details>

![Query Performance - ORE/range_lt_10](query_ore_range_lt_10_chart.png)

## range_lt_100

**Description:** Range query (less than) returning 100 results

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value < $1 LIMIT 100
```

**Parameter:** `5000`

**Table: `integer_encrypted_{rows}` with Block-ORE-encrypted integer values. Index: functional btree on `eql_v2.ore_block_u64_8_256(value)`. Query: WHERE value < 5000 LIMIT 100.**

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `integer_encrypted_v3_10000_ord_index`
- 100,000: `integer_encrypted_v3_100000_ord_index`
- 1,000,000: `integer_encrypted_v3_1000000_ord_index`
- 10,000,000: `integer_encrypted_v3_10000000_ord_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 100 | 907.54μs | 43.01ms |
| 100,000 | 100 | 941.48μs | 33.38ms |
| 1,000,000 | 100 | 867.27μs | 41.76ms |
| 10,000,000 | 100 | 896.39μs | 36.92ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_10000_ord_index on integer_encrypted_v3_10000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_10000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbK)I?FpYxps3J?f(PG%{>If7PpY{-*4+!p?MHJKsJhNkZ_gMl`{s!AdG}a-(vN7Mf~u}Cm$k-M=GK89Q;$7KeK&;jKXY552bctY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a2200151d9d8f251f9a8450cb7f8965238f6bb402c5a8912c063f8daefbccccc742bb890860e7df922f3d97fd797c481026337b5ff190ca8ed5db4ff2dc8b0831adab25ed5f366ed6985ba40b14db3d9efcbd2025949decabebe172a6bc0e3de33ea373c0f01020ebea7b4789385cdcbe2eb8a1bfb8c4a60540ee584c91ba5dd15f237d62304abcb81620a7ae713c1bb881c548b82615244149082f42ab897df38741fc8ed64b86b7c4d1c9706b9bc64efe80581f7f4200f4a5f3d776bb0ef661806b8737f0d2719b663109d228fcbf5b4c682319b3f1bf0d3eddea3f3855c2397a7373e2fb46c93129eb8a9fd62370526d142110b7761da40bf3b9a6968d2a77d4b9fc908854497b2418db64b4675da1d9\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_10000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5149,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.79,
          "Total Cost": 7418.8
        }
      ],
      "Startup Cost": 0.79,
      "Total Cost": 144.85
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_100000_ord_index on integer_encrypted_v3_100000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_100000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbKLXPjZ0wxB0~Y%m_Z{ihzp7M{lm2_~<2I!E2$C5n@KhCpvy95oljAbyP01^Sf*)8~r^ajk%0>WLZ1cp1!I2WS_jyjns>rKNUZY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_100000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a22b63b960c2bc150eb18cf73aee22c17bbdbecc429c0a90997726b60a0d6127b83fab39e26bd8c9c6049babfe679ad1a5cffd0fa1f413526365cf252549dc1b9bed29d5ba969c24e2abd76cb8e9be4e98c718ed94a5417f4227dccc946e0f1d95f09719de46ef9383c6ed870cb1c9b2a70a8fbae841071aaa856f6b78fb230e8551aee998702d5baa038147a2621b440d5c1aea4df11474e33ed9e0420d6f885d8a5ee381a5fef942a4f930245b0f8845c26dfc4b71023cc357c197a7ce7f37f6b4283e400195a830d73568da2311fcb4eff4b788ae5343c0fde034249a15feec02a5c1b99256a2a77e00e9878275c15a20ef2e091c2a7611d92d267a5f785e1a9f6de9a9a362bb09d660ee996eeeec625\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_100000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50499,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.92,
          "Total Cost": 73774.16
        }
      ],
      "Startup Cost": 0.92,
      "Total Cost": 147.01
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_1000000_ord_index on integer_encrypted_v3_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_1000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbK1?UWs05B(c}lWsY?n>Dh;7M3x<Zh;ni5%Y~3)~DX7iKKEqxbvRGAecE;zw<a+v*bISLh{*Y0Ec0h-y2_igh%qso|a)*_N8`VY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_1000000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a22525d6862ef1d96f693d80c1778bfc5f12fb17cd78c0abd7a5c66eaf97961b0fbccb2facb9a8b20764743ab536d3768686465a00d736cb03e605cad68f45064a4f5e0bc50d1c2512874928f746b8fb819da186cea14fa0f4f69e0bc32430239ee50b5fb928f937ef6058b84185dd0cb10584ed2ccb2221b5b9a5e1b8910e81846a2a5795b17b3f4659c34a30e6ac187ccdeab77588ace709d3f66094aca19470f9e98132123075d2fdc27f0edfb99314366ee59b912374161e3186118d1182a775f7a4c112b447ee0d9e67181a0bff4937087bd829d7c34bdcc0d7ce4e89b2e9cef8ab9e6a81c17d6198c3babf199ab7846c2117234541b7d32d9beb5051c4e37b8e56b3959193843e450de4f8178deef\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_1000000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 505002,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 1.05,
          "Total Cost": 737652.95
        }
      ],
      "Startup Cost": 1.05,
      "Total Cost": 147.12
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_10000000_ord_index on integer_encrypted_v3_10000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 100,
      "Plan Width": 1064,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_10000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbJ*qyP<HXMWM)kFLpka#C5u7LM|(i*CU2qAJ!$QXl2^p#MbcQ=fXoASp(lEVz$@1zD|uO@1$9ZAdo+_3#2t_~@_;LlbCKRi$=eY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a220cbbfdd194506cd9d450d97af4846b14a16c993321bd4655404405d5180bdae7aa718f21ff459f7d4c98195a46ddd0841fcb2ee8db127774f5960efb80d26458ce40d96ffb2c1caedb96521595a2c3e83867c8c665da06f57f28e1ee60d9e883b058838f0e810f185951a325480644e7f384c2b2e08f59b0b57ca41e2e333554b92b76579f75662e439f3d0468aba6f2967a888df221bef3f0b9f49e2c6a16ac48d9b1e6869492d006caf44bef6bd5c4ed69fa19057bdf1bfb8cd2fc027371159846a2e45696bea01fa234b4197c7d90c18da79ca2a4bdcbf8f6c69e0e72053a272a7c3e7ed853a6d455c1bc806484746f9b2a09f32fe2a5413d3ae9c747695fb1e547a435ed266fd1af081ba1e32015\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_10000000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4950001,
          "Plan Width": 1064,
          "Relation Name": "integer_encrypted_v3_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 1.19,
          "Total Cost": 7343539.07
        }
      ],
      "Startup Cost": 1.19,
      "Total Cost": 149.54
    }
  }
]
```

</details>

![Query Performance - ORE/range_lt_100](query_ore_range_lt_100_chart.png)

## range_lt_ordered_10

**Description:** Unknown query

****

**Indexes available on the table:**
```sql
CREATE INDEX
integer_encrypted_10000_ore_index
ON integer_encrypted_10000 (
    eql_v2.ore_block_u64_8_256(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `integer_encrypted_v3_10000_ord_index`
- 100,000: `integer_encrypted_v3_100000_ord_index`
- 1,000,000: `integer_encrypted_v3_1000000_ord_index`
- 10,000,000: `integer_encrypted_v3_10000000_ord_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 526.53μs | 26.50ms |
| 100,000 | 10 | 549.42μs | 25.54ms |
| 1,000,000 | 10 | 533.49μs | 27.02ms |
| 10,000,000 | 10 | 543.56μs | 26.34ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_10000_ord_index on integer_encrypted_v3_10000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 1096,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_10000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbK-he;<Xee}+zGO09SS5fA~78gEJ(fl#Z2mb{EQMKu8T55}=^M;YcAV$D24^liSaPe6ZQ{)h-dF)3k@+E?r$CrusOulM5e5H0_Y;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a2200d8f72210923597a171ad266453b215e6473ab07a8e310ed7feb75e4ac69926402cc1248fca4f0819cf05fec3ef69433b550dac8f5993adc66b4820e630ac999c50617e3638ddebb8532dea97bc1e1215fa185340c6c2c14c9b1854dfda8eef328fad65dfaeab686add3b9ebd07fbbf90aa32f8fb596878e07d801979d1c34a3563720fedcf17855aca308e8341b5e37e41b2c8cb4624c18a747c91b038a116e6c3a07a326d0353af401f2aa9564ac66c19f87925f16085126490a27c16bb0b34eda3ce201b34827d4157d92fed075f3665fafe085a3971171c1bd3d5e0dd9b5bf4cdb655cd4faae80c0dff33b019f6c613838d9453980e2458b0b3ea45db71bc74d4340896927fe4d1ef40639fc025\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_10000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 5149,
          "Plan Width": 1096,
          "Relation Name": "integer_encrypted_v3_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.79,
          "Total Cost": 8706.05
        }
      ],
      "Startup Cost": 0.79,
      "Total Cost": 17.69
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_100000_ord_index on integer_encrypted_v3_100000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 1096,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_100000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbKMtz)X5AXWDA>)Ze{a9;?-7IUKW56CpW=hm!Q4<`~#)?1fbp6Oo1AR~9V??kFoo|kb;*$Kj@VH?U<GC(OGK1)uMK%xy~@}+iRY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_100000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a2208a4f2ea6045fd0227fd35672c13ed2b06acf3b1ff8e3ca02b607723f1fa9d102847fa67cf5bde64f39e76043d1c9462c541dc21a4584200bb99d9a4e1a66ff6c39eafb837e502f74b34f8038b1480b183d1caf36e67e1711f8c62a177685ed2ebb12be39b0f1a8838894e145efce628f13702a7e17f3d476f01caa87c53471528d3e031f700f4a5b3904b0599731316863164a746541cd8cba08be6ed4b3b44f6fe7e99f9e46ed77d512b27962fa0d284057f71e5ce25eac4f6b32beda95cd9acb3e33190af25d5b0d495429926a0a6095843e39012889c729c6bf98eefd8a14689b1a8d1f845e85aafddd9ce317ecdd93c359bc3c0f993f39d8eb9d1a0a885f29db45f9f2b28dfb1b8688843a15eb1\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_100000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 50499,
          "Plan Width": 1096,
          "Relation Name": "integer_encrypted_v3_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.92,
          "Total Cost": 86398.91
        }
      ],
      "Startup Cost": 0.92,
      "Total Cost": 18.03
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_1000000_ord_index on integer_encrypted_v3_1000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 1096,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_1000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbKu&>F<f5KBQKb|@I27U&zq7Ip}^b=ux!$$~QVJZBe5u)x4xpS1PFAOx~+i@wCXAhLiT*Zm&ldx3G7v#0kAFgD{C6o5o4JEeAEY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_1000000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a2201ac5855cc2df164043fb66b64b9870102bf83134a98c3d38125ec4a49c055afb2b22a0d5f21141f5394467d53147beef7e4a32d34dd3f0c4baaf2782374ab1b6351ff48477b49bdf4eeaaec2abf0eff153c247a5693e277e3af8293f89272bbcf3fb9d25229dcac6207143cb565cecac64726265b78a8834f22c248afad6174f0a1b050569a227baada68744b40e20a7fa976694eeabd8cb534f5fb746d70cff8d995847e85a8ad8bc09fb66c3d7be22722a306c933bfb927f0d1ca314f7e3b58fed5084f39039f7d298c857e60ca5cfc7048ab2adfed30181d53b21a148841e011fc24d33394150a237a2781435e07fdc38a2737f779c7999be5f88da07d4a9c62c9c14d3227d27832ea78b06d0593\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_1000000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 505002,
          "Plan Width": 1096,
          "Relation Name": "integer_encrypted_v3_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 1.05,
          "Total Cost": 863903.45
        }
      ],
      "Startup Cost": 1.05,
      "Total Cost": 18.16
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using integer_encrypted_v3_10000000_ord_index on integer_encrypted_v3_10000000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 10,
      "Plan Width": 1096,
      "Plans": [
        {
          "Alias": "integer_encrypted_v3_10000000",
          "Async Capable": false,
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbLM!LZm@RyJ%Bi*J#RGi4^k7Ktp1y)VEanN$@nNtg)i5pfhZqzBH#Ag_q$u=7zUnpYI*dvfopY{m%`0YD8Se;KqpG5ZizEv0s0Y;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a2298db740f0e43f420833b65d7338d47bcf878829cd80b00ae4ea0ea7d9aa2f236494ad4fbf9f06a4efb59f1b7203520abd9e77e9fea9478eee17f8eafaab26a1f397c513575061606a79c178bd990f7e5c36af0b7da0565dae1f0bfafd9b06508da4b0b9dd0ed9430f2b0ee6d203e083033e6a44bad70b2bf0cdda811ffeaa6f6181b55fe8582fc0e7e7c19334163c65f8755d87d790beea9f7c57b38956e64a17bb2b23725ab712ce81a135c2922b14cabcca408a0dc57971e33cd4ba22c429db0b57df7bccf4434c3e177ed5c240fa371955889c54dafa8e08c834e2afa6df20948a2cf1925a8bfd0607a7bd03e6d981b3a5304f5faf9f76c1ca3dfd2114ce902db6407c7bfe847f405f15dc53487b8\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
          "Index Name": "integer_encrypted_v3_10000000_ord_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 4950001,
          "Plan Width": 1096,
          "Relation Name": "integer_encrypted_v3_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 1.19,
          "Total Cost": 8581039.32
        }
      ],
      "Startup Cost": 1.19,
      "Total Cost": 18.52
    }
  }
]
```

</details>

![Query Performance - ORE/range_lt_ordered_10](query_ore_range_lt_ordered_10_chart.png)

