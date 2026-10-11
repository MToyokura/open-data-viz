# 2024 daikon: M4 replay and M3 semantic check

Checked 2026-10-11. Scope: e-Stat table `0004044496`, the 2024 daikon table in the [official dataset listing](https://www.e-stat.go.jp/stat-search/files?cycle=7&layout=datalist&page=1&stat_infid=000040317746&tclass1=000001020455&tclass2=000001232266&tclass3val=0&toukei=00500226&tstat=000001015623). This is an inspection record, not a normalized or publishable dataset. Statistical mapping approval remains pending.

## M4 replay result

- Raw capture: `runs/observations-1791678455-860323437-165827/`.
- Matching metadata: `runs/metadata-1791675366-450693749-111423/`.
- Metadata SHA-256 bound in the ledger: `db04e07721badf811b429bb82a6ffee6ac151a6727a18f1d9029da909c0b90ef`.
- Command: `cargo run --quiet -- vegetable-relay-acquisition-replay <raw capture> <matching metadata>` (paths above).
- Result: `Verified 14976 raw observations; no semantic approval` (exit status 0). Replay checked page bytes and SHA-256 digests, consecutive page intervals, API totals, and metadata code references. The ledger reports 15 pages and 14,976 observations; the replayed count agrees.
- Structural coverage: every cell in the `48 cat01 × 2 cat02 × 12 cat03 × 13 time` grid appears exactly once: 14,976 distinct coordinates, no duplicates. This establishes table-grid coverage, not that every cell contains a measured quantity.

## Source-code interpretations for the 2024 table only

The checked source is the [2024 *利用者のために* guide](https://www.e-stat.go.jp/stat-search/file-download?fileKind=2&statInfId=000040317745). It describes monthly flow from origin prefectures to major consumer regions, defines quantity and price, lists the regions and their constituent cities, and explains table symbols. The exact code-to-label bindings below come from the replayed metadata inventory. The codebook in `2024_daikon_codebook.json` retains all 47 prefecture codes and all other axis codes.

| Axis | Source codes and interpretation | Processing consequence |
|---|---|---|
| `time` | `2024000101` through `2024001212` are individually labelled January through December 2024 in the metadata. `10` is labelled `計`. | Use the twelve explicit month codes for monthly records. Treat `10` as a summary and exclude it from monthly aggregation; reconciliation of that summary with monthly values remains to be checked. |
| `cat01` | `1001` is `計`; `1002`–`1048` name 47 prefectures in standard order. The guide defines reported origin as the prefecture where the consignor resided when goods arrived at the market. | Use the 47 explicit prefecture codes as origin categories. Exclude `1001` from prefecture aggregation. Do not describe this as production location. |
| `cat02` | `1001` is `数量` with metadata unit `t`; `1002` is `価格` with unit `円/kg`. All 7,488 quantity rows have `t`; all 7,488 price rows have `円/kg`. | Select `1001` for quantity and retain tonnes. Do not sum prices. |
| `cat03` | Nine level-1 major consumer regions: `1001`, `1002`, `1004`, `1006`–`1011`. Three level-2 subsets: `1003` (仙台・盛岡 within `1002`), `1005` (京浜 within `1004`), `1012` (北九州 within `1011`). The guide names the constituent cities and confirms those three subsets sit inside the larger regions. | The nine level-1 regions are the candidate disjoint region set. Exclude the three subsets from any sum that also includes their parent. This table describes *major consumer regions*, so a sum of them must not be called national production or national wholesale quantity. |

The guide defines a displayed `0` as a nonzero amount rounded below the display unit, `－` as no fact, and `…` as unknown or uncollected. It also states that rounded components need not sum exactly to displayed totals. The raw API uses ASCII `-` for the no-fact symbol. Across the capture, raw values contain numeric strings, `-`, and `…`; quantity rows contain no `…`. These states must remain distinct in M5. The guide also cautions that its origin prefecture can differ from the harvest or shipment prefecture in production statistics.

## Coverage and remaining gates

- Each of the 13 time codes has 1,152 cells: 48 origins × 2 measures × 12 regions. Each month has 576 quantity cells, of which 153–289 have numeric values and the others have `-`. A numeric `0` is present in every month and represents below-unit precision under the guide.
- Each month has all `47 prefectures × 9 level-1 regions` quantity coordinates. Some are `-`; their presence establishes structural coverage only.
- No coordinate is duplicated, and observed units agree with the metadata. This does **not** validate additive completeness, the meaning of `計` as a sum of months, or a derived national total.
- Before M5/M6 publication: review this mapping independently; reconcile sample cells and region totals with the official spreadsheet; define a policy for rounded zero and no-fact cells; check quantity additivity within the chosen nine regions and against the source summary where appropriate. Keep lineage and rounding differences visible.
- The 2021–2023 repeated `cat03` blocks have no approved month mapping. Nothing in this check changes their publication gate.
