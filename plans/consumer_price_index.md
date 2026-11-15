# Consumer Price Index

## Goal

Add a page for exploring Japan's CPI over time, following the comparison workflow in `/commodity_prices`.

## Data

- Source: [e-Stat table 0004052037](https://www.e-stat.go.jp/dbview?sid=0004052037), CPI (2025 base).
- Prepare data with the Rust tools in `data_preparation/src/modules/`: reuse the shared e-Stat API modules in `estat_api/` and add CPI-specific processing under `projects/consumer_price_index/`, following `projects/commodity_prices/` where useful.
- Write prepared data and item/region lookups under `data_preparation/output/consumer_price_index/`.
- Keep frontend-facing JSON minimal, following `commodity_prices`: store only the selected index measure's time-value observations and series identifiers in each series payload; keep display names in lookups and raw API responses separate for reproducible transformations.
- Start with the **index** measure. Keep percentage-change measures out of the initial chart so their different meaning is explicit in a later iteration.
- Treat the observations as index values, not yen prices; show the base period and source in the page reference panel.
- Add an optional frontend-only normalized view; do not create or persist additional normalized data.
- Normalize to 100 at the latest time period shared by all selected series: `normalized = value / reference-period value * 100`.
- Let users switch between the source index and normalized view. Explain that normalized values show relative change, not prices in yen.

## Implementation steps

### 1. Inspect the source

Work in `data_preparation/inspections/consumer_price_index/`.

**Progress:** API and source-report checks confirmed the four axes, codes, item hierarchy, monthly time level, and series behavior for a parent and leaf item across national and Tokyo regions. Monthly index records total 3,972,597; the measured compact JSON estimate is about 68 MB. A provisional region-by-ten-item chunk layout yields about 5,360 files, averaging roughly 13 KB. Representative API values match Statistics Bureau releases. The API metadata record count differs from `getStatsData` by 2,348, which is documented for the preparation fetch to validate against `TOTAL_NUMBER` and `NEXT_KEY`. See the [inspection record](../data_preparation/inspections/consumer_price_index/README.md).

- [x] Retrieve the e-Stat table metadata and a small API sample for the index measure.
- [x] Record the codes and hierarchy for items, regions, measures, and time; confirm the codes that identify monthly observations and the index measure.
- [x] Check missing rows, the `***` no-data note, and latest available month across representative item/region series.
- [x] Confirm API response limits and sample page boundaries; record the metadata/data total-count discrepancy for validation during the full fetch.
- [ ] Re-fetch representative values after a later table update to check for revisions.
- [x] Save reproducible request details and small raw samples with a README describing findings and caveats.
- [x] Define the frontend data contract and a provisional region-by-ten-item chunk layout from the measured data volume.

- [x] **Checkpoint:** The sample decodes unambiguously, and the provisional chunks are small enough to load selected item/region data without fetching the whole table.

### 2. Prepare data

Implement under `data_preparation/src/modules/projects/consumer_price_index/`, reusing shared `estat_api/` and IO modules. Follow `projects/commodity_prices/` where useful.

- [x] Add the project module, table/output constants, and wire it into the existing preparation command structure.
- [x] Fetch metadata and index observations through the shared e-Stat API tools; retain reusable raw responses under `output/consumer_price_index/raw_data/`.
- [x] Transform the confirmed index observations into the agreed lookup and series JSON files under `output/consumer_price_index/`, preserving time order and missing-value meaning.
- [x] Keep each frontend series payload limited to its item/region identifiers and ordered time-value pairs; exclude unused measures and API metadata.
- [ ] Verify representative transformed time-value observations against the saved raw API samples.
- [x] Support rerunning transformation from saved raw responses so data-shape fixes do not require refetching.
- [x] Document the command and output layout in `data_preparation/README.md`.

- [ ] **Checkpoint:** The preparation command produces valid output for representative items and regions, and the generated files match the frontend contract.

**Progress:** The production command now fetches the Japanese metadata and monthly index pages, validates page totals and `NEXT_KEY`, saves redacted raw responses, and can rebuild output with `--transform-only`. The transform creates item/region/month lookups and region-by-ten-item series chunks, validates numeric values and observation counts, and includes comparisons against the four saved representative series. `cargo check` succeeds. The full acquisition and generated-output checkpoint have not yet been run.

### 3. Build the frontend

Implement `/consumer_price_index` using the shared page patterns and chart behavior from `/commodity_prices`.

- [ ] Add constants and fetch utilities for the prepared data; load lookups first, then request only selected series.
- [ ] Build a hierarchical item selector, region selector, selected-series list, loading/error/empty states, and reset/remove behavior.
- [ ] Add a line chart with ordered dates and clear CPI index labels; handle missing observations without drawing misleading values.
- [ ] Add a source-index / normalized-view toggle. In normalized view, use the latest time period with a valid value in every selected series as the common reference and calculate `value / reference value * 100` in the frontend. Return an informative empty state if the selected series have no shared reference period.
- [ ] Display the normalization reference month and update it whenever selection changes recalculate the common reference period.
- [ ] Add a reference panel that explains the 2025 base, links to e-Stat, and clarifies that normalized values describe relative changes rather than yen prices.
- [ ] Link the page from the home page and add its route to the frontend page documentation.

- [ ] **Checkpoint:** Users can compare item/region series in both views; changing selections recalculates the normalized values from the same raw observations without writing normalized data.

## Initial scope

- Compare selected items within one region, or the same item across regions.
- Keep item categories navigable in the selector; the source table has a large item hierarchy.
- In normalized view, use one common reference period for all selected series so their changes are directly comparable.
- Defer percentage-change views, custom rebasing, and other CPI tables.

## Done when

- [ ] Prepared CPI data can be served locally and the page loads it through the normal app data path.
- [ ] Users can select and compare valid series, and the chart handles missing observations.
- [ ] The frontend can switch between source index values and values normalized to 100 at the latest common period.
- [ ] The page clearly distinguishes CPI index values from retail prices and links to the source table.
