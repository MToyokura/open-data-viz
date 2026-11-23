# Commodity prices pipeline refactor

Migrate Commodity prices first in [Stage 1](index.md). Use the [CPI pipeline](../../data_preparation/src/modules/projects/consumer_price_index/pipeline.rs) as the reference for explicit acquisition, offline replay, and staged promotion. Keep the Commodity prices browser contract stable so both existing routes can use the regenerated data.

## 1. Capture and verify source inputs

- Keep `./data_preparation/scripts/prepare.sh run -- commodity-prices` as the full refresh command. Require `ESTAT_APP_ID` and fetch metadata and observations for e-Stat table `0003421913` into a fresh staging directory.
- Derive the item list from the captured metadata. Fetch every requested item's observations, following pagination where the API indicates more pages. Check page boundaries, item identity, and observed counts against the API response. Record verified zero-row responses explicitly so replay can distinguish them from missing items.
- Save the metadata response and per-item response pages under `output/commodity_prices/raw_data/` in the promoted capture. Write `meta_data/acquisition.json` with the table ID, acquisition time, and per-item page paths, row counts, and checksums covering all saved inputs, including metadata. Keep credentials out of the capture and diagnostics.
- Add `commodity-prices --transform-only` to replay the last verified capture without network access or an API key. Fail with instructions to run a full refresh if the capture is missing, incomplete, or predates the acquisition record.

## 2. Rebuild and promote the browser bundle

- Use one transformation path for fresh captures and offline replay. Verify the capture record and input files, derive both lookups from metadata, and group observations by item and area. Write a complete, deterministic bundle from scratch, including stable lookup and observation order. Preserve the existing treatment of source values and missing observations.
- Preserve the existing JSON field names, value types, and browser paths under `commodity_prices/`:
  - `meta_data/cat02_lookup.json`
  - `meta_data/area_lookup.json`
  - `transformed_data/commodity_prices_<item>_<area>.json`
- Before promotion, check that item and area codes resolve through the lookups, each emitted observation is unique for its item, area, and time, and the files account for all observations expected under the existing transformation rules. Check that filenames and document contents agree and that the staged output contains no obsolete series.
- Stage acquisition and browser output under `output/commodity_prices/.staging/`. A full refresh promotes the verified capture and the matching browser bundle together, with rollback if promotion fails. Offline replay replaces the browser bundle while preserving the verified capture. Keep the previous active outputs available during build and validation.

## 3. Verify and release

1. Run the Rust tests. Replay the same verified capture twice and compare the browser output byte for byte. Check that offline replay works without `ESTAT_APP_ID`, that rebuilding removes stale series, and that a missing or corrupt input causes a failure while the previous browser bundle still serves.
2. Serve the regenerated output locally. In `/commodity_prices`, load the item and area selectors and select series that fetch data on demand. In `/articles/commodity_prices`, verify the price charts against the regenerated assets. Run the browser baseline as well.
3. Confirm that the publish allowlist contains both lookups and the complete series tree and excludes the capture and acquisition metadata. Retain a validated copy of the currently published output for rollback.
4. If old and new lookups and series can safely mix during upload and cache expiry, publish in place using `./data_preparation/scripts/publish.sh --only commodity_prices`. If the files must switch together, publish the complete bundle to a versioned prefix in the existing bucket and verify a release build against that prefix before deploying the loader change.
5. Verify both routes in production, including series selection and article charts. Record the published prefix and verification result. If verification fails, republish the previous bundle for an in-place update or redeploy the previous frontend for a versioned release.

Update the [data preparation README](../../data_preparation/README.md) with both command modes and the capture layout as part of implementation. See the [pipeline inventory](pipeline_by_project.md#commodity-prices) for the current behavior.
