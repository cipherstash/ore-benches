# EXACT Queries

[← Back to overview](./BENCHMARK_REPORT.md)

Per-tier query performance. Each scenario lists its SQL, the indexes available on the target table, the indexes the planner actually picked per tier, the timing table, and the full EXPLAIN plan in a collapsed block.

## eql_cast

**Description:** Exact match using EQL cast operator

**SQL Query:**
```sql
SELECT value FROM {TABLE} WHERE value = $1 LIMIT 1
```

**Parameter:** `Bob Johnson`

**Table: `string_encrypted_{rows}` with encrypted string values. Index: UNIQUE index on the encrypted value column.**

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

- 10,000: `string_encrypted_v3_10000_eq_btree_index`
- 100,000: `string_encrypted_v3_100000_eq_btree_index`
- 1,000,000: `string_encrypted_v3_1000000_eq_btree_index`
- 10,000,000: `string_encrypted_v3_10000000_eq_btree_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 1 | 125.98μs | 24.58ms |
| 100,000 | 1 | 120.73μs | 24.54ms |
| 1,000,000 | 1 | 123.30μs | 24.26ms |
| 10,000,000 | 1 | 140.68μs | 24.79ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using string_encrypted_v3_10000_eq_btree_index on string_encrypted_v3_10000
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
          "Index Cond": "(((value)::jsonb ->> 'hm'::text) = ((('{\"c\": \"mBbJ@pO<31RWq<2Vo1gF#%gQCBETb!kn=gu2|2J}0jLelmc+1B&xl*WJMGZJ&??7QhzJM7AR)R^oa~Ft;Gcf*^=SZ8Ap{Y9ZJUzWuXn&w#ikBUE~R#1Y;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [6, 870, 1786, 1726, 2009, 927, 1853, 551, 1662, 1592, 529, 300, 1710, 488, 167, 621, 485, 19, 88, 1112, 528, 558, 1154, 687, 966, 1603, 1454, 1455, 1268, 393, 569, 309, 463, 1197, 238, 1970, 193, 843, 1824, 1766, 964, 1756, 981, 1325, 1414, 578, 1108, 2012, 296, 1953, 373, 1221, 1675, 1161, 1563, 857, 1854, 1470, 1582, 1911, 884, 1545, 697, 1637, 1066, 38, 1912, 164, 1412, 365, 1651, 207, 17, 277, 62, 699, 761, 923, 1772, 712, 385, 1370, 1925], \"hm\": \"b0b055247cded917baabf9b0200b89d76d32981c1ed299fd2210b3217f5baf57\", \"ob\": [\"c5c5aa2ca88a380754bc68f51c80961f466ccfbf630a65e114447e0fc086de94d65e963662935ec00983ec4e8ac643ce31599f418d5df32d71b4d66e9892c679f909bd89cfe0a01152cbe053126d527a31615775bd8c15940a1850271dacff08bcfb2463b6e4e0e2c1f5d65e2778db270453134bad4d46314f0ce95e8a271753b7b6efa372c122690b9a0f0f3031fbfda479b57c8e5ac902b7711944809b948182f305eaa132dfc25cdb32b699c5fb1dcc607b6bdcec05e65f0f114fe2c26f470609e79c78606ae0edf49443476643a98bb90dc97b833748fd6fc3b5b8317afaeac1ce8bcaca9a2d5dec9df1e25ae856b367a21d056325e85534d89a8a832ae885801e152af64b7f7f73eaaf1f49a2f8be3a7fe210ba811d97ab5e00374f1e567f7c078b37d4e443731e3f8d84a8fe4caeca9267d2e812da5ef3ffe41a7a30ae505d228ea7dcc54e10eceee3a88bf852737c588dbccd9d372054da2caa8038a1e311db50b4f7eeb03b7d3aef8fd3c89f4e21caf21a22fe67e2f42122b0a0b847d30dedf0d65fe828d2744c046ce04505a5c7084226e42f73\", \"951757575757575733f0468c7d82d838dbe077939798e6a3ad43556be5c5ed05ea70a2e354ee8b6f3d0566d065838731e54792d7fc5828ccde3e800c818bc246243c799bbe9f15d5b0b8ef862c5b8e740435aabc91b2babea3dc1f93e9828651582e0b25dd28a2f7e8522b56556151b6ce7ef45ab2fb89af4e88ba5c55eff0f1d8137b92e7b94f88cdf7b76e477a88d906c2c1567ec0c8d7dfa51aec655a24daf78a819637261a1ee677f8cf19140db4143dd4584466db2ce3a856e4341dd24302f7783e9e727d3aede3adddf4df01d81ec4a7393ad26f6fbbbcf5b285058c0692e45cac1f292875afac0da65b33d5e9350561da510541f31ed2da8ecdc429ba91db0deb1f8bf6b519e8433a033e07d0d6c268f57f99f1810357fda0bc8e7dbccf2169e87e15bb2386cf1186f712871413faa946ae6e790e6844dafe73cddb50787d0044db7c7a935004e9c6459640c8642036773e57ae198a70211077dbd3541f8638a7daac3082c479c917ea23a7aabf069a821d04ed213e28fe3f0268b08bfbd525974ca8b403a6d7aa2667a5bd409e2a710d7f9f64a8\"]}'::jsonb)::eql_v3.text_search)::jsonb ->> 'hm'::text))",
          "Index Name": "string_encrypted_v3_10000_eq_btree_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 649,
          "Relation Name": "string_encrypted_v3_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.41,
          "Total Cost": 8.43
        }
      ],
      "Startup Cost": 0.41,
      "Total Cost": 8.43
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using string_encrypted_v3_100000_eq_btree_index on string_encrypted_v3_100000
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
          "Index Cond": "(((value)::jsonb ->> 'hm'::text) = ((('{\"c\": \"mBbLN+r8g4YgcBdu+5a_Lnqe6A#q049S2ijJ3B5-T>G%kqL=V<F5H9rvZ>?kAf;R~XSl>5Z(?Nrmp-cHXD2slwK}s02`W`yy5>p<#pW>YmsLmarFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [1350, 141, 1697, 1485, 1453, 1805, 286, 1531, 1254, 204, 1615, 748, 36, 763, 461, 1589, 1885, 1315, 588, 607, 827, 300, 925, 1504, 1373, 1090, 159, 1309, 130, 586, 351, 725, 792, 1672, 1920, 1841, 1765, 1247, 1641, 1592, 438, 1405, 556, 426, 1081, 951, 147, 233, 251, 1333, 1341, 632, 808, 1230, 1699, 804, 410, 1422, 1556, 2021, 1593, 1818, 1355, 1590, 1969, 1994, 1541, 1400, 1826, 238, 1537, 677, 1075, 637, 18, 1813], \"hm\": \"28369ac87a26801c88729f107b6a0a59ec8e7df2bc199971e57de95fa4d9a205\", \"ob\": [\"7d5be628a8a1a5d8550934ce6650d06a3ba0db032e9677c59070ec3b61f5e1410f873fc57f0455f8e4866ec8d044b7e2e5daaba310d45ce075dcc639715a04a7650f34f2a945608192d1081cac077659b02af00c5dc97d17fe033eb5a150c0488ba090f61c0dfa884edc8309a5d8399e0ed283202f0dc595b8d11df49b74c3d8707682824faac4532c4350dac5e334756a28633b25f871eaedd9e6298077ff8a04e3807cd8a5afe75066d1c1b9e8d6361468d1889ca18e6908eccea859a7c9b7e0aad78ed923bdd09cbdb1e50fe7ea83bea9db1be6c8f336bd9918fc50aa31b09a6c5b67125861e7ac0cd264d20edc1cc77610fea5f26df36e51737bb91aeb1f4803d8b382bfef85e478674a02ccd30c470817f0c3ab402648081618a829eb8b3b596d2940f9ad1eee3f748b297329c1f1f84a873c3ef9e0f86d866c7b289d590fb911b56fa86b54258ef21fdcbd53b07372c0a143d06341154f9a1b26055f50228d20f92b3ce79c3682d5e5dbe3d5b329bd090814c0f24bb53d09f5da41bd7acb2f825d6a7a846cc6d2f66839591cd94b9feb39252e3d61\", \"e499141414141414c02a268347ec03abbc18838e87c562b0389adeed563dcfe75dff440e298879f077147f22e6d37d5311b9f9ddde45e5f6febcb10067ea8dbb560ddf7e01037c2e470f95abe04b8ed9cfeb049b4c5b7b29ec51eee9c826b3d52f7274074e6e7be64c11ee74e2c591e076be277277d339108032cfeaef2d68b0a7e7b295526b03ca5a1f0e70d281bb2fb6cda637321efde2676078500edc8541a6d4b3274ebd156093d1fad5d996315ee9f7a372e9e9fd2dd8ccd95a5e8e780310f95f835c3af1924c32ff0620a59e36ac160f1a4f0f43d578da4259953b1586808874470ebd113fec7f150bd1c53dd906fd3e0a4552e91059d9f6d4e2cd7fa4ff56c8688f64b66200093c3d2b2f5ca3c2cd7b03ee723fc9abb9d323bf8d8146e89c77123f44cf5774dde4f822913b94e542bfde60c8cce42c89464574b9f13ca609de7f9ba81060d4b9439f74629fdb04fc092a0c0ec9f5e65014f7f9a61ef78f81491edc74088a71f956f8a7b9a1242cc45bc47c22a5d7e14501baa29e6e67ccb9e0b86cc10693313739d6f0465a4ee4808d1b5163cede\"]}'::jsonb)::eql_v3.text_search)::jsonb ->> 'hm'::text))",
          "Index Name": "string_encrypted_v3_100000_eq_btree_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 635,
          "Relation Name": "string_encrypted_v3_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.42,
          "Total Cost": 8.44
        }
      ],
      "Startup Cost": 0.42,
      "Total Cost": 8.44
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using string_encrypted_v3_1000000_eq_btree_index on string_encrypted_v3_1000000
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
          "Index Cond": "(((value)::jsonb ->> 'hm'::text) = ((('{\"c\": \"mBbJdt#m#xF-e-eiNK9exV6i~9!7y<WX1y?!X2^$I3!3E>^kZw24Q{Ux3r)d74=laAYW?=YKsT+08{odOj1_;s(tmi-sG_stf(ev$Oo>sBBgd=Y;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_1000000\"}, \"v\": 3, \"bf\": [220, 422, 1613, 1366, 1893, 677, 748, 621, 167, 884, 1770, 973, 986, 1392, 578, 1098, 1114, 1968, 141, 1529, 290, 1897, 1772, 1106, 964, 981, 1603, 93, 1662, 830, 1066, 1470, 802, 1663, 38, 1325, 1412, 1592, 1487, 788, 1699, 1360, 1370, 1563, 602, 173, 1369, 584, 887, 551, 769, 331, 1710, 365, 296, 1766, 761, 2036, 2018, 1637], \"hm\": \"848bc7b916bdb4038118ecad4ec5cca948ff41d628e9c1c7f79c4b595dda3b4f\", \"ob\": [\"26157d3ada2a0dacf1e9f3d8959225b29bf4516b76043b053b4322e06c45b0bf7094c9b385464299cef6045327c7d2dbb09ac938175caf732efa2fd5fc9d5a04b6e61bc7df41237b3bcc3ea0fae2d492766d799e3c3beeeb24d82f43039473ba8faa768120b72af48d7e60373a1adb47c0fc48bb40bc8f01f4d9fa25c414db43799b04523bd26ef40c2ee80dbea777b741bed5c88384bab3405b928a948a9b35e886d7f3f585f7886e6621c7b18e6da2ef5c16ce142f23eebaf840370767702ad4ca227fba25fe07156c64d0e4786256f1b9fcde333d52ae825a0fe91a3e43945acbbdf38e9f2f65479f59d59c0985275875204b88b056f05f1abccdf10d7428dcfe0cf55f002fce27ff62e6e2b07c90e450311f5898391bc6062e035dd1346c7dce1dea519a88037c45a603adb1b3bd9295d3e633865c4ff98a51b1669c5c04dbcf76739f00b28481107b4166a7582f65387f31df13424b4c491d4c056261aae6888e821a3f86a06e8f24c3136e7ed801d15b3095db408a1a63e2ce7cef8462612c2948cbe34948daede275b3ee8588533668af10763068\"]}'::jsonb)::eql_v3.text_search)::jsonb ->> 'hm'::text))",
          "Index Name": "string_encrypted_v3_1000000_eq_btree_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 2,
          "Plan Width": 640,
          "Relation Name": "string_encrypted_v3_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.56,
          "Total Cost": 12.59
        }
      ],
      "Startup Cost": 0.56,
      "Total Cost": 6.57
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using string_encrypted_v3_10000000_eq_btree_index on string_encrypted_v3_10000000
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
      "Plan Width": 636,
      "Plans": [
        {
          "Alias": "string_encrypted_v3_10000000",
          "Async Capable": false,
          "Index Cond": "(((value)::jsonb ->> 'hm'::text) = ((('{\"c\": \"mBbLA^%@(97XVTaaM9LYMt-ftBR5CY5R#2|34Hi1MR-oCrXvA%LspSYovTTH9Ey}M<K*Ra#2`^<-%tc-aq9D+XFF-iavaM`40fvJu=kD!@oCM!f%2txVQh6}#1Q2)NxIzjOYXJFCv9j}(_#\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000000\"}, \"v\": 3, \"bf\": [1885, 295, 902, 1007, 808, 951, 792, 1645, 1593, 607, 661, 286, 1097, 159, 516, 870, 426, 1767, 1590, 1693, 151, 1751, 1664, 1058, 1561, 804, 802, 1641, 750, 556, 1821, 166, 1765, 341, 1451, 620, 474, 464, 315, 1774, 1350, 386, 586, 1805, 1467, 1254, 1341, 1355, 780, 408, 1315, 1670, 402, 1453, 925, 1676, 1699, 289, 1348, 588, 2021, 36, 1405, 1065, 1762, 1875, 564, 1309, 763, 1671, 383, 637, 410, 1075, 434, 634, 632, 723, 20, 38, 1241, 417, 1877, 1064, 83, 1081, 1952, 1589, 677], \"hm\": \"d2724b9d2d45e0f740a3e916f1c6dbd21ba93c51b1e298a24d6018ba6044696d\", \"ob\": [\"e23a02c59ee11eab8020a79df213755783bcaf970aaf69061eac8ea97b8b912015cb1436737b200f5b9a3094a55f8766eab33944b6845c16adecc15df7879a9d6ac329cc890a2ed026395d422c1de4f0221f10655a3fa5dd3617fd9587add7ca996d485751069db560add03feed5fd5616682fb53c2680db98d951dcf77d2b35ac8b0024865884708374dfcc038261248728b7454cfc80857355d33e9ae52db0680f6384a9e63f1cd2e030580ca42cd04f207da795060892359ba68a8e9290c6d323048cadfdd521e325083bda1565847f6e941de68a549b6d717cec3c6ad96b7a42b68e4eb3c78904e1825c1ce1271d20b467916b590e9e16b6bf81ef182997392b2e5fb5f160f12bf355bf0ed234f98cc6356cac072c59aa68213a245d3aad3c09bec73f602fbd831b9f7cf5a4622059aa95e3ccf9d0692b184e240f57cccfb878514b8b865a231f290cf3aa27cdd2a44fb84315183267bb127f64ca6067eb6c65fc7a3dc514cb4db0f2b1ed25e1123b678c074edfda3778c4ea29e5d8d3a36a89c29d68e119317f93b0787bd45263b11e2057d254df6d\", \"6e8a31fdfdfdfdfdd2a053164937bd39150b308f9210a6e0cdc08b07417c519d6ad5de4cd889057cbc2a7f8adf8c7b3ddcc2473e84b00b73a4d10f0e589325765d79859d47ade23cc46291670457cb1ba250da2b8b65b6c81fb158654d6d129a6e6e3a4955e7f9915b203582b0e8cd55897c95fecb19f69bf075819846b0efbe80f131fd8b586e2c5da505375c4e8d4aeabd1b5bcfcdc0a0339984a05c157ae88f51c13d317a6ca2674ccbb5c9dc0d05ca48e1a5b9a5e6ddc72c588e09937042d45b61d9ab921fac565364effebf6f8742686b1ce5667b47025be66f4c55e6350e615aa58a4e625216ddf2e1f77b70c0f5a76b46687b0a02c391e095e3b710b9f1ce7bc7abbb27932b191bcdc534ed573a44f48c7fa33e0ffdb329516f4f0a161779fe6a0b7ba776ecb56bbf1c8530a8fa5046b2d744d7f4aa92cba712e9984b6977bb963cd57b69d619f97205a5b83f7f2b7d9136ae128adde0ab1fd325e8c0a5f7a0ed77f7c056b50fc2a8ce3822ac36b7c21fdd832c03a6d8ec3ddc9e7029dd174eb2c3b477592adca13a4c5fa362479202572b8d193a\"]}'::jsonb)::eql_v3.text_search)::jsonb ->> 'hm'::text))",
          "Index Name": "string_encrypted_v3_10000000_eq_btree_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 8,
          "Plan Width": 636,
          "Relation Name": "string_encrypted_v3_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.56,
          "Total Cost": 36.7
        }
      ],
      "Startup Cost": 0.56,
      "Total Cost": 5.08
    }
  }
]
```

</details>

![Query Performance - EXACT/eql_cast](query_exact_eql_cast_chart.png)

## eql_hash

**Description:** Exact match using EQL HMAC-256 hash function

**SQL Query:**
```sql
SELECT value FROM {TABLE} WHERE eql_v2.hmac_256(value) = eql_v2.hmac_256($1::jsonb) LIMIT 1
```

**Parameter:** `Bob Johnson`

**Table: `string_encrypted_{rows}` with encrypted string values. Index: Hash-based unique index using `eql_v2.hmac_256`.**

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

- 10,000: `string_encrypted_v3_10000_eq_btree_index`
- 100,000: `string_encrypted_v3_100000_eq_btree_index`
- 1,000,000: `string_encrypted_v3_1000000_eq_btree_index`
- 10,000,000: `string_encrypted_v3_10000000_eq_btree_index`

| Data Set Size | Rows Returned | Query Time (no decrypt) | Query Time (with decrypt) |
|---------------|---------------|-------------------------|---------------------------|
| 10,000 | 1 | 124.09μs | 23.69ms |
| 100,000 | 1 | 125.41μs | 24.57ms |
| 1,000,000 | 1 | 122.42μs | 24.12ms |
| 10,000,000 | 1 | 121.98μs | 24.98ms |

_Rows Returned is the actual count from a one-shot pre-bench execution. For LIMIT-bounded queries it matches the LIMIT (or is lower when the table doesn't have enough matching rows); for aggregates wrapped in `count(*)` it's 1._

<details>
<summary>EXPLAIN plans (per data set size)</summary>

**10,000 rows**

```
Limit
  Index Scan using string_encrypted_v3_10000_eq_btree_index on string_encrypted_v3_10000
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
          "Index Cond": "(((value)::jsonb ->> 'hm'::text) = ((('{\"c\": \"mBbL*j*tj3m5o(Z#9rYvPK@!yBA^dMUbV6lODN0kzh%wSOZvH;SP?cwVd|3(Tlb(y5K95XAh4LoF4BlBpTdz~k>&{o(!)ZtPrXNt`~%ERYb%=RW~FvvY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000\"}, \"v\": 3, \"bf\": [300, 843, 1592, 385, 981, 88, 1197, 1414, 1970, 1824, 569, 1112, 558, 697, 463, 1221, 1925, 1603, 485, 19, 309, 238, 1786, 1637, 488, 2012, 1726, 207, 193, 373, 687, 699, 1710, 528, 927, 164, 277, 62, 529, 1545, 1154, 761, 1470, 966, 1766, 1582, 1066, 296, 1772, 578, 1325, 923, 1370, 1455, 1108, 1953, 1675, 1161, 1454, 167, 2009, 6, 1268, 1651, 1912, 870, 1412, 884, 1563, 38, 1854, 1911, 1756, 712, 551, 964, 393, 365, 857, 621, 1853, 17, 1662], \"hm\": \"b0b055247cded917baabf9b0200b89d76d32981c1ed299fd2210b3217f5baf57\", \"ob\": [\"c5c5aa2ca88a380754bc68f51c80961f466ccfbf630a65e114447e0fc086de94d65e963662935ec00983ec4e8ac643ce31599f418d5df32d71b4d66e9892c679f909bd89cfe0a01152cbe053126d527a31615775bd8c15940a1850271dacff08bcfb2463b6e4e0e2c1f5d65e2778db270453134bad4d46314f0ce95e8a271753b7b6efa372c1226905db028b6741814138b847c3ef7ff694294e05d9bc3eb206b825c928d0798cedad94dba2dec18774787269a2e8a98158e71af371c8280edb61059015d16dd3f6e43644dfccbcad9942e49342a7c5915b9fe7f4d4a912c02e0d61c30725008867b4c6996122a637bc9367ae24cd4e721bfcbdb56830f048d2819a83062346cc2e5409eed776f39ae54261f9f5ce184fdc0a5764e7c9a3f71580984112f1f5cf47835f60b6bcc56ca0bee345fc7047c7e8e7675a2e6097d6a48b7d3cfe9ac52937407bd570bc268ab82e060ddc569f4b08c6f48f8bf77e594ca47198b686529a0086c8ba116595f5e04714be721a235208f6d7ab4f6575d2f3ada740668d4fb7606481bd9265bf2b8c5e1c3cfd7e70449e\", \"951757575757575733f0468c7d82d838dbe077939798e6a3ad43556be5c5ed05ea70a2e354ee8b6f3d0566d065838731e54792d7fc5828ccde3e800c818bc246243c799bbe9f15d5b0b8ef862c5b8e740435aabc91b2babea3dc1f93e9828651582e0b25dd28a2f7e8522b56556151b6ce7ef45ab2fb89af4e88ba5c55eff0f1d8137b92e7b94f88416b4cb4144a3a960ac995d76fcd1dd1821eb2c081aead71ff9a373e62d740c80119a344c6d14a96f2bc78b21849458b507d1b2a2c9ca2b64260122b1834111d7f4582b76cfd0d8ec516a7dc72bdc0d3248ccf39e8182f855b164190e86279d4a7b46b053286fb13aed52abdd9952a168c8cd567b435512ee76f952a17a403d80c254539c7c7018eb2902716688c98008594eb0e77e9ba7e7c41f13080c4088428182db3565a5ec76ec611949cf1a1e2624942eceff2c8e6fa81fe91118c7bd718cd3c476ab3f08d9643c9f2a0ccd8c3d1304f5c3917f1ff31e55743a438dc2a104b61a92440036a90998a8aba29777d63713af08326dfc7340ca68e58c71cec1053a14d0c016ae1aca548d84e1c39ee\"]}'::jsonb)::eql_v3.text_search)::jsonb ->> 'hm'::text))",
          "Index Name": "string_encrypted_v3_10000_eq_btree_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 649,
          "Relation Name": "string_encrypted_v3_10000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.41,
          "Total Cost": 8.43
        }
      ],
      "Startup Cost": 0.41,
      "Total Cost": 8.43
    }
  }
]
```

**100,000 rows**

```
Limit
  Index Scan using string_encrypted_v3_100000_eq_btree_index on string_encrypted_v3_100000
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
          "Index Cond": "(((value)::jsonb ->> 'hm'::text) = ((('{\"c\": \"mBbJSl}Ec!H_TSK+j{`dYu2U2A+WSn+k~c)jauZ$1dPl4<h58Y>hr;4M{-+}t2P>1$}+?t)i&A-hFy)yv#8w{K!4|Jn!!_<<+Oe!$FC%WjGyu3rFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [1765, 1818, 1593, 1485, 1373, 1920, 1333, 725, 1309, 1969, 586, 1350, 1531, 159, 792, 36, 925, 556, 438, 804, 426, 827, 1841, 1090, 204, 637, 141, 1254, 238, 147, 1315, 1230, 1556, 1541, 1355, 1247, 351, 1504, 286, 1590, 588, 1081, 748, 951, 410, 1615, 1537, 1422, 1400, 1075, 1592, 1885, 1826, 233, 1453, 461, 632, 1813, 18, 1405, 677, 251, 1699, 1341, 808, 763, 1589, 1641, 607, 1994, 1697, 300, 1805, 130, 1672, 2021], \"hm\": \"28369ac87a26801c88729f107b6a0a59ec8e7df2bc199971e57de95fa4d9a205\", \"ob\": [\"7d5be628a8a1a5d8550934ce6650d06a3ba0db032e9677c59070ec3b61f5e1410f873fc57f0455f8e4866ec8d044b7e2e5daaba310d45ce075dcc639715a04a7650f34f2a945608192d1081cac077659b02af00c5dc97d17fe033eb5a150c0488ba090f61c0dfa884edc8309a5d8399e0ed283202f0dc595b8d11df49b74c3d8707682824faac4535b619a031fdca7f5b89c691569db58fcc3b9ffa6da05ab1a2420374b9c0c3b6771cc1d5a6f763f02a79867f791b0cba7f72cc2bd75b33addc30d37e82675777ab434e7666ef899093d1b51c5f064e3b6b6963d2cf007a54931d804d3d234e32e46e8582af0922b6bd2a94aac5379f87161c25286415902270a70b0ebccbf4f0ae1112a37a56e28e300b1620a3d96b4c75bf94ff119eb415695bd919500158da3035d01e0f87a13ef2bfd600b21b9aa892966aeda4873324b9ac7e9d2b08e6b47943dc3e645c790aaca262655bbe14928cc6c98b4416210f095f17e45230719163c2c8d76644b1a4746380e403afba5482a4e96e408563780f34949f4eb82fc068fc6d29e6416748a6e192f2581efa68b\", \"e499141414141414c02a268347ec03abbc18838e87c562b0389adeed563dcfe75dff440e298879f077147f22e6d37d5311b9f9ddde45e5f6febcb10067ea8dbb560ddf7e01037c2e470f95abe04b8ed9cfeb049b4c5b7b29ec51eee9c826b3d52f7274074e6e7be64c11ee74e2c591e076be277277d339108032cfeaef2d68b0a7e7b295526b03cafac6f413202c5ae86ebe3824ee28ee25125695e9a5a2d8c06a2af68be0c33dd1fc3ad6bf26899a0072437ae3a96356ebad1c9ba853d464564ed526540fb0852578af0eddb2b4dfe4c55d6367669d5985dc3f1ef5922924dd6bfb7fa06d4ef848f4b986d44a01971a0c9849a848202c6e4a03e55e6cdd7fca30f547a399744c5522fd30284ba30cb00a2cf4badd87a20cd151798cbdbf6238c7fb8979addee51ae087e69cd281ce5cc00d4aeacbc36cbf9e30dd925cb906c6cfda1822f665c3d92562e928b1630f25cf343b86c81b543044a191beaf5e1352b1029d2ce4bb4c6cdfed068a1285e736dd27daf65e96cfb747db8ecf0456be2ec3b62c18a803ab550e1fa6dc86e7c1576837fa93c1451656\"]}'::jsonb)::eql_v3.text_search)::jsonb ->> 'hm'::text))",
          "Index Name": "string_encrypted_v3_100000_eq_btree_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 1,
          "Plan Width": 635,
          "Relation Name": "string_encrypted_v3_100000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.42,
          "Total Cost": 8.44
        }
      ],
      "Startup Cost": 0.42,
      "Total Cost": 8.44
    }
  }
]
```

**1,000,000 rows**

```
Limit
  Index Scan using string_encrypted_v3_1000000_eq_btree_index on string_encrypted_v3_1000000
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
          "Index Cond": "(((value)::jsonb ->> 'hm'::text) = ((('{\"c\": \"mBbLpfeKUBfl{rFVU-@|s<eQ_9`WHpO8x-EbP%6D>B<iaGo#G%rgZZ+G~lUo3H1HMAkGZlce(0pZI0k&ll$yo#m&;o>|z?Ay}8nH44)4k_@#DXY;|SC5al#Uy4?0l?zPD$ZD>~0Vg\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_1000000\"}, \"v\": 3, \"bf\": [964, 1360, 1369, 830, 1663, 748, 1968, 1710, 1098, 365, 2018, 1766, 331, 884, 38, 1770, 1370, 1529, 1487, 220, 1772, 296, 981, 621, 1114, 141, 2036, 1412, 1366, 1897, 788, 1603, 1066, 986, 1637, 173, 887, 167, 1699, 677, 602, 1563, 761, 769, 973, 584, 93, 1592, 1106, 1470, 290, 1392, 1893, 578, 1662, 802, 1613, 422, 1325, 551], \"hm\": \"848bc7b916bdb4038118ecad4ec5cca948ff41d628e9c1c7f79c4b595dda3b4f\", \"ob\": [\"26157d3ada2a0dacf1e9f3d8959225b29bf4516b76043b053b4322e06c45b0bf7094c9b385464299cef6045327c7d2dbb09ac938175caf732efa2fd5fc9d5a04b6e61bc7df41237b3bcc3ea0fae2d492766d799e3c3beeeb24d82f43039473ba8faa768120b72af48d7e60373a1adb47c0fc48bb40bc8f01f4d9fa25c414db43799b04523bd26ef441d4aa82be97710bebf2a5ab7b20d4f9d5ed764b7f1c85713311594bbc7d4ed56ce568ae730c28fc8bb37ffb6f72486f02f910903a78727abe013d2e3ab2025aecc80da2ecec6fffa65775c9448f76a34940235cdc8b3526de544e0c6eb2803de8016a3e8f59be3f0e8d06aa30aa58543eba5cabad8c17aff95452483600d1a9a901a2b112b66cfc46c33dc710426a84436c3d33bfd8c006a118e0898d3a15c09d1bf97f9b6864c807a8920fa5705c0bba2c274b817a3cf89a1c21e796b47dff20c4da5150996cb965dce5de13e13fce060b001cbce747dfca51cd8dc74da9892a2c1df3a2da233eb30d5a597ad2c76bfd4a9d2b220815681e9163966d85fe30eec8b9ce1dcf3ce84a146a9ddb72d6f6\"]}'::jsonb)::eql_v3.text_search)::jsonb ->> 'hm'::text))",
          "Index Name": "string_encrypted_v3_1000000_eq_btree_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 2,
          "Plan Width": 640,
          "Relation Name": "string_encrypted_v3_1000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.56,
          "Total Cost": 12.59
        }
      ],
      "Startup Cost": 0.56,
      "Total Cost": 6.57
    }
  }
]
```

**10,000,000 rows**

```
Limit
  Index Scan using string_encrypted_v3_10000000_eq_btree_index on string_encrypted_v3_10000000
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
      "Plan Width": 636,
      "Plans": [
        {
          "Alias": "string_encrypted_v3_10000000",
          "Async Capable": false,
          "Index Cond": "(((value)::jsonb ->> 'hm'::text) = ((('{\"c\": \"mBbLqC5@G1suIviQg^nWXnOg?BbiPoYlp6+G9t?!&!?K%ASRq!)qQ37AlfBV@ejYDkVIUE#2}J{*(X&vygtFk*+>$~{febuEPC^=d=Ht)hn?;Yg=(dCVQh6}#1Q2)NxIzjOYXJFCv9j}(_#\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000000\"}, \"v\": 3, \"bf\": [2021, 286, 410, 802, 1065, 386, 780, 1341, 1467, 632, 289, 1670, 516, 402, 1254, 808, 1451, 36, 1699, 1664, 1676, 1315, 38, 1081, 1075, 1405, 607, 1875, 341, 951, 408, 556, 1453, 426, 295, 1767, 166, 564, 151, 1885, 1589, 586, 804, 315, 1877, 792, 474, 902, 1007, 1821, 83, 1762, 1751, 634, 925, 417, 677, 1350, 434, 1097, 1693, 1952, 1348, 870, 1590, 1241, 1309, 1058, 637, 20, 1355, 763, 383, 1805, 1064, 1765, 159, 1774, 1593, 464, 661, 723, 1645, 1671, 588, 1561, 620, 750, 1641], \"hm\": \"d2724b9d2d45e0f740a3e916f1c6dbd21ba93c51b1e298a24d6018ba6044696d\", \"ob\": [\"e23a02c59ee11eab8020a79df213755783bcaf970aaf69061eac8ea97b8b912015cb1436737b200f5b9a3094a55f8766eab33944b6845c16adecc15df7879a9d6ac329cc890a2ed026395d422c1de4f0221f10655a3fa5dd3617fd9587add7ca996d485751069db560add03feed5fd5616682fb53c2680db98d951dcf77d2b35ac8b002486588470df6572451311826120cb16cb80f64253c8b50c85154266fd70ff15fd2858b9e6770ba811816ccf00a648fe382f730637bbe32ca7200e7c3949d50c3c7b36a27e4a1f0c53998479afb8d3fce537b7e01102ad77477f182da29923fc84a285ecc7b9b003acbdaf56783f7e773f88a6cd47e0e157d533e8a169ab6d3fdef0a292a620f89d04a2f1926ba4d7826c49a04ba122e3e30bf1e068327e2cb438a859b6801a52a38da654bf9c0f647c1e4c35e1b23e42a74621ae027c23322becbc337879fb230f3b019516fbc6b38e1bda57bac2286b9e471f9042b2cb578cde41f66bcd23c03e6b2d04d97f63fc70cddd8ad73f82a6ee293431b451bcfa17f5e5bd7f1a26f73d8eae9312ac507e925e77f9a8c7\", \"6e8a31fdfdfdfdfdd2a053164937bd39150b308f9210a6e0cdc08b07417c519d6ad5de4cd889057cbc2a7f8adf8c7b3ddcc2473e84b00b73a4d10f0e589325765d79859d47ade23cc46291670457cb1ba250da2b8b65b6c81fb158654d6d129a6e6e3a4955e7f9915b203582b0e8cd55897c95fecb19f69bf075819846b0efbe80f131fd8b586e2c66cb75d1cdebeb53ed1fac08762e77a03c76bd6b3c30ed6506f8979827234f39bb0c7a2854e0f559f330b220509ad0b637e0e591d2ad107dfc546bba015d25704e9884a83a918ac136dfdfd858aa5dd3b4154e84a54859faced3ab538d993caf8be511d26ed0dcf8991adfa956bda96ed4d1bcef8f4819a99acd9560c825ead72134b715b07156deb9538641936a96d10f8ab163af40d54011ca03e2caaef477ac36c9b56ca5ec2a930d75b8b4c35e45e40d3be8419f71dc16c45031f9f5a5d6c0a2719f84acf367c754318d7d6a0d65af7bd1f00e4b1df511b3af76e98fc701655bcdc5c9765b7539e6918b316a6b045ee5a463e5bf2bed05ce9e6a6947ee524391cd85db4397c688006e171da346bd\"]}'::jsonb)::eql_v3.text_search)::jsonb ->> 'hm'::text))",
          "Index Name": "string_encrypted_v3_10000000_eq_btree_index",
          "Node Type": "Index Scan",
          "Parallel Aware": false,
          "Parent Relationship": "Outer",
          "Plan Rows": 8,
          "Plan Width": 636,
          "Relation Name": "string_encrypted_v3_10000000",
          "Scan Direction": "Forward",
          "Startup Cost": 0.56,
          "Total Cost": 36.7
        }
      ],
      "Startup Cost": 0.56,
      "Total Cost": 5.08
    }
  }
]
```

</details>

![Query Performance - EXACT/eql_hash](query_exact_eql_hash_chart.png)

