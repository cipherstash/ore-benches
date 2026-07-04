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
| 10,000 | 10 | 507.93μs | 26.93ms |
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
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) > eql_v3_internal.ore_block_256((('{\"c\": \"mBbLi`8P*<Ll8x=AA(O9n|MIP7Ob3@>aPKJ^5OcHEA!TJl5nhx3H>6(AjI%-tF{kl1R&ozsxQ)N$2fQ*aUA|#V@6+nGZwV9)}?k~Y;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a2269f2a33fef19cc58299a7ecdca47aeed18fdd7d21d6dac4f89a07d5a2e08028efd9c5fdc5f0ad15d86b5bb6f9754306c5c37cb72bd9a30342b6e7d5b76415daa3eb772635bc4fd6e1348eb2e7c1c7914bbb299f6e0ce7ea58698e01727fe3468a9fcaed3e9586799b0bb6c1a1f2e3972d3872251289191e6b0618fd7127a53ce53674da7f9769f5fda4d6102af7fe58dc05cce9c489fb3b83577a764baa33297906a01e62971ad02572bc5782a5b5abd9a4388e618182c85de39908ccf35b8b1dc964cd46bcc89743711b650e0124b93a2d421ed861bb69a2b4ca5ba86fbd82e47388f0d2597a8ada3cb61ca2ed3589997b57aa53dc944cafc83f0aa7c06835a0611e9acebc71ec26cda4645d3efcb72\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
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
| 10,000 | 100 | 926.61μs | 40.01ms |
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
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) > eql_v3_internal.ore_block_256((('{\"c\": \"mBbKe!>E(a6h;+Ol2$^u$nMR=7Dw)oamTZr_5CBIOP+{oR>4+x5JP^%AQRlq_N8QjmDhOEVxM^OsFdYIs(^#<Gh!}vkb1ea(xrA`Y;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a22189e6a8d7cda38195e0fa0564c98ac79056600eb713a8472e9576ff306f467148c6dea66afbbc3bd49fd39bcd4847d6af71c18d0cd3f3e5ef459506c8437082ee3d74619cdf2f9505b6881e8b544e842c2f98a6806d2ea16b04593e8075a16d50e26aacbed1e7ac03a8f833dfc30e1c585a9267f211bc02dd9593446922e486354c1dbf58dca6c234ef7bf96f2f0164e68eaa32b2163f9555095fff06325b39c875efbcb2383e6ac919c04b9d3333b12cfcb2ac3627499ac487b5d833bcaa0fb45966835cd8f1553b8c0edc1c0842bb582557579dd56e2abc158ecf983744b3fee81fabb6a4e0e036ee05621c0f03542cb59002e1171fffec415932ab9eb8c3a8153604f0eee5edc62ca28dcb51c995f\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
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
| 10,000 | 10 | 481.58μs | 26.01ms |
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
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbLurbgOmR!vbw_A-;nRW)(M7AcK`MIm}1nMce#S&J|K_T6jOc0c{ZAg@aaEAHRE?OGT~Gt}}T;J8ixJ{`COCqH}3K#7qlETwi~Y;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a226bece6753b211e0f40e61f327da8d83448351a78c2a592e5c9becd22be830c5ad9c9edb4b3fc119f314a3f765060bef11b21189721ee7c41a7fe14e83840bf450bd37208309c82e32da8258fc0c21f91e3f0d2a88c87fcad902a2b82c8ed1dc31ba25a3c4d3a27f567d29a5c5f5ce890a61e4a69a4a271ceed06e144ec916dbb0944c44807f819d4bafbfba746545c35f26c5b082e1e1f9e758c319d180449e93cd14440b43e5d273ce01f543595f6c9c4bc4f8ff940acc3741e10ab45fd4b76878055ed5f1132e0db6af1390a47c522bb06f077757becfbdaa68a46daa68140696377257ea9d912ab3023446f9c22ab5d553bf82b387fcae426e32f9a74117afbe6fc6b6ed1eb0525a3177b180c1212\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
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
| 10,000 | 100 | 926.98μs | 40.61ms |
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
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbKPhB%7y+hhU7K<P7GlkTO&7P256ChZ<I-56fnnUD}SZJQT{FvyF<ApSQdh6sJ4KSEPdNS|Okw%R=aPQPQGa*0Nz##Ce5qNR3WY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a2299f857e14fafa376a990a73e6e1a4e6f222e93bbbbd34898fc751fa25e246cd6020e3e246dd6a3400f981fb56eeada9669652100acf880f63f30c0663f1a9df560615399ee547e9c5bfd9f9218b3847fc8d216e32722e832d276188ac0ed85c5ce8a63a680ae60d286d4fe7969d4cb1f115f37117499137cec8b462259200783aa9d2ebb553a6b597ab71ba8dbc4a70b983ac7a16654732efeda143496a95a38d4fc33b335d610a6c3a82c9b0d87e935fda8dd8b47ec6ab8d413dd30560d76d241f46c048fe9573ac68af944e948413cc2209c4812acbcdbfd620a83b7b49e3ea41a09b097e8751a1868c5473a793f51ae3d010733550905e60578133e762c0f6855d995cf9814856a2e3b086c6c4e0e\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
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
| 10,000 | 10 | 547.44μs | 26.28ms |
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
          "Index Cond": "(eql_v3_internal.ore_block_256((value)::jsonb) < eql_v3_internal.ore_block_256((('{\"c\": \"mBbJ?G)bT;OkIpyMmr}YB9w!~7AnMr<~wmW<Yo7wu341fS*>9`^ct7MAjU<h|JlKa>mIw-y(vLaVFxVlJai1*CmF5)CLmih;iYzAY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"integer_encrypted_v3_10000\"}, \"v\": 3, \"ob\": [\"6a6a6a6ab06c241e413a084c2f2db73ff9b3a65b1371a8aff562d9da3bea15a04f6b769a39b089467ffd328aa870db838f4f023047d3775fb47f5c7d2d1631c795176e1460cb7f91f12252f877e7cd2d93b71ec8363578de5b1d06b35a2fc3c8a768f7315c94c75cd4c047c46455b718955448fbc7e8151d504d143d8d7055b2e717822be1cb0a22324a493bb0053abee8ab27eb7fbd8c87f98765faa75e7a0fb0fc578edd44c3439064686267ef144f4938e8ab37f33c40ef2b23895451ca6265dbe332cb7d20a7ea777f9f72223a4591ac0f3f490f738431eb38dc919d9807b981425db8853fd20730d87088881a276ff9c9c9ef92a0ba0941a7d7d039d0cab6109169776f4c2a012778143c705b11c432386e23d0c7778206f6df7c7d09c4c414880e3c929035fb9ac05a4e481cbeaba84478b914dffa27ea8f3a93bfd82911295bae5388e0eeb2b3530637210f0c11d361e42a85d39cbe96cbb7bdbf2e65abcbc4850ffb058282a41146c3aabe99976d53890f21d569bf573a3622e860ecb61beb28b84acf812faa929bf316e4372509215e6dbcadb9\"]}'::jsonb)::eql_v3.integer_ord)::jsonb))",
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

