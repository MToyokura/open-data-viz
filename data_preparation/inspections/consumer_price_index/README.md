# Consumer Price Index inspection

## Scope

Inspect e-Stat table `0004052037`, **Consumer Price Index (2025 base)**, before choosing the production fetch and chunk layout. Initial metadata and sample captures are recorded below; full pagination, broader special-value checks, revisions, and final output sizing remain open.

## Initial findings

The [source table](https://www.e-stat.go.jp/dbview?sid=0004052037) identifies the table as `消費者物価指数（2025年基準）`. The page reports a publication timestamp of 2026-10-30 08:30. API metadata updated on 2026-10-30 confirms the title and these four classifications:

| Classification | Entries shown |
| --- | ---: |
| Measure (`表章項目`) | 3 |
| Item (`2025年基準品目`) | 797 |
| Region (`地域（2025年基準）`) | 67 |
| Time (`時間軸（年・月）`) | 794 |

The measure axis is `tab`: code `1` is `指数`, `2` is `前月比・前年比・前年度比`, and `3` is `前年同月比`. Item, region, and time axes are `cat01`, `area`, and `time`. The initial item codes include `0001` (総合), `0002` (食料), and `1001` (うるち米Ａ). Item entries preserve API order and include `@level` and `@parentCode`; 751 of 797 items have a parent code. Region entries have level 1 and no parent links.

The time axis contains 682 monthly entries (level 4) and 112 annual entries (level 1). Monthly codes are not chronological in API order; for example, the first is `2026001010` (`2026年10月`) and later entries descend by month. Monthly observations should be selected using the time classification's `@level`, with display dates taken from the corresponding time lookup entry.

The classification product is 127,196,418 measure-item-region-time combinations. Metadata reports `OVERALL_TOTAL_NUMBER=13,123,275`, while `getStatsData` reports `TOTAL_NUMBER=13,125,623`, a difference of 2,348 records. Filtering to the initial index measure (`tab=1`) returns 4,626,165 records. Treat the all-measure count as provisional until a complete paginated scan establishes a consistent total. The official [e-Stat API FAQ](https://www.e-stat.go.jp/api/en/api-dev/faq) says each response can contain at most 100,000 records and later pages use `startPosition` / `NEXT_KEY`.

A filtered sample for `tab=1`, `cat01=0001`, `area=00000` returned 793 records: 681 monthly and 112 annual. The national overall index has no record for `2026年10月`, despite that month being present in the time classification; the matching absence note is `@char="***"` (“該当データがない場合を示す。”). A few recent values are 102.2 for August and September 2026. The sample contains no explicit special value token or annotation field. The unfiltered first page is ordered with Tokyo (`area=13100`), overall (`cat01=0001`), index (`tab=1`), and latest month first. Pagination with `startPosition=1, limit=3` and `startPosition=4, limit=3` returned continuous ranges 1–3 and 4–6.

The first 1,000 unfiltered records with `metaGetFlg=N` occupy 170,596 bytes (about 171 bytes per source record) and gzip to 6,513 bytes. A naive extrapolation to 13.1 million records is about 2.2 GB of uncompressed source JSON; the gzip ratio and selected-measure output size need a broader sample before treating any estimate as a delivery target. The sample covers one item and two regions, so it primarily measures response shape and repeated fields.

The table page shows a maximum of 100,000 cells for table display and 15,000 cells for graph display. These are UI limits, not API page limits.

## API capture

The capture command requests Japanese metadata, page-boundary samples, a 1,000-record volume sample, the index-measure record count, and a complete national overall index series. It redacts the application ID before saving responses:

```sh
cd data_preparation
./scripts/prepare.sh run -- consumer-price-index-inspect
```

Captured files:

- `capture_manifest.json`: capture time, endpoint URLs, request parameters, and sample paths.
- `samples/meta_info.json`: table and classification metadata.
- `samples/stats_data_start_1_limit_3.json` and `samples/stats_data_start_4_limit_3.json`: small page-boundary samples.
- `samples/stats_data_start_1_limit_1000.json`: first 1,000 unfiltered records for response-size measurement.
- `samples/index_measure_start_1_limit_3.json`: index-measure total and first records.
- `samples/index_national_overall_series.json`: complete national overall index series for missing-value inspection.

All saved API responses redact the application ID. SHA-256 checksums for the captured samples:

| File | SHA-256 |
| --- | --- |
| `samples/meta_info.json` | `04947f1bc82ddaa07306c5851c71786cf96c953584bf16cbd4db181977729e34` |
| `samples/stats_data_start_1_limit_3.json` | `d8db3e7a3b2dbb61caaecc8d6b10c9f0eba8ea6106fbbedee881c8d82bab4ce3` |
| `samples/stats_data_start_4_limit_3.json` | `c903c21699b635718fadb00099ff13440266c56a62d038799a7c221598275fef` |
| `samples/stats_data_start_1_limit_1000.json` | `e0aaff16b86320d2148d0ac65d18696b1dbbd1158490a456a844b27e98992ecb` |
| `samples/index_measure_start_1_limit_3.json` | `3f5ca3b827f01f765281fd7dbbe26e169b4258ddc9d9d251b9e0d61803fcd256` |
| `samples/index_national_overall_series.json` | `7be61a1a9d70ced6938c4e1e116d331a1654f7d44754a400e41ffdbffef23e5c` |

## Open questions before the data contract checkpoint

- Find the source of the 2,348-record difference between metadata `OVERALL_TOTAL_NUMBER` and response `TOTAL_NUMBER`; confirm full-table page boundaries with a scan.
- Check special tokens and annotations across representative category/region series, including a leaf item and a non-national region. The current overall national sample contains only numeric values and has no `***` row; its `NOTE` defines `***` as no applicable data.
- Confirm revisions by re-fetching representative records after a later table update, or comparing against a dated official table export where available.
- Measure output size for the selected index measure and more than one item/region; use it to select a chunk layout that can fetch a selected item/region without downloading the full table.
- The existing metadata covers item hierarchy and source order; add targeted child-item, leaf-item, and non-national-region data samples with exact query parameters.
- Compare representative observations against the e-Stat table view. Re-fetch after a later table update to determine whether source values can be revised.

The table exposes only the four axes above, so a series is identified by measure, item, and region; the table ID is fixed by the dataset. The frontend contract and chunk layout remain undecided pending the remaining checks. Keep absent records distinct from explicit special values when representing missing observations.
