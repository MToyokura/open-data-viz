# Commodity prices pipeline refactor

Migrate Commodity prices first in [Stage 1](index.md). Use the [CPI pipeline](../../data_preparation/src/modules/projects/consumer_price_index/pipeline.rs) as the reference for explicit acquisition, offline replay, and staged promotion. Reduce the number of published objects by grouping series into CPI-style chunks. Preserve the existing series fields and values so both routes can keep their chart behavior.

## 1. Capture and verify source inputs

- Keep `./data_preparation/scripts/prepare.sh run -- commodity-prices` as the full refresh command. Require `ESTAT_APP_ID` and fetch metadata and observations for e-Stat table `0003421913` into a fresh staging directory.
- Derive the item list from the captured metadata. Fetch every requested item's observations, following pagination where the API indicates more pages. Check page boundaries, item identity, and observed counts against the API response. Record verified zero-row responses explicitly so replay can distinguish them from missing items.
- Save the metadata response and per-item response pages under `output/commodity_prices/raw_data/` in the promoted capture. Write `meta_data/acquisition.json` with the table ID, acquisition time, and per-item page paths, row counts, and checksums covering all saved inputs, including metadata. Keep credentials out of the capture and diagnostics.
- Add `commodity-prices --transform-only` to replay the last verified capture without network access or an API key. Fail with instructions to run a full refresh if the capture is missing, incomplete, or predates the acquisition record.

## 2. Rebuild and promote the browser bundle

- Use one transformation path for fresh captures and offline replay. Verify the capture record and input files, derive both lookups from metadata, and group observations by item and area. Write a complete, deterministic bundle from scratch, including stable lookup and observation order. Preserve the existing treatment of source values and missing observations.
- Keep the two lookups at `meta_data/{cat02_lookup,area_lookup}.json`. Prototype one chunk per area per ten consecutive items in the published item lookup order. Keep each series intact inside its chunk and omit empty chunks. Use deterministic chunk names derived from area code and item range, as CPI does.
- Compare the existing layout with the prototype using the same capture and gzip settings. Measure published object counts and compressed chunk size distribution. For both routes, measure cold request count and transferred bytes, including single-series views, multi-series comparisons, and selecting more items from the same chunk. Use those results to choose the chunk size.
- Before promotion, check that item and area codes resolve through the lookups, each emitted observation is unique for its item, area, and time, and the files account for all observations expected under the existing transformation rules. Check that filenames and document contents agree, that every series appears in exactly one expected chunk, and that the staged output contains no obsolete files.
- Stage acquisition and browser output under `output/commodity_prices/.staging/`. A full refresh promotes the verified capture and the matching browser bundle together, with rollback if promotion fails. Offline replay replaces the browser bundle while preserving the verified capture. Keep the previous active outputs available during build and validation.

A prototype on the existing local output grouped 77,686 series files into 8,343 chunks, an 89.3% reduction. The median gzip payload was 759 bytes per current file and 5,753 bytes per chunk. This estimate groups the existing JSON documents into arrays and excludes lookups. Validate both routes with a fresh capture before choosing the final chunk size.

## 3. Update the browser loader

- Use a shared chunk loader for the Commodity prices workspace and article. Resolve item codes to chunk ranges from the published item lookup, then extract the requested series by item and area code. Keep the lookup and chunk URLs scoped to the same dataset version.
- Cache in-flight and successful chunk requests by URL so multiple series in one chunk share a fetch. Allow failed requests to retry and preserve the current missing-series behavior.
- Cover shared requests, absent series, retry after failure, and rendering in both routes with browser fixtures. Measure loading with the generated data as well.

## 4. Verify and release

1. Run the Rust tests. Replay the same verified capture twice and compare the browser output byte for byte. Check that offline replay works without `ESTAT_APP_ID`, that rebuilding removes stale series, and that a missing or corrupt input causes a failure while the previous browser bundle still serves.
2. Serve the regenerated output locally. In `/commodity_prices`, load the item and area selectors and select series that fetch data on demand. In `/articles/commodity_prices`, verify the price charts against the regenerated assets. Run the browser baseline as well.
3. Confirm that the publish allowlist contains both lookups and the complete chunk tree and excludes the capture and acquisition metadata. Retain a validated copy of the currently published output for rollback.
4. Publish the new bundle to a versioned Commodity prices prefix in the existing GCS bucket. Update the project-scoped publisher and loader as needed to keep other datasets on their existing paths. Keep the previous prefix available and verify a release build against the new bundle before deploying the loader.
5. Verify both routes in production, including series selection and article charts. Record the published prefix and verification result. If verification fails, redeploy the previous frontend.

Update the [data preparation README](../../data_preparation/README.md) with both command modes and the capture layout as part of implementation. See the [pipeline inventory](pipeline_by_project.md#commodity-prices) for the current behavior.
