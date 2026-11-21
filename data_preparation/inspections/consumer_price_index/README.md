# Consumer Price Index inspection

## Scope

Inspect e-Stat table `0004052037`, **Consumer Price Index (2025 base)**, before choosing the production fetch and chunk layout. Initial metadata, representative full series, and source comparisons are recorded below. The production fetch should validate all page boundaries; revision monitoring remains a follow-up after a later table update.

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

The classification product is 127,196,418 measure-item-region-time combinations. Metadata reports `OVERALL_TOTAL_NUMBER=13,123,275`, while `getStatsData` reports `TOTAL_NUMBER=13,125,623`, a difference of 2,348 records. Filtering to the index measure (`tab=1`) returns 4,626,165 records; filtering that to monthly time (`lvTime=4`) returns 3,972,597. The metadata count discrepancy is retained as a caveat. The official [e-Stat API FAQ](https://www.e-stat.go.jp/api/en/api-dev/faq) says each response can contain at most 100,000 records and later pages use `startPosition` / `NEXT_KEY`; the sample pages returned continuous ranges 1–3 and 4–6.

A filtered national overall series (`tab=1`, `cat01=0001`, `area=00000`) has 681 monthly and 112 annual observations. Its October 2026 row is absent, although October exists in the time lookup; the response note defines `@char="***"` as “no applicable data.” In the representative series, missing months are omitted records and there are no explicit nonnumeric value tokens or annotation fields. A Tokyo overall observation for October is present. The API orders observations newest first, so the preparation step must order monthly time values chronologically.

Four representative series show the same regional coverage pattern for a parent item (`0002` 食料) and a leaf (`1001` うるち米Ａ):

| Item | Region | Monthly records | Missing months | Latest observation |
| --- | --- | ---: | ---: | --- |
| `0002` 食料 | `00000` 全国 | 681 | 1 (October 2026) | September 2026: 103.6 |
| `0002` 食料 | `13100` 東京都区部 | 682 | 0 | October 2026: 105.5 |
| `1001` うるち米Ａ | `00000` 全国 | 621 | 61 | September 2026: 81.4 |
| `1001` うるち米Ａ | `13100` 東京都区部 | 620 | 62 | October 2026: 75.4 |

The national overall September 2026 index (`0001`, 102.2) matches the Statistics Bureau's [national September release](https://www.stat.go.jp/data/cpi/sokuhou/tsuki/pdf/zenkoku.pdf). Tokyo food in October 2026 (`0002`, 105.5) matches the [Tokyo October release](https://www.stat.go.jp/data/cpi/sokuhou/tsuki/pdf/kubu.pdf). This also confirms that October is available in the table for Tokyo while the national series in these samples ends at September. The latest common month for a national/Tokyo comparison is September.

The first 1,000 unfiltered records with `metaGetFlg=N` occupy 170,596 bytes (about 171 bytes per source record) and gzip to 6,513 bytes. Four representative monthly series serialize to 10.6–11.7 KB each using only item/region IDs and `[YYYY-MM, value]` pairs. Extrapolating the measured average across 3,972,597 monthly index observations gives roughly 68 MB of compact JSON before gzip; treat this as an estimate until the full transform is measured. Fetching the monthly index measure in 100,000-record pages requires about 40 pages.

## Data contract and chunk proposal

- Lookups: item `{code, name, level, parentCode}` in API order; region `{code, name}`; month `{code, yearMonth}` derived from time entries with `@level=4`.
- Series identity: fixed table and measure (`tab=1`) plus `itemCode` and `areaCode`.
- Series values: ascending `[YYYY-MM, numericValue]` pairs. Do not persist annual entries or the other two measures.
- Missing observations: preserve absence in the sparse series. When charting against the selected series' union of dates, map an absent date to `null` so the line shows a gap.
- Provisional files: group ten consecutive item lookup entries per region, yielding at most `67 × ceil(797/10) = 5,360` chunks. The four sampled series average about 11 KB each; a ten-item region chunk is estimated around 13 KB on average, with dense groups expected to be larger. This keeps requests selective without creating over 53,000 tiny files. Measure actual chunk sizes during transformation before locking the layout.

The table page shows a maximum of 100,000 cells for table display and 15,000 cells for graph display. These are UI limits, not API page limits.

## API capture

The capture command requests Japanese metadata, page-boundary samples, a 1,000-record volume sample, index-measure counts, and monthly representative index series (`lvTime=4`). The original archived representative captures predate that filter and also contain annual rows; the production comparison ignores those rows for compatibility. It redacts the application ID before saving responses:

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
- `samples/index_item_{0002,1001}_area_{00000,13100}_series.json`: complete parent/leaf series for national and Tokyo regions.
- `samples/index_monthly_measure_start_1_limit_3.json`: count and sample for the monthly index measure.

All saved API responses redact the application ID. SHA-256 checksums for the captured samples:

| File | SHA-256 |
| --- | --- |
| `samples/meta_info.json` | `813c6e60f19cee7275149ad882ced84ceb2d39cf5f286b87eb40dcafd155f467` |
| `samples/stats_data_start_1_limit_3.json` | `f4d8fccf83bee58bae6c4e768bb748a391f6b04f64ad563369a11933ae947409` |
| `samples/stats_data_start_4_limit_3.json` | `63139ce1b3be7fb969b0f4bffc91f6cebaa022256316303b9c6e674d4d8f001d` |
| `samples/stats_data_start_1_limit_1000.json` | `58485cca6cc6ccc2828c388d814fd540801f159e62d9a8a32c7e76defcbefde8` |
| `samples/index_measure_start_1_limit_3.json` | `7a0d97b923b279da630a99dda13582faa2503072bfd043f31a681cda3d60b419` |
| `samples/index_monthly_measure_start_1_limit_3.json` | `8fff9c92e38f07214d068463ed6b6bcd77930b6784a95ebf7407eea21bc52c17` |
| `samples/index_national_overall_series.json` | `28db73048fa4bbbd3fd59dd6e534b7b53bcd1389ce23cb1435c79157ae913b6e` |
| `samples/index_item_0002_area_00000_series.json` | `d0bdc68356f90e77e42118e555dec886ddef5c73a6c52d24a205c48d8dc4c03a` |
| `samples/index_item_0002_area_13100_series.json` | `ecaba425f9a2aea2827f3e5dc6ba29912ef2bf5f16bd1049873228d4da2eb3b8` |
| `samples/index_item_1001_area_00000_series.json` | `96ab2f671a28d3fdc877c2fd0271678b9ce723121b480f42ced642b24ba8f32b` |
| `samples/index_item_1001_area_13100_series.json` | `634fc5b57b8c5465519e087251aab6d50bcaec23b991839751ec2e9ed9b005b1` |

## Follow-up checks during data preparation

- Keep the 2,348-record mismatch as an API metadata caveat. Validate continuous production pages against `getStatsData` `TOTAL_NUMBER` and `NEXT_KEY`.
- Re-fetch after a later table update to determine whether source values can be revised; current representative values match dated Statistics Bureau reports.

The API sample decodes the table unambiguously. The monthly series contract and chunk layout above are suitable to carry into the preparation pipeline, with actual chunk sizes measured during generation.
