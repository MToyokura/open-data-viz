# Vegetable relay API inspections

## Current status (2026-10-11)

The next milestone is **2021–2023 semantic review and aggregation verification**. Only daikon monthly flows for those three years are in display scope. Historical expansion and 2024-only validation/demo work are out of scope.

The **2021–2023 temporal mapping is now evidenced** for tables `0002008540`, `0002120386` and `0002114330`: official e-Stat spreadsheets labelled `計` and January–December each uniquely match one `cat03` block across all 1,152 cells. Block 1 (`1001`–`1012`) is the annual `計`; blocks 2–13 (`1013`–`1156`) are January–December in order. See [`2021_2023_temporal_evidence.md`](2021_2023_temporal_evidence.md), [`2021_2023_temporal_mapping.json`](2021_2023_temporal_mapping.json) and [`verify_temporal_mapping.py`](verify_temporal_mapping.py).

Independent reviewer approval and the remaining origin, region, unit, symbol, completeness and cross-year checks are pending. **Monthly normalization and publication remain blocked.** The handoff below records what was known before the dated spreadsheet comparison; its unresolved-month statements are historical.

## M4: complete raw observation capture (JSON source format)

From `data_preparation/`, with a **replay-verified** M3 metadata capture:

```sh
export ESTAT_APP_ID='your e-Stat application ID'
cargo run -- vegetable-relay-acquire inspections/vegetable_production_relay/runs/metadata-<run-id>
cargo run -- vegetable-relay-acquisition-replay inspections/vegetable_production_relay/runs/observations-<run-id> inspections/vegetable_production_relay/runs/metadata-<run-id>
```

Captures every paginated `getStatsData` JSON response as exact bytes with SHA-256, page positions, returned totals, timing, and sanitized ledger metadata. The ledger is written only after all pages pass basic validation. Offline replay binds the capture to the metadata SHA-256, rejects missing, duplicated or inconsistent pages, and preserves lexical values (including `-`), source attribute codes and units. `source_observations.json` is a **raw evidence export only**, never a normalized or publishable dataset. No month/total inference is made for 2021–2023, and no cross-family code join is performed. The M4 plan's `getSimpleStatsData` CSV encoding/schema remains to be separately verified; this implementation uses the JSON API already exercised in M3. All run files remain Git-ignored.

From `data_preparation/`, run:

```sh
export ESTAT_APP_ID='your e-Stat application ID'
cargo run -- vegetable-relay-inspect
cargo run -- vegetable-relay-replay inspections/vegetable_production_relay/runs/<run-id>
```

The first command captures one `getStatsList` response (maximum 10 table entries) as exact bytes in `response.bin`, with a SHA-256 checksum, sanitized request parameters, timing, HTTP status, and attempt count in `metadata.json`. It does not interpret or select statistical tables, perform aggregation, or publish any dataset.

A non-2xx response is preserved and the command exits unsuccessfully. Replay verifies the captured bytes and checksum without network access or an application ID, and likewise fails for captured non-2xx statuses.

Inspection runs may contain API-derived data and are intentionally ignored by Git. Review and sanitize any fixture before committing it. `ESTAT_APP_ID` must never appear in logs or captured files. Transient retries are capped at three attempts; only the final response is captured, not intermediate failed attempts. Network errors are recorded without their potentially credential-bearing URLs.

## M2-A: complete catalogue discovery

```sh
export ESTAT_APP_ID='your e-Stat application ID'
cargo run -- vegetable-relay-discover
cargo run -- vegetable-relay-discovery-replay inspections/vegetable_production_relay/runs/catalogue-<run-id>
```

This captures every `getStatsList` page for statistics code `00500226` (100 records per request), including raw bytes and per-page checksums. Successful inspection writes a validated `catalogue.json` containing table IDs, titles, and basic metadata. Replay verifies all captured pages offline and rejects gaps, duplicates, checksum mismatches, changed totals, and API errors. Selection of statistical tables is explicitly out of scope. On failed inspection, captured pages remain in the ignored run directory for diagnosis; no validated catalogue is published.

## M2-B: offline daikon candidate assessment

After successfully running discovery and its offline replay:

```sh
cargo run -- vegetable-relay-assess inspections/vegetable_production_relay/runs/catalogue-<run-id>
```

The command replays the raw page captures and compares the saved catalogue against replayed evidence, then creates `daikon_assessment.json` and `daikon_assessment.md` in the ignored run directory. Existing reports are not overwritten. It only classifies explicit daikon title patterns and groups records by survey-year hints. It **does not** infer temporal, geographic, origin, or measurement dimensions from titles, and **selects no tables**. All candidates remain pending `getMetaInfo` verification. Review findings before committing any sanitized report.

## M3: metadata inventory (not semantic approval)

After examining the M2 assessment, supply an **explicit** candidate table ID.

```sh
export ESTAT_APP_ID='your e-Stat application ID'
cargo run -- vegetable-relay-metadata-inspect <statsDataId>
cargo run -- vegetable-relay-metadata-replay inspections/vegetable_production_relay/runs/metadata-<run-id>
cargo run -- vegetable-relay-metadata-report inspections/vegetable_production_relay/runs/metadata-<run-id>
```

The inspection preserves exact `getMetaInfo` response bytes in `response.bin`, sanitized `capture.json`, and an `inventory.json` / `inventory.md` retaining source classification codes, names, levels, parent references, and source units where present. Replay validates the response checksum offline. Reports never overwrite existing files. Treat the inventory as **unverified evidence**, not an origin/prefecture/market/time mapping. The M3 gate remains blocked until documented interpretations are reviewed and checked against real captured metadata. Do not publish statistical observations from this inspection.

## M3 extension: repeated-region semantic evidence (historical investigation)

For 2021 (`0002008540`), 2022 (`0002120386`) and 2023 (`0002114330`), use the corresponding **replay-verified** metadata run directory:

```sh
cargo run -- vegetable-relay-semantic-blocks inspections/vegetable_production_relay/runs/metadata-<run-id>
export ESTAT_APP_ID='your e-Stat application ID'
cargo run -- vegetable-relay-semantic-probe inspections/vegetable_production_relay/runs/metadata-<run-id> 1001
cargo run -- vegetable-relay-semantic-replay inspections/vegetable_production_relay/runs/semantic-probe-<run-id> inspections/vegetable_production_relay/runs/metadata-<run-id>
```

`semantic_blocks.md` records exact source-code boundaries for the 13 repeated 12-label blocks **without assigning month names**. The probe explicitly filters one valid `cat03` code and requests at most 10 `getStatsData` rows. It stores exact final bytes, checksum, and sanitized request metadata, and can be verified offline against the same metadata capture. `sample.json` retains raw observation references and lexical values only; it is inspection evidence, **not a normalized dataset**. Failures leave evidence in ignored run directories; source values may be sensitive and must be sanitized before committing. Month-to-block correspondence, total-block identity, geographic coverage and unit comparability remain unverified. Limited samples cannot prove any of them. No candidate selection or M4 data publication occurs.

## M3 handoff decision: proceed to M4 with unresolved temporal semantics (historical)

**Decision (2026-10-11):** M3 inspection tooling and representative evidence collection are complete enough to begin M4 acquisition work. This is a **scoped engineering handoff**, not approval of every statistical interpretation. Older M3 command output and reports that say `Gate: BLOCKED` remain valid: **semantic approval for the ambiguous 2021–2023 blocks is still blocked**. Do not reinterpret that message as a failed capture/replay, or treat this handoff as a data-quality pass.

Evidence gathered:

- **2021–2023:** Consolidated candidate tables `0002008540`, `0002120386`, and `0002114330` each expose 156 `cat03` codes: 13 consecutive repetitions of the same 12 region labels and hierarchy levels, without an explicit `time` axis. The 2023 `TABLE_INF.SURVEY_DATE` is `202301-202312`, and its title is `計_1月～12月`; these establish overall coverage only, **not** a month or total for any individual block.
- **2023 API checks:** `getMetaInfo` and a bounded `getStatsData` probe succeeded. The observation capture replayed successfully against the matching metadata capture. The probed observations had classification references, units, and lexical values (including `-`); they carried no independent month mapping. An explicit `getMetaInfo` request with `explanationGetFlg=Y` returned an empty table description and no classification-object descriptions or extra attributes in the inspected fields. The e-Stat display and exported workbook likewise did not identify the 13 blocks by month.

### M4 requirements: fail closed on unresolved meaning

1. M4 may **acquire and replay raw observations** for these tables with exact source table IDs, classification codes, lexical values (including nonnumeric markers), response checksums, and provenance. Acquisition is **not** monthly normalization or publication.
2. For 2021–2023, represent the 13 blocks by their **source `cat03` codes / block ordinals only**. Keep `reporting_month` and annual-total identity **unknown**; never derive them from block order, repeated labels, `SURVEY_DATE`, titles, or arithmetic patterns (`156 = 13 × 12`).
3. Do not infer additional years from similar table shapes. Only 2021–2023 are in scope.
4. Do not join codes across table families by raw identifier alone (for example, pre-2021 `001` versus later `1001`), and do not infer geography, inclusion/exclusion rules, unit equivalence, or measure comparability solely from similar labels. Preserve original units and missing/suppression markers; `-` is **not** zero.
5. Block downstream **monthly normalization and publication of 2021–2023 data** until independent, citable source evidence establishes a complete `cat03` block-to-month (and total) mapping and a reviewer approves it. Keep the source reference, mapping rationale, and validation results alongside the decision. An unverified mapping must never silently become a default.

**Status:** M3 capture/replay and structural investigation: **complete for representative tables**. M3 cross-year semantic verification: **partially unresolved**. M4 raw acquisition: **permitted under these constraints**. Monthly publication of 2021–2023 observations: **blocked**.
