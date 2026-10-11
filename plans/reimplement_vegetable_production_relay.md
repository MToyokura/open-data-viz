# Vegetable Production Relay: Product Expansion Plan

**Status:** Complete for all 50 vegetable products in the cached e-Stat catalogue for 2021–2023. Product-specific workbook mappings and units were verified, all product datasets were generated, and the frontend now selects from the generated product index. そらまめ has exact-value-equivalent candidate blocks in five periods; the equivalent values are preserved and the limitation is reported in the view.

## Objective and scope

Build a product-aware data-preparation workflow and SvelteKit view for monthly vegetable quantities in 2021, 2022, and 2023. Generate one static JSON file per verified product during development/build and deploy the files with the application. The initial expansion target is every vegetable product in the cached `00500226` catalogue, not fruit products or additional years.

Daikon's tables are `0002008540` (2021), `0002120386` (2022), and `0002114330` (2023). Their 13 repeated `cat03` blocks have been matched cell-for-cell to official annual and month-labelled workbooks. The cached catalogue contains 50 vegetable products per year, each with a product-specific table. The daikon mapping is evidence only for daikon: verify each added product's metadata, periods, and units against its own official workbooks. Store product table IDs, mapping evidence, and source references under `data_preparation/inspections/vegetable_production_relay/`.

This is an application visualization, not an official statistical publication system. Use ordinary code review and tests. A second independent reviewer, immutable release generations, publication manifests, rollback machinery, and formal milestone signoff are out of scope.

## Data correctness rules

- Verify each product's source mappings against e-Stat metadata and official period-labelled workbooks. Do not copy daikon's mapping to another product or infer a mapping from code order or labels alone. When two or more blocks are exactly identical, record the equivalent candidates and label the limitation; only display values proven identical across every valid assignment.
- Preserve the distinction between quantity and price, annual totals and monthly observations, origin and destination classifications, and parent regions and their subregions. Never add a subtotal to its constituent categories.
- Document destination inclusion rules. Sum the nine level-1 major consumer regions and exclude their three nested categories: `東北_仙台・盛岡`, `関東_京浜`, and `九州_北九州`.
- Use each product's official workbook headings to establish quantity and price units. Do not assume a product's unit metadata or temporal mapping from another product. Daikon's verified units are `t` and `円/kg`; its 2021 API metadata omits units, while its annual/month workbooks provide them.
- Preserve raw values and distinguish below-display-unit `0`, no-fact `－`, unknown/uncollected `…`, and invalid input. Never turn these states into an exact zero.
- Describe `cat01` as reported origin prefecture and the result as wholesale flows to selected major consumer regions, not agricultural production or a national total.
- Check whether MAFF's 2022 change to the annual survey target affects these three origin-by-region tables before making year-to-year comparisons.
- Treat incomplete but interpretable results as incomplete and label them. Reject unrecognized formats, categories, units, and value symbols instead of guessing.
- Use the established aggregation rule only after its included markets are documented:

  `Q(prefecture, month) = sum(quantity for included, non-overlapping destination markets)`

- Keep source table IDs and concise transformation rules with generated data. Full per-observation lineage is unnecessary for the application.

## Processing flow

```text
Cached e-Stat source data and metadata
        ↓
Rust parsing and normalization
        ↓
Input and semantic validation
        ↓
Monthly aggregation
        ↓
Static JSON generated during build
        ↓
SvelteKit visualization
```

Cache source files locally so data can be regenerated without an API request. Existing replay-verified observation and metadata captures may be reused; there is no requirement to build a general replay framework or immutable evidence store.

Use straightforward validation states:

| State | Meaning | Display behavior |
|---|---|---|
| `valid` | Values and mappings are interpretable and required inputs are present | Display the quantity |
| `incomplete` | Values are interpretable, with missing or suppressed observations | Display with an incomplete-data indication |
| `invalid` | Source data or semantics cannot be interpreted reliably | Fail generation or omit the affected quantity |

Prefer a quantity of `number | null` with a status alongside it. Remove the separate `publishable` gate when updating the schema and frontend.

## Milestones

### M1 — Product source inventory — complete

Use the cached catalogue to create an explicit inventory of the 50 vegetable products and their 2021–2023 table IDs. Acquire and replay metadata and observations per product. Record product key, Japanese label, table IDs, metadata hashes, source availability, and verification state. Do not treat catalogue titles as semantic or temporal evidence.

**Done when:** The product inventory is complete and every product has an explicit per-year source identity or a documented unavailable state.

### M2 — Product-specific source mapping — complete

For each product, verify all annual/month `cat03` blocks against its official period-labelled workbooks, including quantity and price units. Require a complete one-to-one assignment for all 13 periods. If duplicate blocks permit multiple assignments, record the equivalent exact-match candidates and verify that displayed values are unchanged across those assignments; show a warning in the view. Keep the daikon mapping unchanged and add separate mapping evidence per product. Do not expose a product in the app until all three years pass these checks.

**Done when:** Every enabled product has complete exact workbook matches and explicit units for all supported years; unsupported or unverified products remain unavailable.

### M3 — Product-aware normalization — complete

Process each enabled product's three years from cached source data and metadata. Reuse the daikon mapping and apply the same validation contract to separately verified product mappings. Normalize explicit source codes into product, year, month, origin prefecture, destination category, measure, unit, value, and value status.

Reject unexpected input shapes or unmapped codes. Keep concise mapping documentation next to the source references. Existing missing and suppressed value states are preserved through the generated datasets.

**Done when:** Every enabled product processes consistently for all three years, mappings match its official workbooks, and quantities, units, categories, and exceptional values remain distinguishable.

### M4 — Aggregation — complete

Aggregate quantity observations by origin prefecture and month using documented destination-market inclusion rules. Do not combine annual totals with monthly values or parent regions with subregions. Mark results incomplete when required observations are absent or suppressed.

Keep aggregation product-independent and manually verify representative prefecture-month examples for each newly enabled product against its official source tables, including an ordinary value and an incomplete or exceptional case where available.

**Done when:** Selected results match the source examples, no known summary categories are double-counted, and incomplete quantities are identified explicitly.

### M5 — Data generation — complete

Generate stable, documented static JSON per enabled product for 2021–2023 from cached inputs, plus a product index listing only generated, verified products. Run parsing, semantic validation, and aggregation before the frontend build; fail generation for invalid input or critical mapping errors. A successful build may include valid and clearly labelled incomplete values.

No versioned generations, publication manifests, or rollback mechanism are needed. Build the data before deploying the application.

**Done when:** The JSON is reproducible from cached source files, follows the documented schema, and the frontend can load it.

### M6 — Visualization — complete

Display the selected product's monthly quantities on the prefecture map. Add a product selector populated from the generated product index, alongside year and month selectors limited to 2021–2023. Distinguish zero from missing or suppressed values, label incomplete data, and show ordinary loading and error states.

**Done when:** The view displays the generated values for all supported years and does not present missing, suppressed, or invalid data as zero.

## Validation and definition of done

- All 1,950 official annual/month workbooks were compared against every source `cat03` block (1,152 cells per workbook); all 50 products have complete three-year mappings and explicit units.
- Rust build/check and SvelteKit check pass. The generated index contains 50 products and each has a generated 2021–2023 JSON dataset with 564 prefecture-month rows per year.
- The view loads the selected dataset and presents units, incomplete states, and the equivalent-block limitation for そらまめ.

The expansion is done when those checks pass for all 50 catalogue vegetable products and the visualization presents each available product's 2021–2023 quantities with its units and data-quality states. Products that cannot be verified are explicitly unavailable and are not presented as supported data.

## Completed implementation and evidence

- Inspection notes, cached catalogue, metadata inventories, and replay-verified captures under `data_preparation/inspections/vegetable_production_relay/`.
- `product_inventory_2021_2023.json` lists all 50 catalogue products, their 150 per-year table IDs, and the completed mapping status and evidence paths.
- Daikon's official workbook temporal matching in `2021_2023_temporal_evidence.md` and `2021_2023_temporal_mapping.json`. This evidence applies only to daikon.
- Product-specific normalization and aggregation interfaces, a product index generator, and the SvelteKit product selector.
- Validation performed: Rust `cargo check`, frontend `npm run check`, product artifact audit, and `git diff --check`. Unit tests were not run.
