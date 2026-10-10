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
