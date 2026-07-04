# MATCH Queries

[← Back to overview](./BENCHMARK_REPORT.md)

Per-tier query performance. Each scenario lists its SQL, the indexes available on the target table, the indexes the planner actually picked per tier, the timing table, and the full EXPLAIN plan in a collapsed block.

## eql_bloom

**Description:** Pattern matching using EQL bloom filter containment

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE eql_v2.bloom_filter(value) @> eql_v2.bloom_filter($1) LIMIT 10
```

**Parameter:** `Johnson`

**Table: `string_encrypted_{rows}` with encrypted string values. Index: Bloom filter index using `eql_v2.bloom_filter`. Query returns LIMIT 10 results.**

**Indexes available on the table:**
```sql
CREATE INDEX
string_encrypted_10000_hash_index
ON string_encrypted_10000 using hash (
    eql_v2.hmac_256(value)
);

CREATE INDEX
string_encrypted_10000_gin_index
ON string_encrypted_10000 USING GIN (
    eql_v2.bloom_filter(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `string_encrypted_v3_10000_match_gin_index`
- 100,000: `string_encrypted_v3_100000_match_gin_index`
- 1,000,000: `string_encrypted_v3_1000000_match_gin_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 503.70μs | 26.48ms |
| 100,000 | 10 | 1.94ms | 27.22ms |
| 1,000,000 | 10 | 15.91ms | 38.47ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_v3_10000
    Bitmap Index Scan using string_encrypted_v3_10000_match_gin_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 1,
      "Plan Width": 649,
      "Plans": [
        {
          "Alias": "string_encrypted_v3_10000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 649,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbKO$}mRqZZ|NevEZ;XK%jfX8MVs4+C6@huRQge{TnN1VPB-yprvd^in_!glYA{e$AhWDXbT4y%lRhDBxacC_9<QOVyd-G(@L^1rFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [684, 690, 438, 0, 413, 1646, 1369, 1826, 1070, 1334, 1030, 872, 1342, 1849, 953, 1548, 1215, 948, 606, 104, 1705, 587, 566, 1574, 192, 687, 1265, 899, 1129, 1524], \"hm\": \"fe348dd0003b5ce3173aaaaf495e30c6c4bce6201da8e9d6064d18598c818022\", \"ob\": [\"1e17dc69f8909090c00f2d549cf7c0571af2a3f32b85275b2bb943dde8b1a1df2f72ad9917386cd49096575761dc049e5ba711e2fdc2e3fdbfab357e00beb0fc5bc0dfced927aab195495bd31947d7aacf083e7046b8f503180caa4083372f1b0f4e9aed43d4735e87ecdeda4798cf4a5a5016da7eade2cf9461be2912a70c4e3f1b411b258548429f9d5ebd329dfa3495d6afba74ba169a925dfe0e7fca8800f9448bc0bb671779bde2f6b63862910a57e584a233fcee8c1a6bf08dcefaec8191bf3aaca5c7ec33c5ccead3aa19ad847fb6adbfadf4be7acf5e89a82fce2255d93480a0b6776588f879438c8a7f4bc24fcf13f77228cf41cd007922ce02a3e1aa56babed118cd11efc5d53d8ddf7453768aedd4375ce3f3a77559c6c4acd89b9be57fe6d79e30c25ea60f119d1e2c908b250393d9f4c05c14f9487d25d835ee1113eca12e02cfa2231d15f9aa1f1de1a3464d64c24ce5ad03146323f1a1881ebfabc860a4eea6258ac3cd70ff2eb091f4866c89937eca076fc1045ffc0c0d0483bf5ad5b85731896e81f629532f335ea944efbf5c09692b\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Index Name": "string_encrypted_v3_10000_match_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 264.85
            }
          ],
          "Recheck Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbKO$}mRqZZ|NevEZ;XK%jfX8MVs4+C6@huRQge{TnN1VPB-yprvd^in_!glYA{e$AhWDXbT4y%lRhDBxacC_9<QOVyd-G(@L^1rFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [684, 690, 438, 0, 413, 1646, 1369, 1826, 1070, 1334, 1030, 872, 1342, 1849, 953, 1548, 1215, 948, 606, 104, 1705, 587, 566, 1574, 192, 687, 1265, 899, 1129, 1524], \"hm\": \"fe348dd0003b5ce3173aaaaf495e30c6c4bce6201da8e9d6064d18598c818022\", \"ob\": [\"1e17dc69f8909090c00f2d549cf7c0571af2a3f32b85275b2bb943dde8b1a1df2f72ad9917386cd49096575761dc049e5ba711e2fdc2e3fdbfab357e00beb0fc5bc0dfced927aab195495bd31947d7aacf083e7046b8f503180caa4083372f1b0f4e9aed43d4735e87ecdeda4798cf4a5a5016da7eade2cf9461be2912a70c4e3f1b411b258548429f9d5ebd329dfa3495d6afba74ba169a925dfe0e7fca8800f9448bc0bb671779bde2f6b63862910a57e584a233fcee8c1a6bf08dcefaec8191bf3aaca5c7ec33c5ccead3aa19ad847fb6adbfadf4be7acf5e89a82fce2255d93480a0b6776588f879438c8a7f4bc24fcf13f77228cf41cd007922ce02a3e1aa56babed118cd11efc5d53d8ddf7453768aedd4375ce3f3a77559c6c4acd89b9be57fe6d79e30c25ea60f119d1e2c908b250393d9f4c05c14f9487d25d835ee1113eca12e02cfa2231d15f9aa1f1de1a3464d64c24ce5ad03146323f1a1881ebfabc860a4eea6258ac3cd70ff2eb091f4866c89937eca076fc1045ffc0c0d0483bf5ad5b85731896e81f629532f335ea944efbf5c09692b\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
          "Relation Name": "string_encrypted_v3_10000",
          "Startup Cost": 264.85,
          "Total Cost": 269.37
        }
      ],
      "Startup Cost": 264.85,
      "Total Cost": 269.37
    }
  }
]
```

**100,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_v3_100000
    Bitmap Index Scan using string_encrypted_v3_100000_match_gin_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 1,
      "Plan Width": 635,
      "Plans": [
        {
          "Alias": "string_encrypted_v3_100000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 635,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbLE9E2f5{Z(7DX-`9~7F5*48R1ytR5Agx`ktr4Az!6uI~#kR6qwhp?$E>_AXr6YO@n@(5^dBRDN3J$dKi~^RA5Rx1Ea!IRTW-rrFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [1070, 1849, 192, 1524, 413, 1548, 1129, 1215, 1646, 0, 1574, 1705, 690, 684, 1265, 1030, 687, 1826, 104, 899, 1334, 566, 1342, 438, 872, 587, 953, 606, 1369, 948], \"hm\": \"fe348dd0003b5ce3173aaaaf495e30c6c4bce6201da8e9d6064d18598c818022\", \"ob\": [\"1e17dc69f8909090c00f2d549cf7c0571af2a3f32b85275b2bb943dde8b1a1df2f72ad9917386cd49096575761dc049e5ba711e2fdc2e3fdbfab357e00beb0fc5bc0dfced927aab195495bd31947d7aacf083e7046b8f503180caa4083372f1b0f4e9aed43d4735e87ecdeda4798cf4a5a5016da7eade2cf9461be2912a70c4e3f1b411b258548425d25f50483ad10d09b4032862d26408993a78ce6d2e74b527a03523686b38fdfbd264cdacc7a29601b56de2ee7fc8aaf5ad8a8d3c2233eccade7ee8a255597feba1913a64086d952bb3f92995e9b203ab57b9676bfac9cae965609a2acfa1f23ef656147083bf0b25a8a3ce680ba7b9091422044d6ce37b686bf2f2bc14092a5d7bdd2c9a11991a29babddc867907c0636ca47a3afc3a43b54280e74366f6d0185b5c8f4f26f4e0a48a8815871e8b727bc10963cb67a22433b359fe0c2c91472602343b630c0cb84d46290f5476e1836a5174a89781a8199ba0b27914edbb9b85ddcba7db7bbb098dc2be54a96e80b5cdb6d41ee413abb4645b97c852cc8e269574f63ce4fe3e5a3066191e9e845ccd1\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Index Name": "string_encrypted_v3_100000_match_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 433.98
            }
          ],
          "Recheck Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbLE9E2f5{Z(7DX-`9~7F5*48R1ytR5Agx`ktr4Az!6uI~#kR6qwhp?$E>_AXr6YO@n@(5^dBRDN3J$dKi~^RA5Rx1Ea!IRTW-rrFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [1070, 1849, 192, 1524, 413, 1548, 1129, 1215, 1646, 0, 1574, 1705, 690, 684, 1265, 1030, 687, 1826, 104, 899, 1334, 566, 1342, 438, 872, 587, 953, 606, 1369, 948], \"hm\": \"fe348dd0003b5ce3173aaaaf495e30c6c4bce6201da8e9d6064d18598c818022\", \"ob\": [\"1e17dc69f8909090c00f2d549cf7c0571af2a3f32b85275b2bb943dde8b1a1df2f72ad9917386cd49096575761dc049e5ba711e2fdc2e3fdbfab357e00beb0fc5bc0dfced927aab195495bd31947d7aacf083e7046b8f503180caa4083372f1b0f4e9aed43d4735e87ecdeda4798cf4a5a5016da7eade2cf9461be2912a70c4e3f1b411b258548425d25f50483ad10d09b4032862d26408993a78ce6d2e74b527a03523686b38fdfbd264cdacc7a29601b56de2ee7fc8aaf5ad8a8d3c2233eccade7ee8a255597feba1913a64086d952bb3f92995e9b203ab57b9676bfac9cae965609a2acfa1f23ef656147083bf0b25a8a3ce680ba7b9091422044d6ce37b686bf2f2bc14092a5d7bdd2c9a11991a29babddc867907c0636ca47a3afc3a43b54280e74366f6d0185b5c8f4f26f4e0a48a8815871e8b727bc10963cb67a22433b359fe0c2c91472602343b630c0cb84d46290f5476e1836a5174a89781a8199ba0b27914edbb9b85ddcba7db7bbb098dc2be54a96e80b5cdb6d41ee413abb4645b97c852cc8e269574f63ce4fe3e5a3066191e9e845ccd1\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
          "Relation Name": "string_encrypted_v3_100000",
          "Startup Cost": 433.98,
          "Total Cost": 438.49
        }
      ],
      "Startup Cost": 433.98,
      "Total Cost": 438.49
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_v3_1000000
    Bitmap Index Scan using string_encrypted_v3_1000000_match_gin_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 1,
      "Plan Width": 640,
      "Plans": [
        {
          "Alias": "string_encrypted_v3_1000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 640,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbLtxJ<e!p@Uj!f-?Bf;>*y)8G7e;FlzQ)wP)Pb4ESN&ek;C_ScK0!jl9Gl%f}S5o@rpaMNn4jp{G`01PtKKvz);2c!lu5IuJhKrFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_1000000\"}, \"v\": 3, \"bf\": [690, 606, 587, 1334, 1705, 413, 1030, 684, 1548, 948, 438, 1826, 1342, 1215, 1129, 953, 192, 1524, 1849, 104, 1070, 0, 1369, 687, 1646, 1265, 566, 1574, 872, 899], \"hm\": \"fe348dd0003b5ce3173aaaaf495e30c6c4bce6201da8e9d6064d18598c818022\", \"ob\": [\"1e17dc69f8909090c00f2d549cf7c0571af2a3f32b85275b2bb943dde8b1a1df2f72ad9917386cd49096575761dc049e5ba711e2fdc2e3fdbfab357e00beb0fc5bc0dfced927aab195495bd31947d7aacf083e7046b8f503180caa4083372f1b0f4e9aed43d4735e87ecdeda4798cf4a5a5016da7eade2cf9461be2912a70c4e3f1b411b25854842c65933bde3c42f3b778f2860f3ee2b3856693c0590abb3b8cfcda80c8be05cda0d953cefe5a3ac465b05619de25abf07abf71eedcda1f5fbc5354d0329c6f5df52cb2234bdd1d05ee8c9614b8a547ab2a2c5867e5fd5e399282e89c94b18d3e3913fa6d34a2624465aa21400866d00c530f943d9d0e79dfdf881ddbed0bb36029b30d6328567424e0c0c41a5d232d876f3b6a378dd4142b14e61c1a1d05aff5b92d371f94d122d7a2983b1670dabaa31464903a1a3f5358f050bd8f9ff5e94ef5af93d01340a8ab9c2b64ecb52e723c1fbd0136a2046a7f9bbffaf874fa52a8deccf4e4d51e600e7ce895962fa22d879ab5fca19d966382792ffb1319513399c2f700a957e7fc964d33d8414151a889b\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Index Name": "string_encrypted_v3_1000000_match_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 1292.35
            }
          ],
          "Recheck Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbLtxJ<e!p@Uj!f-?Bf;>*y)8G7e;FlzQ)wP)Pb4ESN&ek;C_ScK0!jl9Gl%f}S5o@rpaMNn4jp{G`01PtKKvz);2c!lu5IuJhKrFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_1000000\"}, \"v\": 3, \"bf\": [690, 606, 587, 1334, 1705, 413, 1030, 684, 1548, 948, 438, 1826, 1342, 1215, 1129, 953, 192, 1524, 1849, 104, 1070, 0, 1369, 687, 1646, 1265, 566, 1574, 872, 899], \"hm\": \"fe348dd0003b5ce3173aaaaf495e30c6c4bce6201da8e9d6064d18598c818022\", \"ob\": [\"1e17dc69f8909090c00f2d549cf7c0571af2a3f32b85275b2bb943dde8b1a1df2f72ad9917386cd49096575761dc049e5ba711e2fdc2e3fdbfab357e00beb0fc5bc0dfced927aab195495bd31947d7aacf083e7046b8f503180caa4083372f1b0f4e9aed43d4735e87ecdeda4798cf4a5a5016da7eade2cf9461be2912a70c4e3f1b411b25854842c65933bde3c42f3b778f2860f3ee2b3856693c0590abb3b8cfcda80c8be05cda0d953cefe5a3ac465b05619de25abf07abf71eedcda1f5fbc5354d0329c6f5df52cb2234bdd1d05ee8c9614b8a547ab2a2c5867e5fd5e399282e89c94b18d3e3913fa6d34a2624465aa21400866d00c530f943d9d0e79dfdf881ddbed0bb36029b30d6328567424e0c0c41a5d232d876f3b6a378dd4142b14e61c1a1d05aff5b92d371f94d122d7a2983b1670dabaa31464903a1a3f5358f050bd8f9ff5e94ef5af93d01340a8ab9c2b64ecb52e723c1fbd0136a2046a7f9bbffaf874fa52a8deccf4e4d51e600e7ce895962fa22d879ab5fca19d966382792ffb1319513399c2f700a957e7fc964d33d8414151a889b\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
          "Relation Name": "string_encrypted_v3_1000000",
          "Startup Cost": 1292.35,
          "Total Cost": 1296.87
        }
      ],
      "Startup Cost": 1292.35,
      "Total Cost": 1296.87
    }
  }
]
```

</details>

![Query Performance - MATCH/eql_bloom](query_match_eql_bloom_chart.png)

## eql_bloom_noindex

**Description:** Unknown query

****

**Indexes available on the table:**
```sql
CREATE INDEX
string_encrypted_10000_hash_index
ON string_encrypted_10000 using hash (
    eql_v2.hmac_256(value)
);

CREATE INDEX
string_encrypted_10000_gin_index
ON string_encrypted_10000 USING GIN (
    eql_v2.bloom_filter(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 56.05ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Seq Scan on string_encrypted_v3_10000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 1,
      "Plan Width": 649,
      "Plans": [
        {
          "Alias": "string_encrypted_v3_10000",
          "Async Capable": false,
          "Filter": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbKO$}mRqZZ|NevEZ;XK%jfX8MVs4+C6@huRQge{TnN1VPB-yprvd^in_!glYA{e$AhWDXbT4y%lRhDBxacC_9<QOVyd-G(@L^1rFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [684, 690, 438, 0, 413, 1646, 1369, 1826, 1070, 1334, 1030, 872, 1342, 1849, 953, 1548, 1215, 948, 606, 104, 1705, 587, 566, 1574, 192, 687, 1265, 899, 1129, 1524], \"hm\": \"fe348dd0003b5ce3173aaaaf495e30c6c4bce6201da8e9d6064d18598c818022\", \"ob\": [\"1e17dc69f8909090c00f2d549cf7c0571af2a3f32b85275b2bb943dde8b1a1df2f72ad9917386cd49096575761dc049e5ba711e2fdc2e3fdbfab357e00beb0fc5bc0dfced927aab195495bd31947d7aacf083e7046b8f503180caa4083372f1b0f4e9aed43d4735e87ecdeda4798cf4a5a5016da7eade2cf9461be2912a70c4e3f1b411b258548429f9d5ebd329dfa3495d6afba74ba169a925dfe0e7fca8800f9448bc0bb671779bde2f6b63862910a57e584a233fcee8c1a6bf08dcefaec8191bf3aaca5c7ec33c5ccead3aa19ad847fb6adbfadf4be7acf5e89a82fce2255d93480a0b6776588f879438c8a7f4bc24fcf13f77228cf41cd007922ce02a3e1aa56babed118cd11efc5d53d8ddf7453768aedd4375ce3f3a77559c6c4acd89b9be57fe6d79e30c25ea60f119d1e2c908b250393d9f4c05c14f9487d25d835ee1113eca12e02cfa2231d15f9aa1f1de1a3464d64c24ce5ad03146323f1a1881ebfabc860a4eea6258ac3cd70ff2eb091f4866c89937eca076fc1045ffc0c0d0483bf5ad5b85731896e81f629532f335ea944efbf5c09692b\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 649,
          "Relation Name": "string_encrypted_v3_10000",
          "Startup Cost": 0.0,
          "Total Cost": 6033.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 6033.0
    }
  }
]
```

</details>

## eql_cast_firstname

**Description:** Pattern matching on first name using EQL cast and LIKE

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value LIKE $1 LIMIT 10
```

**Parameter:** `Bob`

**Table: `string_encrypted_{rows}` with encrypted string values. Index: MATCH index for substring searches. Query returns LIMIT 10 results.**

**Indexes available on the table:**
```sql
CREATE INDEX
string_encrypted_10000_hash_index
ON string_encrypted_10000 using hash (
    eql_v2.hmac_256(value)
);

CREATE INDEX
string_encrypted_10000_gin_index
ON string_encrypted_10000 USING GIN (
    eql_v2.bloom_filter(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `string_encrypted_v3_10000_match_gin_index`
- 100,000: `string_encrypted_v3_100000_match_gin_index`
- 1,000,000: `string_encrypted_v3_1000000_match_gin_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 4 | 200.13μs | 25.25ms |
| 100,000 | 10 | 647.12μs | 26.80ms |
| 1,000,000 | 10 | 3.30ms | 29.18ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_v3_10000
    Bitmap Index Scan using string_encrypted_v3_10000_match_gin_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 1,
      "Plan Width": 649,
      "Plans": [
        {
          "Alias": "string_encrypted_v3_10000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 649,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbJTA$yevNcQ<B%EL=XI}ioL6?j|yaoo|*PTXaAeW%(Pvunx2F?Ylus<OfX?Y!mT{W#1gGpANVd?$cjC#`3~-c!~6=-E+NrFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [1673, 943, 1471, 567, 1076, 1346], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c3c691c3fdaa841d296a811e13709d9896c632e135062f1a7b981023b57bd27045411e58c20df24ca63a0519c6bea8a93e60eb32174a364b437867acb4615032a624e2607293263572991a9a0666222658740ab7c213ec152e99da90426e94623473a4da1373e5ce9bb0b6a80bf5d67bf95d9d57caa4000a3eaf8c245bc637435c675177032e5a36c51c78d39fdca72b010351a6b68a52448c56118967744b6dffeccdfad39481478e0fe862de8789fed46d313c2daf64e2423f44b1d4df7f85c15a95bf47bf500fe1a0bb0e57dfc7e2b354849f8146d10476ed9486bc85d63976c43dcc716f7ae9e0b9001ce5bd1e542c77740739fb3cb528368634c9376fd833bf33597ece6466070cabd96d753ac052\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Index Name": "string_encrypted_v3_10000_match_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 56.47
            }
          ],
          "Recheck Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbJTA$yevNcQ<B%EL=XI}ioL6?j|yaoo|*PTXaAeW%(Pvunx2F?Ylus<OfX?Y!mT{W#1gGpANVd?$cjC#`3~-c!~6=-E+NrFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [1673, 943, 1471, 567, 1076, 1346], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c3c691c3fdaa841d296a811e13709d9896c632e135062f1a7b981023b57bd27045411e58c20df24ca63a0519c6bea8a93e60eb32174a364b437867acb4615032a624e2607293263572991a9a0666222658740ab7c213ec152e99da90426e94623473a4da1373e5ce9bb0b6a80bf5d67bf95d9d57caa4000a3eaf8c245bc637435c675177032e5a36c51c78d39fdca72b010351a6b68a52448c56118967744b6dffeccdfad39481478e0fe862de8789fed46d313c2daf64e2423f44b1d4df7f85c15a95bf47bf500fe1a0bb0e57dfc7e2b354849f8146d10476ed9486bc85d63976c43dcc716f7ae9e0b9001ce5bd1e542c77740739fb3cb528368634c9376fd833bf33597ece6466070cabd96d753ac052\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
          "Relation Name": "string_encrypted_v3_10000",
          "Startup Cost": 56.47,
          "Total Cost": 60.99
        }
      ],
      "Startup Cost": 56.47,
      "Total Cost": 60.99
    }
  }
]
```

**100,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_v3_100000
    Bitmap Index Scan using string_encrypted_v3_100000_match_gin_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 1,
      "Plan Width": 635,
      "Plans": [
        {
          "Alias": "string_encrypted_v3_100000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 635,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbMLz~2cP@FT7`S21vuMQ&ci6~9jRjYwz<^4*v?e~*vfjs2_eCcMNTm06{}Y6Vr$M4t10C_aUbyg1XU@>B70@|IMkxe?&2rFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [943, 1673, 567, 1471, 1076, 1346], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c3f7e710301cc2123f394b5a797d230f19b14fa894e16e83ba142eed1580604dc4f69d05fef92aaa557928b42521f7c121713cf9069247481ae08964366ef19cf904d7298f654797ea280b537deb0159dac5ad6718512832a370a1c20056853536f439a2b65fa9d93e40ca4cc7152ec617ee4ddbe089d6d793518754b9e5c73478a0d352acd77f4f7cb9aef687c39f53f6aa3b180c2ea1757374de938fdcd049dc443a85c17297e86b05ae82c74370dfcadb20a6ccaf01986ae0233d5a0dd5e7aa8529719732c1ce4abbf7d58183dd08da09b75ed888e5e3ca3d340f1ace42aebb097530d81c7fec23fc5bdf567ab79136a576e8ff55ab8dc49d89dc3dbda6bb7b47102ed97ae5a0cabb24342ec168ceb4\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Index Name": "string_encrypted_v3_100000_match_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 89.47
            }
          ],
          "Recheck Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbMLz~2cP@FT7`S21vuMQ&ci6~9jRjYwz<^4*v?e~*vfjs2_eCcMNTm06{}Y6Vr$M4t10C_aUbyg1XU@>B70@|IMkxe?&2rFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [943, 1673, 567, 1471, 1076, 1346], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c3f7e710301cc2123f394b5a797d230f19b14fa894e16e83ba142eed1580604dc4f69d05fef92aaa557928b42521f7c121713cf9069247481ae08964366ef19cf904d7298f654797ea280b537deb0159dac5ad6718512832a370a1c20056853536f439a2b65fa9d93e40ca4cc7152ec617ee4ddbe089d6d793518754b9e5c73478a0d352acd77f4f7cb9aef687c39f53f6aa3b180c2ea1757374de938fdcd049dc443a85c17297e86b05ae82c74370dfcadb20a6ccaf01986ae0233d5a0dd5e7aa8529719732c1ce4abbf7d58183dd08da09b75ed888e5e3ca3d340f1ace42aebb097530d81c7fec23fc5bdf567ab79136a576e8ff55ab8dc49d89dc3dbda6bb7b47102ed97ae5a0cabb24342ec168ceb4\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
          "Relation Name": "string_encrypted_v3_100000",
          "Startup Cost": 89.47,
          "Total Cost": 93.99
        }
      ],
      "Startup Cost": 89.47,
      "Total Cost": 93.99
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_v3_1000000
    Bitmap Index Scan using string_encrypted_v3_1000000_match_gin_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 1,
      "Plan Width": 640,
      "Plans": [
        {
          "Alias": "string_encrypted_v3_1000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 640,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbJ_X{{~BP#>&#D7Y>RXZF*?6>C%&G83QGc|dYrvBqP`>Ll5pn|H(@aXk=W91Dul)AD0BwsiOFUeqs<(o@qL45|qKKu{3mrFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_1000000\"}, \"v\": 3, \"bf\": [943, 1346, 567, 1076, 1673, 1471], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c3fb8f016716f26e806861b7071fd9c926ce8a3e42eb3d05a89c961ad1772c49e4e0d9810214e0f2fe310f9a6272bd7820f19eaaa81c1a1dcf26cf4d39ff0ccecde5529af08ab32b9331bf73d6863925d22a9167c3b8caab39535cfc4a464f8af035fcd0d2b1c25e1cfe3dc9e827377827b2761b5040584dc3958d91a216dc8c0d8514c41f9d8c4883bc6bebafe6fd1c3e9b527e1a9addf613efa533d4bfc8c6d8744d0e482f666160ce3941d8f71f3c96385e64aa9f5c090fb1be3109e4d74f8342de520daef6f6c0802d3b967ac06c8bff709b8b88567ee05356119c79da4d604d54e79811467c90673bfbf6e3c12de068527eb972a56c57da8388224354a776ad0f0050e045662efa04dc4b312ad90e\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Index Name": "string_encrypted_v3_1000000_match_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 261.97
            }
          ],
          "Recheck Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbJ_X{{~BP#>&#D7Y>RXZF*?6>C%&G83QGc|dYrvBqP`>Ll5pn|H(@aXk=W91Dul)AD0BwsiOFUeqs<(o@qL45|qKKu{3mrFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_1000000\"}, \"v\": 3, \"bf\": [943, 1346, 567, 1076, 1673, 1471], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c3fb8f016716f26e806861b7071fd9c926ce8a3e42eb3d05a89c961ad1772c49e4e0d9810214e0f2fe310f9a6272bd7820f19eaaa81c1a1dcf26cf4d39ff0ccecde5529af08ab32b9331bf73d6863925d22a9167c3b8caab39535cfc4a464f8af035fcd0d2b1c25e1cfe3dc9e827377827b2761b5040584dc3958d91a216dc8c0d8514c41f9d8c4883bc6bebafe6fd1c3e9b527e1a9addf613efa533d4bfc8c6d8744d0e482f666160ce3941d8f71f3c96385e64aa9f5c090fb1be3109e4d74f8342de520daef6f6c0802d3b967ac06c8bff709b8b88567ee05356119c79da4d604d54e79811467c90673bfbf6e3c12de068527eb972a56c57da8388224354a776ad0f0050e045662efa04dc4b312ad90e\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
          "Relation Name": "string_encrypted_v3_1000000",
          "Startup Cost": 261.97,
          "Total Cost": 266.49
        }
      ],
      "Startup Cost": 261.97,
      "Total Cost": 266.49
    }
  }
]
```

</details>

![Query Performance - MATCH/eql_cast_firstname](query_match_eql_cast_firstname_chart.png)

## eql_cast_firstname_noindex

**Description:** Unknown query

****

**Indexes available on the table:**
```sql
CREATE INDEX
string_encrypted_10000_hash_index
ON string_encrypted_10000 using hash (
    eql_v2.hmac_256(value)
);

CREATE INDEX
string_encrypted_10000_gin_index
ON string_encrypted_10000 USING GIN (
    eql_v2.bloom_filter(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_

*⚠️ = Query time exceeds 100ms*

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 4 | ⚠️ 192.11ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Seq Scan on string_encrypted_v3_10000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 1,
      "Plan Width": 649,
      "Plans": [
        {
          "Alias": "string_encrypted_v3_10000",
          "Async Capable": false,
          "Filter": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbJTA$yevNcQ<B%EL=XI}ioL6?j|yaoo|*PTXaAeW%(Pvunx2F?Ylus<OfX?Y!mT{W#1gGpANVd?$cjC#`3~-c!~6=-E+NrFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [1673, 943, 1471, 567, 1076, 1346], \"hm\": \"d22ac625c9c1daba9b8150da007d2b5439805886ec39577fd9ef95ffe4f86795\", \"ob\": [\"f8bc8c8c8c8c8c8c148eaac3a25aee669623505bb82c16eb29a7175a9ef4e2a737a2773bf9618cb2cd9052275e9f85cd524150b3812138804ed85eb01498ba3069836158f28bed37ea70b957eba0dedb7716b5ff06e87dd519cb5b26536c0999ca90b109bda9b2e96f77317232133257105cdbf358dc8b95db957a0530f37ae1bc4014d6d80f09c3c691c3fdaa841d296a811e13709d9896c632e135062f1a7b981023b57bd27045411e58c20df24ca63a0519c6bea8a93e60eb32174a364b437867acb4615032a624e2607293263572991a9a0666222658740ab7c213ec152e99da90426e94623473a4da1373e5ce9bb0b6a80bf5d67bf95d9d57caa4000a3eaf8c245bc637435c675177032e5a36c51c78d39fdca72b010351a6b68a52448c56118967744b6dffeccdfad39481478e0fe862de8789fed46d313c2daf64e2423f44b1d4df7f85c15a95bf47bf500fe1a0bb0e57dfc7e2b354849f8146d10476ed9486bc85d63976c43dcc716f7ae9e0b9001ce5bd1e542c77740739fb3cb528368634c9376fd833bf33597ece6466070cabd96d753ac052\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 649,
          "Relation Name": "string_encrypted_v3_10000",
          "Startup Cost": 0.0,
          "Total Cost": 6033.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 6033.0
    }
  }
]
```

</details>

## eql_cast_lastname

**Description:** Pattern matching on last name using EQL cast and LIKE

**SQL Query:**
```sql
SELECT id,value::jsonb FROM {TABLE} WHERE value LIKE $1 LIMIT 10
```

**Parameter:** `Johnson`

**Table: `string_encrypted_{rows}` with encrypted string values. Index: MATCH index for substring searches. Query returns LIMIT 10 results.**

**Indexes available on the table:**
```sql
CREATE INDEX
string_encrypted_10000_hash_index
ON string_encrypted_10000 using hash (
    eql_v2.hmac_256(value)
);

CREATE INDEX
string_encrypted_10000_gin_index
ON string_encrypted_10000 USING GIN (
    eql_v2.bloom_filter(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: `string_encrypted_v3_10000_match_gin_index`
- 100,000: `string_encrypted_v3_100000_match_gin_index`
- 1,000,000: `string_encrypted_v3_1000000_match_gin_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 487.55μs | 26.81ms |
| 100,000 | 10 | 2.03ms | 27.45ms |
| 1,000,000 | 10 | 15.86ms | 38.34ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_v3_10000
    Bitmap Index Scan using string_encrypted_v3_10000_match_gin_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 1,
      "Plan Width": 649,
      "Plans": [
        {
          "Alias": "string_encrypted_v3_10000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 649,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbLm<XRA&_gM)DM*j{xIME-(8BZF95u~IwY>bP*Dp=_i5>&DLujoTe)n>#XM_DTi2zPY(IuI8ojU3bq=tr&BKflTutN0nLYesKsrFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [1129, 1524, 1342, 690, 953, 1548, 1334, 606, 872, 684, 1369, 1070, 687, 104, 587, 1826, 948, 1215, 438, 1705, 1849, 899, 1030, 566, 1265, 0, 1646, 1574, 413, 192], \"hm\": \"fe348dd0003b5ce3173aaaaf495e30c6c4bce6201da8e9d6064d18598c818022\", \"ob\": [\"1e17dc69f8909090c00f2d549cf7c0571af2a3f32b85275b2bb943dde8b1a1df2f72ad9917386cd49096575761dc049e5ba711e2fdc2e3fdbfab357e00beb0fc5bc0dfced927aab195495bd31947d7aacf083e7046b8f503180caa4083372f1b0f4e9aed43d4735e87ecdeda4798cf4a5a5016da7eade2cf9461be2912a70c4e3f1b411b25854842664a283bb0e244d7eaf47e37ab90bcf840e704dad7c48006d0c6db212934a79648791b7be3e8bf0d5ffbd5f00910adc1330107fa5a3aa980a6ca61026a881771802a5a987dac4e8d270d95d4e9aeaf4ab206c1422324cab69324f0c7e33a33adae94577b2d9ed0e7e243bdcc7aad7f8b49ae957bb8f4ab2935d64defe6584c29e249a55a09b418b8d279af27628e65ed992ac31b5f6313a5742a1532eccd0d2314991bba64e64f68d34217d6829cc422c3abd788acbb4483487dac1c1df53e078939e93bb7a3a31159c0d47fb899cffc0509663460da2670e3d1958a21df00ff9bc9947e8e31491db4ae7c67c9e051a14604842097df6d4f59e031443844033847bea1200092d9785ff2d8f4a9754f09\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Index Name": "string_encrypted_v3_10000_match_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 264.85
            }
          ],
          "Recheck Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbLm<XRA&_gM)DM*j{xIME-(8BZF95u~IwY>bP*Dp=_i5>&DLujoTe)n>#XM_DTi2zPY(IuI8ojU3bq=tr&BKflTutN0nLYesKsrFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [1129, 1524, 1342, 690, 953, 1548, 1334, 606, 872, 684, 1369, 1070, 687, 104, 587, 1826, 948, 1215, 438, 1705, 1849, 899, 1030, 566, 1265, 0, 1646, 1574, 413, 192], \"hm\": \"fe348dd0003b5ce3173aaaaf495e30c6c4bce6201da8e9d6064d18598c818022\", \"ob\": [\"1e17dc69f8909090c00f2d549cf7c0571af2a3f32b85275b2bb943dde8b1a1df2f72ad9917386cd49096575761dc049e5ba711e2fdc2e3fdbfab357e00beb0fc5bc0dfced927aab195495bd31947d7aacf083e7046b8f503180caa4083372f1b0f4e9aed43d4735e87ecdeda4798cf4a5a5016da7eade2cf9461be2912a70c4e3f1b411b25854842664a283bb0e244d7eaf47e37ab90bcf840e704dad7c48006d0c6db212934a79648791b7be3e8bf0d5ffbd5f00910adc1330107fa5a3aa980a6ca61026a881771802a5a987dac4e8d270d95d4e9aeaf4ab206c1422324cab69324f0c7e33a33adae94577b2d9ed0e7e243bdcc7aad7f8b49ae957bb8f4ab2935d64defe6584c29e249a55a09b418b8d279af27628e65ed992ac31b5f6313a5742a1532eccd0d2314991bba64e64f68d34217d6829cc422c3abd788acbb4483487dac1c1df53e078939e93bb7a3a31159c0d47fb899cffc0509663460da2670e3d1958a21df00ff9bc9947e8e31491db4ae7c67c9e051a14604842097df6d4f59e031443844033847bea1200092d9785ff2d8f4a9754f09\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
          "Relation Name": "string_encrypted_v3_10000",
          "Startup Cost": 264.85,
          "Total Cost": 269.37
        }
      ],
      "Startup Cost": 264.85,
      "Total Cost": 269.37
    }
  }
]
```

**100,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_v3_100000
    Bitmap Index Scan using string_encrypted_v3_100000_match_gin_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 1,
      "Plan Width": 635,
      "Plans": [
        {
          "Alias": "string_encrypted_v3_100000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 635,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbK3c9YZ~YabiHj6ChG6PoPA86?R^#FT36G&YMDin={;E^5p$n<gf2dq2b=0x5??BwX9;?u}}c=m=?%>{Rny;MEEKCbRP=$(U^*rFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [684, 413, 1265, 1070, 1342, 606, 1574, 1215, 192, 438, 1030, 1826, 899, 1548, 1849, 872, 690, 0, 687, 1646, 1369, 566, 953, 1524, 1129, 104, 1705, 1334, 587, 948], \"hm\": \"fe348dd0003b5ce3173aaaaf495e30c6c4bce6201da8e9d6064d18598c818022\", \"ob\": [\"1e17dc69f8909090c00f2d549cf7c0571af2a3f32b85275b2bb943dde8b1a1df2f72ad9917386cd49096575761dc049e5ba711e2fdc2e3fdbfab357e00beb0fc5bc0dfced927aab195495bd31947d7aacf083e7046b8f503180caa4083372f1b0f4e9aed43d4735e87ecdeda4798cf4a5a5016da7eade2cf9461be2912a70c4e3f1b411b25854842f9658f24a34f1f3b411b4c38ef6c0f94c3d11e693437ad3cff97cffb259176a4bf250f49e36db67351c31d8ffd7e0f2b5908377205ab3cb27ac37b61a75ec5a344d5ee353c4a2660c868116ac947d393383a6195a29f3253202aac00cf998c8cca533091e94842db6d52afd2438344b5711b6257308b8a86e76b71eae4a7c91b6600d1f557968cade7977106edbd053e074dca9e2ad3839e63573568359949918adf3f1624a08eb00865f093bd5a864bad0c303bff9d929a4b91759083b9908bbae56bb309eda8c8853e60929e23bdd30319c8ab8ff418ea259b5e9f4f7cee43e8c74b6106286f9787aad1490d1fe3fedb1507ae144bb46e502d8e85031c9273adbe36fe7a06f5424eb1e9166fe3b6e6\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Index Name": "string_encrypted_v3_100000_match_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 433.98
            }
          ],
          "Recheck Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbK3c9YZ~YabiHj6ChG6PoPA86?R^#FT36G&YMDin={;E^5p$n<gf2dq2b=0x5??BwX9;?u}}c=m=?%>{Rny;MEEKCbRP=$(U^*rFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [684, 413, 1265, 1070, 1342, 606, 1574, 1215, 192, 438, 1030, 1826, 899, 1548, 1849, 872, 690, 0, 687, 1646, 1369, 566, 953, 1524, 1129, 104, 1705, 1334, 587, 948], \"hm\": \"fe348dd0003b5ce3173aaaaf495e30c6c4bce6201da8e9d6064d18598c818022\", \"ob\": [\"1e17dc69f8909090c00f2d549cf7c0571af2a3f32b85275b2bb943dde8b1a1df2f72ad9917386cd49096575761dc049e5ba711e2fdc2e3fdbfab357e00beb0fc5bc0dfced927aab195495bd31947d7aacf083e7046b8f503180caa4083372f1b0f4e9aed43d4735e87ecdeda4798cf4a5a5016da7eade2cf9461be2912a70c4e3f1b411b25854842f9658f24a34f1f3b411b4c38ef6c0f94c3d11e693437ad3cff97cffb259176a4bf250f49e36db67351c31d8ffd7e0f2b5908377205ab3cb27ac37b61a75ec5a344d5ee353c4a2660c868116ac947d393383a6195a29f3253202aac00cf998c8cca533091e94842db6d52afd2438344b5711b6257308b8a86e76b71eae4a7c91b6600d1f557968cade7977106edbd053e074dca9e2ad3839e63573568359949918adf3f1624a08eb00865f093bd5a864bad0c303bff9d929a4b91759083b9908bbae56bb309eda8c8853e60929e23bdd30319c8ab8ff418ea259b5e9f4f7cee43e8c74b6106286f9787aad1490d1fe3fedb1507ae144bb46e502d8e85031c9273adbe36fe7a06f5424eb1e9166fe3b6e6\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
          "Relation Name": "string_encrypted_v3_100000",
          "Startup Cost": 433.98,
          "Total Cost": 438.49
        }
      ],
      "Startup Cost": 433.98,
      "Total Cost": 438.49
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Bitmap Heap Scan on string_encrypted_v3_1000000
    Bitmap Index Scan using string_encrypted_v3_1000000_match_gin_index
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 1,
      "Plan Width": 640,
      "Plans": [
        {
          "Alias": "string_encrypted_v3_1000000",
          "Async Capable": false,
          "Node Type": "Bitmap Heap Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 640,
          "Plans": [
            {
              "Async Capable": false,
              "Index Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbJprUR4NIxndC;^h8FmqQK286&V4b<t~rvB8^rcn(I9;AvVt^7&7)mhQwLR^djs(txx#iU4qT#)1i~2lw+6+`KM|F@`%45Ha!5rFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_1000000\"}, \"v\": 3, \"bf\": [1646, 1070, 1826, 1030, 104, 192, 684, 1129, 413, 438, 587, 1215, 948, 0, 1524, 899, 1369, 1705, 1334, 690, 1265, 1342, 566, 687, 1548, 953, 1849, 1574, 872, 606], \"hm\": \"fe348dd0003b5ce3173aaaaf495e30c6c4bce6201da8e9d6064d18598c818022\", \"ob\": [\"1e17dc69f8909090c00f2d549cf7c0571af2a3f32b85275b2bb943dde8b1a1df2f72ad9917386cd49096575761dc049e5ba711e2fdc2e3fdbfab357e00beb0fc5bc0dfced927aab195495bd31947d7aacf083e7046b8f503180caa4083372f1b0f4e9aed43d4735e87ecdeda4798cf4a5a5016da7eade2cf9461be2912a70c4e3f1b411b25854842bd0f2113bda21269145210b2e3e010d0f2f4385e580ea84e87b337fc73791ab7f33d8dbb71c3338f6f4b311b410be70eaa5ec45fbca3db1286160372e3ea7392e10432e8aa8556b7fe5afe4a039dd918c66b311cd3f197ed817f3882e55259e03f8a44364353ba8bad1dead0d5f279186935399effc4ac618585a78f2e77221dd6c4df52188a71d0d105f2349c0e05f5656aa0b9071f09caafa0055bb9e9918291fc1d5aea649aa5bed6ba50defac3b9b949687a2e37725963b111638ee7e3e5ff92704b372b3349af5276bb30afa53b727a2a4df60b92113dd435b2de159038e58157f010446e342bcba2b1009f21622c71d7de548bdcbaebe12a864358afd381a1ecd1e9465016ad222a1d22bc5fec\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
              "Index Name": "string_encrypted_v3_1000000_match_gin_index",
              "Node Type": "Bitmap Index Scan",
              "Parallel Aware": false,
              "Parent Relationship": "Outer",
              "Plan Rows": 1,
              "Plan Width": 0,
              "Startup Cost": 0.0,
              "Total Cost": 1292.35
            }
          ],
          "Recheck Cond": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbJprUR4NIxndC;^h8FmqQK286&V4b<t~rvB8^rcn(I9;AvVt^7&7)mhQwLR^djs(txx#iU4qT#)1i~2lw+6+`KM|F@`%45Ha!5rFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_1000000\"}, \"v\": 3, \"bf\": [1646, 1070, 1826, 1030, 104, 192, 684, 1129, 413, 438, 587, 1215, 948, 0, 1524, 899, 1369, 1705, 1334, 690, 1265, 1342, 566, 687, 1548, 953, 1849, 1574, 872, 606], \"hm\": \"fe348dd0003b5ce3173aaaaf495e30c6c4bce6201da8e9d6064d18598c818022\", \"ob\": [\"1e17dc69f8909090c00f2d549cf7c0571af2a3f32b85275b2bb943dde8b1a1df2f72ad9917386cd49096575761dc049e5ba711e2fdc2e3fdbfab357e00beb0fc5bc0dfced927aab195495bd31947d7aacf083e7046b8f503180caa4083372f1b0f4e9aed43d4735e87ecdeda4798cf4a5a5016da7eade2cf9461be2912a70c4e3f1b411b25854842bd0f2113bda21269145210b2e3e010d0f2f4385e580ea84e87b337fc73791ab7f33d8dbb71c3338f6f4b311b410be70eaa5ec45fbca3db1286160372e3ea7392e10432e8aa8556b7fe5afe4a039dd918c66b311cd3f197ed817f3882e55259e03f8a44364353ba8bad1dead0d5f279186935399effc4ac618585a78f2e77221dd6c4df52188a71d0d105f2349c0e05f5656aa0b9071f09caafa0055bb9e9918291fc1d5aea649aa5bed6ba50defac3b9b949687a2e37725963b111638ee7e3e5ff92704b372b3349af5276bb30afa53b727a2a4df60b92113dd435b2de159038e58157f010446e342bcba2b1009f21622c71d7de548bdcbaebe12a864358afd381a1ecd1e9465016ad222a1d22bc5fec\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
          "Relation Name": "string_encrypted_v3_1000000",
          "Startup Cost": 1292.35,
          "Total Cost": 1296.87
        }
      ],
      "Startup Cost": 1292.35,
      "Total Cost": 1296.87
    }
  }
]
```

</details>

![Query Performance - MATCH/eql_cast_lastname](query_match_eql_cast_lastname_chart.png)

## eql_cast_lastname_noindex

**Description:** Unknown query

****

**Indexes available on the table:**
```sql
CREATE INDEX
string_encrypted_10000_hash_index
ON string_encrypted_10000 using hash (
    eql_v2.hmac_256(value)
);

CREATE INDEX
string_encrypted_10000_gin_index
ON string_encrypted_10000 USING GIN (
    eql_v2.bloom_filter(value)
);
```

**Indexes used by the planner (per data set size):**

- 10,000: _none — planner picked a sequential / hash-aggregate / sort plan_

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 10 | 58.69ms | N/A |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Seq Scan on string_encrypted_v3_10000
```

Full `EXPLAIN (FORMAT JSON)`:

```json
[
  {
    "Plan": {
      "Async Capable": false,
      "Node Type": "Limit",
      "Parallel Aware": false,
      "Plan Rows": 1,
      "Plan Width": 649,
      "Plans": [
        {
          "Alias": "string_encrypted_v3_10000",
          "Async Capable": false,
          "Filter": "((eql_v3_internal.bloom_filter((value)::jsonb))::smallint[] @> (eql_v3_internal.bloom_filter((('{\"c\": \"mBbLm<XRA&_gM)DM*j{xIME-(8BZF95u~IwY>bP*Dp=_i5>&DLujoTe)n>#XM_DTi2zPY(IuI8ojU3bq=tr&BKflTutN0nLYesKsrFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [1129, 1524, 1342, 690, 953, 1548, 1334, 606, 872, 684, 1369, 1070, 687, 104, 587, 1826, 948, 1215, 438, 1705, 1849, 899, 1030, 566, 1265, 0, 1646, 1574, 413, 192], \"hm\": \"fe348dd0003b5ce3173aaaaf495e30c6c4bce6201da8e9d6064d18598c818022\", \"ob\": [\"1e17dc69f8909090c00f2d549cf7c0571af2a3f32b85275b2bb943dde8b1a1df2f72ad9917386cd49096575761dc049e5ba711e2fdc2e3fdbfab357e00beb0fc5bc0dfced927aab195495bd31947d7aacf083e7046b8f503180caa4083372f1b0f4e9aed43d4735e87ecdeda4798cf4a5a5016da7eade2cf9461be2912a70c4e3f1b411b25854842664a283bb0e244d7eaf47e37ab90bcf840e704dad7c48006d0c6db212934a79648791b7be3e8bf0d5ffbd5f00910adc1330107fa5a3aa980a6ca61026a881771802a5a987dac4e8d270d95d4e9aeaf4ab206c1422324cab69324f0c7e33a33adae94577b2d9ed0e7e243bdcc7aad7f8b49ae957bb8f4ab2935d64defe6584c29e249a55a09b418b8d279af27628e65ed992ac31b5f6313a5742a1532eccd0d2314991bba64e64f68d34217d6829cc422c3abd788acbb4483487dac1c1df53e078939e93bb7a3a31159c0d47fb899cffc0509663460da2670e3d1958a21df00ff9bc9947e8e31491db4ae7c67c9e051a14604842097df6d4f59e031443844033847bea1200092d9785ff2d8f4a9754f09\"]}'::jsonb)::eql_v3.text_search)::jsonb))::smallint[])",
          "Node Type": "Seq Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 649,
          "Relation Name": "string_encrypted_v3_10000",
          "Startup Cost": 0.0,
          "Total Cost": 6033.0
        }
      ],
      "Startup Cost": 0.0,
      "Total Cost": 6033.0
    }
  }
]
```

</details>

