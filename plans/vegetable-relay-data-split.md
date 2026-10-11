# Vegetable relay data split plan

## Goal

Reduce the amount downloaded when the vegetable relay page opens and when a
product is changed. Split each product's generated aggregation data into:

1. a compact summary used by the Leaflet map, controls, and page references;
2. contribution details fetched only when needed for the selected prefecture's
   card.

Keep the current visualization, values, labels, and source notes unchanged.

## Current behavior and data shape

- `useRelayData` fetches `vegetable_production_relay/{product}.json` whenever a
  product is selected. The product JSON is about 2.8 MB (`veg-02.json` is
  2,819,471 bytes).
- Every product contains three years × twelve months × 47 prefectures, or 1,692
  prefecture-month rows. Every row has nine regional contribution cells.
- `RelayMap.svelte` only uses the monthly prefecture name, reported total,
  reported-cell count, and the counts for below-unit, no-fact, and
  unknown/uncollected cells. It also uses the selected year's source unit for
  its tooltip.
- `+page.svelte` uses summary values for the month slider, card total, and card
  note. The page also needs each year's table ID for the source link, and the
  product level's mapping warnings for the warning notice.
- The card's nine `contributions` are only needed for the currently selected
  product, month, and prefecture. Each contribution's displayed fields are its
  region name, `value_kind`, and raw value. The card does not display
  `source_position` or `source_cat03_code`.

## Proposed data contract

Keep the product index and existing product keys. Replace the single large
product payload with two generated data roles:

### Summary payload

One small summary file per product, fetched during product selection. It
contains:

- `schema_version`, `status`, `commodity`, `commodity_label`, and
  `mapping_warnings`;
- for each year: `year`, `table_id`, `source_unit`, and all month/prefecture
  summary rows;
- each summary row's month and prefecture identity, `reported_value_sum`,
  `status`, and the four cell counts used by the map and card.

It omits the per-cell `contributions` arrays and ingestion/audit hashes that the
frontend does not render. Preserve audit hashes and provenance in the existing
inspection artifacts or a generation-side artifact if they are needed for
reproducibility; do not silently discard their only copy.

### Detail payload

Generate detail data addressable by product, year-month, and prefecture code.
The response contains that observation's nine contribution rows, retaining the
region code/name, `value_kind`, and raw value needed to render the card. Keep
the other source fields if the data preparation pipeline needs them for audit,
but do not send them to the browser unless a consumer needs them.

Use a deterministic, validated path derived from the selected product key and
prefecture code. Avoid creating one static object for every
product/year/month/prefecture combination; that would create tens of thousands
of small objects. A practical first layout is one details file per product and
prefecture containing that prefecture's 36 month records. Fetch that file only
when a prefecture is selected, then choose the selected month locally. This
creates about 47 detail objects per product and reuses one response across
month changes. If hosting/object-count limits make that layout undesirable,
compare it with one product-level details file fetched on the first selection;
record the latency and byte-cost tradeoff before choosing the fallback.

## Implementation sequence

1. **Define and validate the new schema.** Add generator-side summary and
   prefecture-detail types. Validate that every summary row has exactly one
   corresponding detail row and that each detail has the expected nine
   consumer regions. Version the new schema instead of overloading schema
   version 1.
2. **Split generation output.** Update the aggregation output stage to emit the
   summary and detail payloads deterministically. Update product-index
   generation/validation and any output cleanup or publication instructions so
   only complete product splits are advertised.
3. **Update frontend types and fetch helpers.** Fetch the summary on product
   selection. Add an abortable detail fetch keyed by product and prefecture;
   use request identity or cancellation so a slower prior selection cannot
   replace the currently selected card. Cache successful detail responses for
   the current page session, keyed by product and prefecture.
4. **Update the page and map data flow.** Give `RelayMap` summary rows only.
   Keep card totals and the explanatory count note available from the summary.
   Fetch contributions for the selected prefecture, and render a loading/error
   state in the card while its detail request is pending or fails. Request
   details again only if the selected product or prefecture has no cached
   response; month changes should use the already loaded prefecture detail.
5. **Preserve user-visible behavior.** Continue to show the selected month,
   selected prefecture, tooltip labels, map circles, product/year source link,
   mapping warning, and no-data behavior. Ensure changing products does not
   briefly display the old product's totals or contributions.
6. **Roll out compatibly.** Publish all summary/detail objects for a product
   before publishing it in `products.json`. Deploy the frontend to read the new
   schema after the generated data is available. Remove old monolithic product
   files only after the new client/data pair is live and verified.

## Verification checklist

- Generated split output covers the same 1,692 summary observations per
  product and the same nine contributions per observation as the current
  payload.
- For each product/year/month/prefecture, summary totals and status counts
  match the old data; the details match the old contribution order and values.
- Initial load and product changes fetch only the summary. Selecting a
  prefecture fetches its detail data once; changing month reuses the cached
  prefecture details; changing product uses that product's details.
- Aborted or out-of-order requests cannot display another product or
  prefecture's details. Detail fetch failures leave the summary/map usable and
  provide a retry path in the card.
- The product selector, map tooltip/markers, all nine card rows, empty-data
  messages, mapping warnings, and source table links match the current page.
- Record byte sizes for the summary, one prefecture detail file, and the
  current monolithic file. Compare initial transferred bytes and first-click
  bytes for a representative product.

## Main tradeoff

The proposed per-prefecture detail files add object requests and require the
local storage server/deployment to serve nested paths. In return, initial
product selection downloads only map-ready data, and the card downloads only
the selected prefecture's contributions. The product-level details fallback
has fewer objects but delays the large transfer only until the first click;
choose it only if object-count or hosting constraints outweigh the detail
payload savings.

## Relevant code

- `frontend/src/routes/vegetable_production_relay/utils/fetchRelayData.ts`
- `frontend/src/routes/vegetable_production_relay/utils/useRelayData.svelte.ts`
- `frontend/src/routes/vegetable_production_relay/RelayMap.svelte`
- `frontend/src/routes/vegetable_production_relay/+page.svelte`
- `frontend/src/routes/vegetable_production_relay/constants/types.ts`
- `data_preparation/src/modules/projects/vegetable_production_relay/aggregation.rs`
- `data_preparation/src/modules/projects/vegetable_production_relay/product_index.rs`
