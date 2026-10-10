# Vegetable relay API inspections

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


## M3 extension: repeated-region semantic evidence (still blocked)

For 2021 (`0002008540`), 2022 (`0002120386`) and 2023 (`0002114330`), use the corresponding **replay-verified** metadata run directory:

```sh
cargo run -- vegetable-relay-semantic-blocks inspections/vegetable_production_relay/runs/metadata-<run-id>
export ESTAT_APP_ID='your e-Stat application ID'
cargo run -- vegetable-relay-semantic-probe inspections/vegetable_production_relay/runs/metadata-<run-id> 1001
cargo run -- vegetable-relay-semantic-replay inspections/vegetable_production_relay/runs/semantic-probe-<run-id> inspections/vegetable_production_relay/runs/metadata-<run-id>
```

`semantic_blocks.md` records exact source-code boundaries for the 13 repeated 12-label blocks **without assigning month names**. The probe explicitly filters one valid `cat03` code and requests at most 10 `getStatsData` rows. It stores exact final bytes, checksum, and sanitized request metadata, and can be verified offline against the same metadata capture. `sample.json` retains raw observation references and lexical values only; it is inspection evidence, **not a normalized dataset**. Failures leave evidence in ignored run directories; source values may be sensitive and must be sanitized before committing. Month-to-block correspondence, total-block identity, geographic coverage and unit comparability remain unverified. Limited samples cannot prove any of them. No candidate selection or M4 data publication occurs.
