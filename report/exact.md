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
| 100,000 | 1 | 128.49μs | 24.64ms |
| 1,000,000 | 1 | 123.30μs | 24.26ms |
| 10,000,000 | 1 | 139.53μs | 24.50ms |

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
          "Index Cond": "(((value)::jsonb ->> 'hm'::text) = ((('{\"c\": \"mBbJ`dQ743VNri6V!5qfk6cB>A-B!5DvgHsyQh^2D|7_#37!Oj@Ks-cS%&*3MnoZC@C?Ktx<7F#yYZO+G3G6rz(BFALHQrAqW<zdz^erK_8|TYrFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [438, 1341, 426, 141, 1090, 1826, 1537, 1841, 1592, 1333, 1969, 804, 300, 632, 1589, 1350, 1373, 410, 1994, 725, 637, 1697, 238, 1556, 1504, 1615, 1641, 251, 1254, 951, 286, 586, 827, 925, 1405, 204, 18, 130, 233, 159, 1247, 1590, 461, 1230, 1081, 1485, 1315, 808, 1885, 1672, 556, 1593, 1920, 1422, 1818, 1400, 748, 1531, 1813, 147, 1699, 588, 1765, 1541, 763, 2021, 1309, 1453, 36, 1355, 792, 607, 1075, 351, 677, 1805], \"hm\": \"28369ac87a26801c88729f107b6a0a59ec8e7df2bc199971e57de95fa4d9a205\", \"ob\": [\"7d5be628a8a1a5d8550934ce6650d06a3ba0db032e9677c59070ec3b61f5e1410f873fc57f0455f8e4866ec8d044b7e2e5daaba310d45ce075dcc639715a04a7650f34f2a945608192d1081cac077659b02af00c5dc97d17fe033eb5a150c0488ba090f61c0dfa884edc8309a5d8399e0ed283202f0dc595b8d11df49b74c3d8707682824faac453dfdb698e74c89e2e7b8ba76eb716a701185e137fa721ec9e31ec1e84be944f84613baff9bb2b2fab4a3061b9894497ddefbf7122dcac0baaaf4a9bcbc465720f448b924f796031f34a4073fa1a6638ca2414001e7ec99bc5d73deffb10b7341cc87632eb8cc96adcd12c81ac730c1ac1cd6fda24ce5ac69d39fe246d3e2550270707ebc5b7daa8cc30da32f94ee51f2c6c8e74df94d1a70d8965ba59046c93b2de58542353f32955136b2999a6cc193f9f96d335b74240e87430892546b20b9269c1bba9f4ef315c096c376f0af284bcb9d61ed7428735b32d70073d74e35b13d67fcf13b8076f996258d66f81ae7730bda6fafd6a0e4d1c23debe7ade002f5a9806d605a69eb86293f82829fb9f7e25\", \"e499141414141414c02a268347ec03abbc18838e87c562b0389adeed563dcfe75dff440e298879f077147f22e6d37d5311b9f9ddde45e5f6febcb10067ea8dbb560ddf7e01037c2e470f95abe04b8ed9cfeb049b4c5b7b29ec51eee9c826b3d52f7274074e6e7be64c11ee74e2c591e076be277277d339108032cfeaef2d68b0a7e7b295526b03caa28c8bcc594908ddb19783fdd7be38aea12a5b297ed026a3f2277cdde54a2775c461fd2f685e458f1ba4a62bf5273b381d180769a5081a6d12d422de75bc44b7d02f6f8c813074efc03f2336df93d3a60781e31c2fbfec643cc4d043784fab5ce1373928e60589102313511e89d9a0453c362b9afa6dab379aff121d615e5577294669fc12977c688203d9104fda879790aa5ee1cbdfe2fb389c8d89e0ebb7ea39355b1d7e0deb126cf1094a0078a06cdddb172ee9557670b6d240c5a38fdf2e45c183aab2a6163163dc4adf0ff9e774d47d22571dd1e4240afd844379e87cac25bc365daba6df9ba6e96d569ade9e4fac3782e46d1059db0d6cc26cf6ada7c377083b019c355aa15938f8ee9005898d\"]}'::jsonb)::eql_v3.text_search)::jsonb ->> 'hm'::text))",
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
          "Index Cond": "(((value)::jsonb ->> 'hm'::text) = ((('{\"c\": \"mBbK?QXzw32|T??Ne8jm&ADB~Bfp%d+Wwe1a5phQXT>-dTWe<-Y4w1G8-^?u@CNaM{A&{y#2}Kn08ck^2FsAqyMXtynRS{Gph_c(8$k`_V)&|t3L>R;VQh6}#1Q2)NxIzjOYXJFCv9j}(_#\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000000\"}, \"v\": 3, \"bf\": [151, 1590, 1767, 634, 383, 83, 1065, 474, 426, 1341, 1589, 780, 1309, 620, 1254, 159, 632, 289, 1762, 2021, 1058, 1805, 1693, 516, 1821, 1765, 808, 607, 792, 1405, 1097, 1645, 588, 1885, 723, 1699, 677, 464, 410, 1355, 20, 1350, 1671, 637, 1075, 1593, 1241, 1467, 1877, 1751, 38, 402, 315, 1641, 36, 1348, 1451, 902, 564, 1774, 951, 1670, 1561, 417, 802, 434, 286, 386, 1453, 556, 341, 1007, 408, 763, 1952, 586, 804, 1664, 750, 1064, 870, 1315, 295, 661, 166, 1081, 925, 1676, 1875], \"hm\": \"d2724b9d2d45e0f740a3e916f1c6dbd21ba93c51b1e298a24d6018ba6044696d\", \"ob\": [\"e23a02c59ee11eab8020a79df213755783bcaf970aaf69061eac8ea97b8b912015cb1436737b200f5b9a3094a55f8766eab33944b6845c16adecc15df7879a9d6ac329cc890a2ed026395d422c1de4f0221f10655a3fa5dd3617fd9587add7ca996d485751069db560add03feed5fd5616682fb53c2680db98d951dcf77d2b35ac8b002486588470e298ce430d01123197cb54df8743ba8663fe937431b77cbae0acf2d7522c2924cd3255dbadca67fe04ee82a20237c393bb6761c206af9d3461e8e77bd3e3372d92213e31270582c42d390fdd3796c00d2c2f9ed3ef52797aadc70ffcfc1abf5c7a8cf4fd30795da97a58d48125a7c6f7752e9146a045999e87ef16e391c9cec9a0722417b34a00afde420118180f9433587db4c9f6d70c85058755be5e5c394d0acde1d459dfb2ae3e8b26170ac48e9ef41706e4c6c6237ac89d6a90309529ff3be3981da5929d8654015693f45f2f5398bf20763843bee5581ff482e8ce23d3b6d997b90d352feb91e454302ccd6efb7e516a65f22500dee58f94739f3a612b3e8741d090bb897ace4e898c8ec45c54\", \"6e8a31fdfdfdfdfdd2a053164937bd39150b308f9210a6e0cdc08b07417c519d6ad5de4cd889057cbc2a7f8adf8c7b3ddcc2473e84b00b73a4d10f0e589325765d79859d47ade23cc46291670457cb1ba250da2b8b65b6c81fb158654d6d129a6e6e3a4955e7f9915b203582b0e8cd55897c95fecb19f69bf075819846b0efbe80f131fd8b586e2c7193252d84a0b1c71d5bd10cc3ddc3ed8ea11056549c5fced9b0289f154c158bff44cd826ee1fd1eadc3a410e10e6fd1d9c2c92611ed260ba33e79142e51932027e105a305c59ccf3a6f1a88344e3913c3fc276325dbff6975a5a7bcf91ba340f51838a61702b56e025549d07f8a498ad639542d49d3a43ebfce8162d9b10a3ecdd6b702d40c4cae67a7d7f06eda751a4a72ac5c90bec054a44a36aec836d61b360c3497af64e55c4dee6a44fb14e2cd13f089f1f04bd0117dae60715320e6ac0f2793cf68860a533553e97b772d2da5ddb7a7aec7620ee3e923634a3a5ee4e667f40fb8de18f00dd8bf33bae557ea7451176500f514ca3f833067254175a5e3983a1a14186f942b7f94799aa0ae37a4\"]}'::jsonb)::eql_v3.text_search)::jsonb ->> 'hm'::text))",
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
| 100,000 | 1 | 125.66μs | 25.27ms |
| 1,000,000 | 1 | 122.42μs | 24.12ms |
| 10,000,000 | 1 | 135.05μs | 24.51ms |

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
          "Index Cond": "(((value)::jsonb ->> 'hm'::text) = ((('{\"c\": \"mBbLfXfPh_<G`lQJOHsjHMEJuA(1uyld<SAoPH5E%(^YZh`k0-iRl?}Y6!TnKj!&7gQvtG_@je6i33uiSid7m!}@OuiM0L%4+yb}0aLxBRL6cKrFLO#b!Eg5<upmU-1bZEwaF)KXjao=\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_100000\"}, \"v\": 3, \"bf\": [763, 1350, 130, 300, 1247, 1592, 1373, 1994, 1805, 1504, 1405, 1826, 748, 1485, 1333, 1589, 1453, 1885, 607, 426, 141, 1699, 1641, 2021, 808, 286, 1309, 1090, 586, 1254, 1400, 1672, 1531, 588, 1590, 677, 1422, 1920, 159, 1537, 1541, 1593, 827, 351, 1697, 1315, 36, 204, 556, 1341, 1969, 238, 637, 438, 1355, 461, 1230, 1615, 632, 792, 233, 1818, 147, 1813, 804, 1075, 251, 410, 1841, 1556, 18, 725, 925, 951, 1765, 1081], \"hm\": \"28369ac87a26801c88729f107b6a0a59ec8e7df2bc199971e57de95fa4d9a205\", \"ob\": [\"7d5be628a8a1a5d8550934ce6650d06a3ba0db032e9677c59070ec3b61f5e1410f873fc57f0455f8e4866ec8d044b7e2e5daaba310d45ce075dcc639715a04a7650f34f2a945608192d1081cac077659b02af00c5dc97d17fe033eb5a150c0488ba090f61c0dfa884edc8309a5d8399e0ed283202f0dc595b8d11df49b74c3d8707682824faac453c558c12f5bc2e0fe2e599e88d2de9f09aafbee8c6014793b1cb0ddd2a4307d2e570dc76032351b60ba62dd547b93529adb3e3c4266dbbb374969ec0ef9b4756ce5e9c34abbdfd24d226b5eb29089f4c961834728df37ca33129d9a62c5f3c3f716a3760845fde0f014e192be4d23aac93179a10fd97bbd55b7132be6c21f1e3aa69d7c472bc7e89b700db0cc5e5944e86be35257368510d495627ac3ee72f31ee3639a5c8549f8cbbf5242a422d0764b7e6e857ae7df5ed79195aeb64b745651733aa05ce280bdecf5c1bfa83baa71f4ce9571fb8f6d25ed16f0cd85f13549f876aadd37ee0201bc8225a8b251858ef79b0cebf061b280b95fcdf011fdff5afb2d67e311da07a74608b9f718e2859594\", \"e499141414141414c02a268347ec03abbc18838e87c562b0389adeed563dcfe75dff440e298879f077147f22e6d37d5311b9f9ddde45e5f6febcb10067ea8dbb560ddf7e01037c2e470f95abe04b8ed9cfeb049b4c5b7b29ec51eee9c826b3d52f7274074e6e7be64c11ee74e2c591e076be277277d339108032cfeaef2d68b0a7e7b295526b03ca9af2f52a7a79514485084affea92f9c30bad125eba59f9157f845424a8a234a77015ed794f88778edbcd5f0560da82947f5d6373f65a7a53aac8061553e462cf148232de721ce3325893146f7f4e66ee43f2a81bc54a7a74bd547e01f8f86517353f36b83f5cdc2ed1cae4401f0f02e88ec55c6bbe38afa52760e1b0bd43bdeb4998ae8b9ecbc2ecb2d0f2b67423307b4991732ca7aedd0a02414e2bf9b82f252f30e905cb92dc32072774596560f0707795a5e30ca8ec6754d5b52aa51ca1a832dec43cfb57762f8bc710d8ea215b0e680413c4710b7282d29741c27f6c1dd7d13757489d8455063cfa3a6ff6a228e3558376685add3df16351786467fbd161bd84cef5152bb93200db7654f7a2a7e1\"]}'::jsonb)::eql_v3.text_search)::jsonb ->> 'hm'::text))",
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
          "Index Cond": "(((value)::jsonb ->> 'hm'::text) = ((('{\"c\": \"mBbJtXxF2<1gv3ujE<m6*CO}CBkV`LZGc`exPHXfZkombAh0UcSEMFyp|A`dZ26faMO8eD#2~v5v5Jq;lKeoJXiyz|zoJxpW1FNc^x}5U&gGw`9ml11VQh6}#1Q2)NxIzjOYXJFCv9j}(_#\", \"i\": {\"c\": \"value\", \"t\": \"string_encrypted_v3_10000000\"}, \"v\": 3, \"bf\": [516, 1254, 556, 804, 586, 36, 1350, 1589, 951, 1762, 159, 763, 1355, 386, 1451, 151, 1765, 474, 1664, 289, 1699, 402, 607, 426, 1821, 1097, 750, 588, 20, 1007, 1645, 634, 464, 1676, 1875, 383, 723, 83, 1877, 1064, 2021, 780, 1590, 637, 1075, 38, 1670, 286, 902, 1341, 434, 315, 1671, 1774, 1081, 1805, 925, 677, 1952, 1641, 166, 1751, 1058, 1693, 1561, 1453, 792, 620, 808, 410, 408, 802, 1348, 1241, 564, 1467, 1309, 1885, 1405, 1767, 1065, 417, 1593, 632, 870, 661, 341, 295, 1315], \"hm\": \"d2724b9d2d45e0f740a3e916f1c6dbd21ba93c51b1e298a24d6018ba6044696d\", \"ob\": [\"e23a02c59ee11eab8020a79df213755783bcaf970aaf69061eac8ea97b8b912015cb1436737b200f5b9a3094a55f8766eab33944b6845c16adecc15df7879a9d6ac329cc890a2ed026395d422c1de4f0221f10655a3fa5dd3617fd9587add7ca996d485751069db560add03feed5fd5616682fb53c2680db98d951dcf77d2b35ac8b00248658847072638cda153353659058385e811887f75b5d5587019748084a207be74b57bfd634285e0ff20b860e5f3ebc9b144ea4dd6c90b9581a8c3ca910322ab0ec7abfc531304cce0207376bcbc9821ddda7c010366ffe266f3402c5b4fddbbcc9288706a123e7f6ddcb71fc6abae9a2c3f140c6d9998c62fc32c2c34c5213fd8b2ba4a40a648046c896cf4f834bfd55d7772f4ead88ee61c129cfce807ba6dac220e7b796f580e228e203c8779fd507945bb1a1b5eda998174e2783c458eda90e6ef4a45d4520f29e68073814df53d8cb412f18a752fc2cbd084338146577c33f1951892ecb62862048a9c351cbf48ae65bbe43cf5e5afa147b188a4ae7d41f643fb10553bff5ec6f38ca3ce4271b1c5718a9c4\", \"6e8a31fdfdfdfdfdd2a053164937bd39150b308f9210a6e0cdc08b07417c519d6ad5de4cd889057cbc2a7f8adf8c7b3ddcc2473e84b00b73a4d10f0e589325765d79859d47ade23cc46291670457cb1ba250da2b8b65b6c81fb158654d6d129a6e6e3a4955e7f9915b203582b0e8cd55897c95fecb19f69bf075819846b0efbe80f131fd8b586e2cbe7717aa2c21f469870220b7d90d34ba4cee398bc85751a4ada8b6448503b7f1c148f1b273bbc4145234b1483c85db7addba5cf765aafdeef449922cb9d26b51869881570ba49d0f921643c0312fcd14943220444e58b8e949dbc0347f2d21970258bb00a1d4b51e97cda34c3901e94c6afe4af0163ce482b61f203f74c20fad5fa57d5657dac6e8be19332d1b5766f39a86b822c21d5a05a20fc596b005c2c68213ffe5d0182f75d08b1cdbf8bbcec6c4ece066e8fe888ccca27708c1ba5ab1941305a24db4b1a533bb7ee7383d70e27ea60e9b9e73334b28edeec07c1f4b985a9cf2eb05d1b52961c41ed3b97d561c2c54951a37456ec0764bec5546294cea9e4ffe411a461e9aa82d660cd179542e\"]}'::jsonb)::eql_v3.text_search)::jsonb ->> 'hm'::text))",
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

