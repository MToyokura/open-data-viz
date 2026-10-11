# Daikon normalization, 2021–2023

Use the captured e-Stat rows, matching metadata, and official workbook mapping to produce normalized records. Current source meanings and units are in [`SOURCE_INTERPRETATION.md`](SOURCE_INTERPRETATION.md).

## Offline checks

From `data_preparation/`, run:

```sh
cargo run -- vegetable-relay-validate-m3-m5
```

This replays the captures, verifies the dated workbook mapping, normalizes all three tables, and checks source positions, codes, values, and periods. It exits unsuccessfully if an input or mapping check fails. It does not require an independent reviewer record.

## Generate normalized rows

For each year, run the normalizer with its captured observations and matching metadata, the shared temporal mapping, and a new output directory:

```sh
cargo run -- vegetable-relay-normalize-2021-2023 \
  <observations-run-dir> <metadata-run-dir> \
  inspections/vegetable_production_relay/2021_2023_temporal_mapping.json \
  <new-output-dir>
```

The output preserves the source table, `cat03` code, period, raw lexical value, and source metadata unit. The 2021 API unit remains absent in the raw row; aggregation uses the workbook-confirmed display unit `t`.

`0`, `-`, and `…` normalize to `below_display_unit`, `no_fact`, and `unknown_or_uncollected`. The annual `計` block is retained as evidence and excluded from monthly aggregation. The aggregation sums only numeric quantity values across the nine top-level major consumer regions; incomplete cells remain separately counted.
