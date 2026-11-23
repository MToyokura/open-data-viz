# Data pipelines by project

This inventory covers the seven dataset pipelines and the shared prefecture map used by the frontend. It describes the checked-in implementation for [Stage 1](index.md).

## Running and publishing

From the repository root, run `./data_preparation/scripts/prepare.sh run -- <command>`. The wrapper loads the root `.env` and changes into `data_preparation/` before invoking Cargo. Local paths below are relative to `data_preparation/`. Each project's output paths are relative to its named directory under `output/`, except for the social security path mismatch described below.

Run `npm run serve` from the repository root to serve `data_preparation/output/` locally. The frontend reads assets from `VITE_STORAGE_BASE_URL`, which defaults to `http://127.0.0.1:8000`. Publish one project with `./data_preparation/scripts/publish.sh --only <output-directory-name>`. The script allowlists assets, gzip-compresses them, and uploads each object to the matching relative path in GCS. Local staging and promotion apply to generation. GCS uploads happen file by file. [Publishing script](../../data_preparation/scripts/publish.sh) · [Frontend storage configuration](../../frontend/src/lib/paths.js)

## Consumer price index

- **Command and inputs.** `consumer-price-index` downloads e-Stat table `0004052037`, filtered to monthly index values. It saves the metadata response and paginated values in `raw_data/` and the acquisition record in `meta_data/acquisition.json`. `--transform-only` replays the saved responses without the API.
- **Generation and validation.** The pipeline checks page boundaries and counts, builds item, region, and month lookups, and groups observations into monthly series. Recognized missing values are omitted and unexpected nonnumeric values fail validation. The full run stages raw, acquisition, and browser outputs in `.staging/` before promotion. Offline replay stages and promotes only browser output. Promotion backs up the previous directories and attempts rollback on failure.
- **Browser and publishing.** The allowlist includes `transformed_data/{items,regions,months,manifest}.json` and `transformed_data/series/area_<area>_items_<first>_<last>.json`. The workspace and both CPI articles load item and region lookups and request chunks for the series they need. Each chunk covers up to ten consecutive items for one region.

[Pipeline](../../data_preparation/src/modules/projects/consumer_price_index/pipeline.rs) · [Browser loader](../../frontend/src/routes/consumer_price_index/utils/fetchData.js)

## Commodity prices

- **Command and inputs.** `commodity-prices` uses e-Stat table `0003421913`. It reuses `meta_data/get_commodity_meta_info.json` and per-item `raw_data/get_commodity_prices_stats_data_<item>.json` when present, and fetches missing inputs. The command still requires `ESTAT_APP_ID` and exposes no `--transform-only` mode.
- **Generation and validation.** The pipeline derives item and area lookups from metadata and splits each item's observations into per-area series. It rewrites the lookups and skips transformed series files that already exist. The shared writer replaces individual files atomically, with no project-level staging.
- **Browser and publishing.** The workspace loads `meta_data/{cat02_lookup,area_lookup}.json` and fetches `transformed_data/commodity_prices_<item>_<area>.json` on selection. The commodity prices article also loads series files for its charts. The allowlist includes both lookups and the transformed series tree.

[Pipeline](../../data_preparation/src/modules/projects/commodity_prices/) · [Series loader](../../frontend/src/routes/commodity_prices/utils/fetchCommodityPrices.js)

## Workers by age, employment type, education, and sex

- **Command and inputs.** `workers-by-age-employment-type-education-sex` downloads e-Stat table `0003006608`. It refreshes `meta_data/get_meta_info.json`, `meta_data/acquisition.json`, and paginated API responses in `raw_data/`. `--transform-only` reuses the saved inputs.
- **Generation and validation.** The pipeline checks pagination and source records, derives classifications from metadata, and groups annual values by age, employment type, education, and sex. It rebuilds `transformed_data/` in place and writes the options index after checking its referenced files. A failure during writing can leave incomplete output.
- **Browser and publishing.** The workspace loads `meta_data/options.json`, then fetches indexed series files on demand, including for comparisons. The allowlist includes the options file and `transformed_data/age-<age>_employment-<employment>_education-<education>_sex-<sex>.json` series.

[Pipeline](../../data_preparation/src/modules/projects/workers_by_age_employment_type_education_sex/pipeline.rs) · [Browser loader](../../frontend/src/routes/workers_by_age_employment_type_education_sex/utils/fetchData.js)

## Asset distribution

- **Command and inputs.** `asset-distribution` fetches e-Stat table `0004056120` and replaces the project output directory before acquisition. It saves `meta_data/get_meta_info.json`, paginated values in `raw_data/`, and `capture_manifest.json`. `--transform-only` preserves and replays the saved inputs.
- **Generation and validation.** The pipeline validates and groups values into household counts by asset band, then writes and checks a schema version 3 bundle directly in the output directory. The bundle uses contiguous ranges of attribute combinations to locate chunks. The old manifest and chunks are removed before the replacement bundle is written.
- **Browser and publishing.** The allowlist includes `manifest.json` and `items/<chunk>.json`. The workspace loads the manifest for classifications and chunk ranges, then fetches the relevant chunks for selected combinations and comparisons.

[Pipeline](../../data_preparation/src/modules/projects/asset_distribution.rs) · [Browser loader](../../frontend/src/routes/asset_distribution/utils/data.js)

## Vegetable production relay

- **Command and inputs.** `vegetable-relay` uses the 2021–2023 product inventory and verified temporal mappings under `inspections/vegetable_production_relay/`. It reuses verified e-Stat metadata and observation captures under `runs/` and acquires missing captures. The full command requires `ESTAT_APP_ID`. Separate inspection commands support capture replay, normalization, and aggregation.
- **Generation and validation.** The pipeline requires verified mappings for each product, normalizes observations by product and year, and aggregates monthly prefecture totals and underlying details. Normalization artifacts stay under `runs/product-normalization/`. Generation clears prior relay-owned outputs, writes products directly into the output directory, and rebuilds the index last.
- **Browser and publishing.** The allowlist includes `products.json`, `<product>.summary.json`, and `<product>/details/<prefecture-code>.json`. The workspace starts from the product index, loads a summary for the selected product, and fetches details on demand when a prefecture is selected. The map geometry comes from the separate prefectures output.

[Generation](../../data_preparation/src/modules/projects/vegetable_production_relay/generation.rs) · [Inspection records](../../data_preparation/inspections/vegetable_production_relay/README.md) · [Browser loader](../../frontend/src/routes/vegetable_production_relay/utils/fetchRelayData.js)

## Marine fishery catch

- **Command and inputs.** `marine-fishery-catch` downloads the national historical XLSX from e-Stat (file ID `000040416370`) without an API key. It saves `raw_data/marine_fishery_catch.xlsx` and `raw_data/capture.json` with the source URL, timestamp, and checksum. `--transform-only` reads the saved workbook.
- **Generation and validation.** The parser checks the worksheet structure, heading fingerprint, units, and source notes. It produces one JSON file with years, species series, and scoped source notes. The full run builds under `.staging/` before switching the raw and transformed directories. Offline replay stages and replaces just the generated file.
- **Browser and publishing.** The allowlist includes `transformed_data/catches.json`. The marine fishery catch workspace loads it once and uses local series selection and note filtering.

[Pipeline](../../data_preparation/src/modules/projects/marine_fishery_catch.rs) · [Source inspection](../../data_preparation/inspections/marine_fishery_catch/README.md)

## Social security budget flow

- **Command and inputs.** `social-security-budget-flow` downloads metadata and values for the 2024 e-Stat income table `0004050702` and expenditure table `0004050703`. It saves both source pairs under `raw_data/`. There is a separate inspection command for samples. The production command always reacquires the tables.
- **Generation and validation.** The pipeline checks source counts and classification codes, aligns programs across the two tables, and classifies values for display. It writes audit files and the generation manifest in the project directory. The public catalog is written to a temporary file and renamed into place.
- **Browser and publishing.** The workspace fetches `social_security_budget_flow/catalog.json` and renders the Sankey diagrams from its program-level income and expenditure arrays. The allowlist includes only the catalog. The Rust output constant currently starts with `data_preparation/output/` even though the wrapper runs Cargo from `data_preparation/`. The wrapper therefore writes to `data_preparation/data_preparation/output/social_security_budget_flow/` from the repository root, while the publisher expects `data_preparation/output/social_security_budget_flow/`.

[Entry point](../../data_preparation/src/modules/projects/social_security_budget_flow/mod.rs) · [Pipeline](../../data_preparation/src/modules/projects/social_security_budget_flow/pipeline.rs) · [Browser path](../../frontend/src/routes/social_security_budget_flow/constants.js)

## Prefecture boundaries

- **Command and inputs.** `prefectures-update` downloads the MLIT N03 2026 administrative boundaries ZIP and extracts it under `input/prefectures/N03-2026/`. It reuses the archive and extracted GeoJSON when present. Generation requires Node.js, Mapshaper, and `unzip` for extraction.
- **Generation and validation.** Mapshaper simplifies geometry, dissolves features by prefecture name (`N03_001`), and filters small islands. It overwrites the destination directly, then the Rust command validates the written GeoJSON.
- **Browser and publishing.** The allowlist includes `prefectures/N03-20260101_prefecture_simplified.geojson`. The vegetable relay workspace fetches this file for the map.

[Pipeline](../../data_preparation/src/modules/prefectures.rs) · [Browser path](../../frontend/src/routes/vegetable_production_relay/constants/constants.js)
