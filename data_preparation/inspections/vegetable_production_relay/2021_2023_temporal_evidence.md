# 2021–2023 daikon `cat03` temporal mapping evidence

Investigated 2026-10-11. Scope: e-Stat daikon database tables `0002008540` (2021), `0002120386` (2022), and `0002114330` (2023). **Finding: the temporal identity of all 13 blocks is supported by dated official spreadsheets and exact cell-level matches.**

## Independent source

e-Stat publishes a separate official spreadsheet headed `だいこん 計` or `だいこん <month>月` for every period in each year: [2021 listing](https://www.e-stat.go.jp/stat-search/files?cycle=7&layout=datalist&page=1&tclass1=000001020455&tclass2=000001169106&tclass3val=0&toukei=00500226&tstat=000001015623), [2022 listing](https://www.e-stat.go.jp/stat-search/files?cycle=7&layout=datalist&page=1&tclass1=000001020455&tclass2=000001209867&tclass3val=0&toukei=00500226&tstat=000001015623), [2023 listing](https://www.e-stat.go.jp/stat-search/files?cycle=7&layout=datalist&page=1&tclass1=000001020455&tclass2=000001220580&tclass3val=0&toukei=00500226&tstat=000001015623). All 39 spreadsheet URLs, SHA-256 digests, source code ranges and comparison scores are in `2021_2023_temporal_mapping.json`. Exact downloaded bytes are retained locally in the Git-ignored `runs/official-daikon-2021-2023/` directory.

The three raw API captures were replayed offline against their matching metadata captures. Each passed checksum, page, count and code-reference checks with **14,976 observations**. The matching capture paths and metadata hashes are in the mapping JSON.

## Comparison method and result

For each dated spreadsheet, compare every displayed cell in its 48 origin rows, 12 consumer-region columns and 2 measures with each of the database table's 13 candidate `cat03` blocks. Row numbers come from the spreadsheet's explicit origin index, columns from its region and quantity/price headings, and database values from exact source `cat01`, `cat02` and `cat03` codes. Compare lexical strings, including `0`, `-` and `…`; no numeric imputation or month-order assumption is involved.

**Every spreadsheet in every year uniquely matched one block at 1,152/1,152 cells.** The next-best candidate block for any spreadsheet matched at most 872/1,152 cells. Thus all 39 period-to-block assignments have a unique exact match. The workbook's own title independently states the period for the matching values.

| Period | Block | `cat03` first–last code | Result in each of 2021, 2022 and 2023 |
|---|---:|---|---|
| Annual `計` | 1 | `1001`–`1012` | 1,152/1,152 exact |
| January | 2 | `1013`–`1024` | 1,152/1,152 exact |
| February | 3 | `1025`–`1036` | 1,152/1,152 exact |
| March | 4 | `1037`–`1048` | 1,152/1,152 exact |
| April | 5 | `1049`–`1060` | 1,152/1,152 exact |
| May | 6 | `1061`–`1072` | 1,152/1,152 exact |
| June | 7 | `1073`–`1084` | 1,152/1,152 exact |
| July | 8 | `1085`–`1096` | 1,152/1,152 exact |
| August | 9 | `1097`–`1108` | 1,152/1,152 exact |
| September | 10 | `1109`–`1120` | 1,152/1,152 exact |
| October | 11 | `1121`–`1132` | 1,152/1,152 exact |
| November | 12 | `1133`–`1144` | 1,152/1,152 exact |
| December | 13 | `1145`–`1156` | 1,152/1,152 exact |

To reproduce the comparison locally after obtaining the 39 files at the paths in the verifier's docstring, run `python3 inspections/vegetable_production_relay/verify_temporal_mapping.py`. It checks each spreadsheet checksum and period heading, all 48 origin rows, all 1,152 values against all 13 candidate blocks, uniqueness and the recorded runner-up scores. It exits unsuccessfully on a mismatch.

## Decision boundary

This resolves **temporal identification** for these three exact table IDs and captured revisions. The historical M3 handoff correctly treated the mapping as unknown at that time; this evidence supersedes that particular uncertainty. Do not generalize the mapping to other commodities or revised tables without checking their values.

The annual block is a source `計` category and must not be counted alongside the months. Unit, origin, destination hierarchy and value-symbol interpretations are summarized in [`SOURCE_INTERPRETATION.md`](SOURCE_INTERPRETATION.md). MAFF states that the annual handling survey target was revised in 2022 to wholesalers in central wholesale markets only; the effect on these exact tables remains unconfirmed, so use caution when comparing years.
