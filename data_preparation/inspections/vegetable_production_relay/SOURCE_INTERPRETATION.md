# Daikon source interpretation, 2021–2023

Scope: e-Stat tables `0002008540` (2021), `0002120386` (2022), and `0002114330` (2023). This note records the practical interpretations used by the application. It is based on the captured metadata, all 39 official period workbooks, and the official survey descriptions linked below.

## Supported interpretations

| Source field | Interpretation and application rule | Evidence |
|---|---|---|
| Period (`cat03` blocks) | Block 1 is annual `計`; blocks 2–13 are January–December. Do not add the annual block to monthly values. | Each official annual/month workbook uniquely matches all 1,152 source cells for its block. See [`2021_2023_temporal_evidence.md`](2021_2023_temporal_evidence.md) and [`2021_2023_temporal_mapping.json`](2021_2023_temporal_mapping.json). |
| Measure (`cat02`) | `1001` is quantity; `1002` is price. The relay visualizes quantity and excludes price. | e-Stat metadata and period workbook headings. |
| Quantity unit | All 39 official workbooks label quantity as `t` and price as `円/kg`. The 2021 API metadata omits units; use the workbook evidence to label its quantity values `t`. | Official workbook headers, locally checked for every period in 2021, 2022, and 2023; metadata inventories. |
| Origin (`cat01`) | `計` is a summary; codes `1002`–`1048` are origin prefectures. Label them as reported origin (`産地`) prefectures. The values describe wholesale quantities attributed to origins, not agricultural production totals. | e-Stat table title: “主要消費地域別・産地別の卸売数量及び卸売価格”; [MAFF survey overview](https://www.maff.go.jp/j/tokei/kouhyou/seika_orosi/gaiyou/). |
| Destination (`cat03`) | Each period block contains nine level-1 major consumer regions and three level-2 subregions: `東北_仙台・盛岡`, `関東_京浜`, and `九州_北九州`. Aggregate the nine parent regions only; do not add their nested subregions. | e-Stat metadata levels/parent codes and workbook headings. See the [2021 e-Stat table listing](https://www.e-stat.go.jp/stat-search/files?cycle=7&kikan=00500&layout=datalist&page=1&result_page=1&tclass1=000001020455&tclass2=000001169106&tclass3val=0&toukei=00500226&tstat=000001015623). |
| Value `0` | Below the displayed unit, rather than an exact zero. Keep it as `below_display_unit`, count it, and exclude it from numeric sums. | [MAFF symbol definitions](https://www.maff.go.jp/j/tokei/kouhyou/seika_orosi/gaiyou/) define `0` as less than the display unit and give `0.4t → 0t` as an example. |
| Value `－` | No fact / no transaction. Preserve as `no_fact`; do not convert to zero. | [MAFF symbol definitions](https://www.maff.go.jp/j/tokei/kouhyou/seika_orosi/gaiyou/). |
| Value `…` | Fact unknown or survey missing. Preserve separately as `unknown_or_uncollected`; do not convert to zero. | [MAFF symbol definitions](https://www.maff.go.jp/j/tokei/kouhyou/seika_orosi/gaiyou/). |

The displayed sum includes only numeric quantity values across the nine parent regions. It excludes below-unit, no-fact, and unknown/uncollected cells, and reports their counts so the result is visibly incomplete. It represents wholesale flows into the selected major consumer regions, not total production or a national quantity.

## Year-to-year scope check

MAFF's survey history states: “令和4年：年間取扱量等調査の調査対象を中央卸売市場における卸売会社のみに見直した” (in 2022, the annual handling survey target was revised to wholesalers in central wholesale markets only). The impact on the selected 2021–2023 origin-by-consumer-region tables has not yet been established. Show each year as its own observed series and use caution when comparing years until the scope of these specific tables is confirmed.

Source: [MAFF survey overview](https://www.maff.go.jp/j/tokei/kouhyou/seika_orosi/gaiyou/), “調査の沿革”. The year-specific tables and workbooks are linked from [`2021_2023_temporal_mapping.json`](2021_2023_temporal_mapping.json).

## Implementation notes

- Historical API rows keep their original metadata unit (`null` for 2021). The aggregation output uses the verified workbook display unit `t` for all three years.
- `0`, `－`, and `…` remain distinct in normalized rows and aggregation contributions.
- The current aggregation sums only the nine level-1 consumer regions and excludes the three nested subregions.
