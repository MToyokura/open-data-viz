# Marine fishery catch visualization

Add a Japanese-language page at `/marine_fishery_catch` for exploring Japan’s annual marine fishery catch by species and category. Use the national table 海面漁業魚種別漁獲量累年統計（全国） from 漁業・養殖業生産統計年報. [Download the e-Stat workbook](https://www.e-stat.go.jp/stat-search/file-download?statInfId=000040416370&fileKind=0).

## Workbook inspection

`data_preparation/inspections/` already has a precedent for inspecting official Excel workbooks in `vegetable_production_relay/`. The inspection index and directories currently have no fishery entry. The findings below come from reading the downloaded workbook’s worksheet XML and shared strings.

| Location | Finding |
| --- | --- |
| `Sheet1!A4` | National coverage (全国). |
| `Sheet1!A5:DH8` | Rows 5–7 describe groups, totals, subtotals and individual series. Row 8 gives the unit `t` for all 111 series in B–DH. |
| `Sheet1!A9:DH77` | 69 consecutive annual rows, 1956–2024. Gregorian years appear in parentheses, including era-transition labels and labels with trailing kana, such as `令.元(2019)レイモト`. |
| `Sheet1!B79:B107` | A notes heading followed by 28 numbered notes on rounding, historical coverage, unit conversion and changes in species groupings. Note 28 records missing survey data in Iwate, Miyagi and Fukushima after the 2011 earthquake. |

Of 7,659 catch cells, 1,114 contain the text marker `… ` with trailing whitespace. The current snapshot has no numeric zeros or blank catch cells. As reference points, the total is 4,511,923 t in 1956 (`B9`) and 2,786,195 t in 2024 (`B77`). さんま is 327,813 t in 1956 (`AH9`) and 39,335 t in 2024 (`AH77`). `F9` (みなみまぐろ) contains the marker. The inspected workbook is 74,858 bytes with SHA-256 `0e057ae6dc8476837cf564f10a8039921ef9b56238d4b81e2103316bc9aced8e`.

## 1. Download and transform in data_preparation

- Register a Rust `marine-fishery-catch` command in the existing dispatcher. From `data_preparation/`, run `./scripts/prepare.sh run -- marine-fishery-catch` to fetch the XLSX with the existing blocking HTTP client and generate JSON. This public file endpoint requires no e-Stat API key. Support `--transform-only` to regenerate from the saved workbook.
- Store the original under `output/marine_fishery_catch/raw_data/` with a small capture record containing the source URL, retrieval timestamp and SHA-256. Stage downloads and transform outputs. Check that the response is a readable XLSX with the expected table structure before replacing the saved workbook, and replace the public JSON only after validation succeeds.
- Use an XLSX reader such as Calamine. Verify the sheet, heading rows and units, then locate and parse the year block using the parenthesized Gregorian year in column A. Validate that years are unique and ascending. Decode B–DH using a reviewed mapping of spreadsheet column to stable ID, display label, group path and series type (total, subtotal or detail). Check the source headings against the mapping so column drift fails visibly. Keep the source’s values for all published aggregates because rounding and overlapping definitions can make sums differ.
- Preserve numeric zero. Trim the ellipsis marker and emit `null` for it. Report unexpected blank or nonnumeric cells with coordinates. Keep the source notes, and index affected series and years where the notes identify them. Review Japanese display labels and notes with trailing kana against the workbook.
- Write one versioned `output/marine_fishery_catch/transformed_data/catches.json` with source metadata, unit, ordered years, notes, and the series catalog with ordered numeric or null values aligned to the year array. The current table fits in one gzip-compressed fetch. Add `data_preparation/inspections/marine_fishery_catch/README.md` and a link from the inspections index with the capture details, column mapping decisions, marker handling and reproducible checks.

## 2. Build the visualization page

- Add the SvelteKit route under `frontend/src/routes/marine_fishery_catch/` and link it from the home page’s 食と農業 section using `VITE_ROOT_PATH`. Fetch the generated JSON from `VITE_STORAGE_BASE_URL` and use the existing workspace, load-state and reference components where applicable.
- Open with a line chart of the national total (漁獲量計). Provide a searchable series selector grouped by the workbook hierarchy and let users compare a few series on the same tonne axis. Switch to the first selected species when the user leaves the total view so the total’s scale does not flatten the initial species chart. Show the year and exact tonnes for the selected series in chart tooltips. Break lines at `null` values and label those entries `データなし`.
- Place the source link, year coverage and notes next to the chart. Show the 2011 coverage caveat and surface relevant definition changes for selected series, such as くろまぐろ including みなみまぐろ through 1994. Use the source series as published across definition changes and keep totals and subtotals individually selectable.

## 3. Publish and verify

- Add the generated JSON to the allowlist in `data_preparation/scripts/publish.sh` and document generation and publishing in `data_preparation/README.md` and `data_preparation/PUBLISHING.md`. Publish the JSON to the `marine_fishery_catch/transformed_data/` GCS path before deploying the GitHub Pages route.
- Verify the reference cells above, the 69 years, the 111 series and the 1,114 nulls against this workbook snapshot. Cover era transitions, hierarchical headings, footnote exclusion, synthetic zero handling, and a failed validation that leaves the previous output usable. Check that download-and-transform and `--transform-only` produce equivalent series data from the same source bytes.
- Run the frontend checks and exercise the page at desktop and mobile widths. Verify keyboard navigation, series changes, a visible missing-data gap, source notes, and loading/error states. After publishing, verify the deployed route can fetch the gzip JSON from the configured storage URL.
