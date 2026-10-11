# Vegetable Production Relay: Practical Reimplementation Plan

**Status:** In progress. The 2021–2023 daikon source captures, workbook-backed temporal mapping, normalization, aggregation candidate, and Svelte preview already exist. Reuse and simplify this work; do not restart the investigation.

## Objective and scope

Build a small data-preparation workflow and SvelteKit view for monthly daikon quantities in 2021, 2022, and 2023. Generate static JSON during development/build and deploy it with the application.

The three e-Stat tables currently in scope are `0002008540` (2021), `0002120386` (2022), and `0002114330` (2023). The 13 repeated `cat03` blocks in each table have been matched cell-for-cell to official annual and month-labelled workbooks. The mapping evidence and source references are recorded in `data_preparation/inspections/vegetable_production_relay/`.

This is an application visualization, not an official statistical publication system. Use ordinary code review and tests. A second independent reviewer, immutable release generations, publication manifests, rollback machinery, and formal milestone signoff are out of scope.

## Data correctness rules

- Verify source mappings against e-Stat metadata and official period-labelled workbooks. Do not infer a mapping from code order or labels alone.
- Preserve the distinction between quantity and price, annual totals and monthly observations, origin and destination classifications, and parent regions and their subregions. Never add a subtotal to its constituent categories.
- Document which destination markets contribute to each displayed quantity and why excluded categories overlap or represent summaries.
- Confirm units before combining values. The 2021 source unit still needs confirmation; do not assume it matches later years.
- Preserve raw values and distinguish reported zero, missing observation, suppression, below-precision symbols, and invalid input. Never turn missing or suppressed data into zero.
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

### M1 — Data preparation

Process the three supported years from cached source data and metadata. Reuse the captured tables and verified temporal mapping. Normalize explicit source codes into year, month, origin prefecture, destination category, measure, unit, value, and value status.

Reject unexpected input shapes or unmapped codes. Keep concise mapping documentation next to the source references. Add basic parsing and normalization tests, including missing and suppressed values.

**Done when:** All three years process consistently, monthly mappings match the official workbooks, and quantities, units, categories, and exceptional values remain distinguishable.

### M2 — Aggregation

Aggregate quantity observations by origin prefecture and month using documented destination-market inclusion rules. Do not combine annual totals with monthly values or parent regions with subregions. Mark results incomplete when required observations are absent or suppressed.

Add unit tests for the aggregation rule and manually verify several representative prefecture-month examples against the official source tables, including an ordinary value and an incomplete or exceptional case where available.

**Done when:** Selected results match the source examples, no known summary categories are double-counted, and incomplete quantities are identified explicitly.

### M3 — Data generation

Generate stable, documented static JSON for 2021–2023 from cached inputs. Run parsing, semantic validation, and aggregation before the frontend build; fail the build for invalid input or critical mapping errors. A successful build may include valid and clearly labelled incomplete values.

No versioned generations, publication manifests, or rollback mechanism are needed. Build the data before deploying the application.

**Done when:** The JSON is reproducible from cached source files, follows the documented schema, and the frontend can load it.

### M4 — Visualization

Display monthly daikon quantities on the prefecture map. Provide year and month selectors limited to 2021–2023, distinguish zero from missing or suppressed values, label incomplete data, and show ordinary loading and error states.

**Done when:** The view displays the generated values for all supported years and does not present missing, suppressed, or invalid data as zero.

## Testing and definition of done

- Unit tests cover parsing, source-code mappings, status classification, market inclusion, and aggregation.
- A small set of manually verified examples checks generated quantities against official source tables.
- Build-time validation catches malformed input, unknown classifications, unit mismatches, and invalid values.
- The generated dataset is reproducible from cached inputs, and the SvelteKit view renders the supported years with explicit incomplete-data states.

The feature is done when those checks pass and the visualization presents the 2021–2023 daikon quantities with their units and data-quality states.

## Existing work to reuse

- Inspection notes, metadata inventories, and replay-verified captures under `data_preparation/inspections/vegetable_production_relay/`.
- Official workbook temporal matching in `2021_2023_temporal_evidence.md` and `2021_2023_temporal_mapping.json`.
- Rust normalization and aggregation code plus the current SvelteKit preview. Simplify their interfaces and status model to match this plan; do not reproduce the old pipeline or expand to other years and vegetables as part of this work.
