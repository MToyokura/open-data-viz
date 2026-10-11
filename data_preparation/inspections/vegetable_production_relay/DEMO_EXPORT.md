# 2024 daikon: provisional app-data export

This **development/demo-only** exporter consumes the existing M5-A normalized
JSON and the existing 2024 codebook. It does **not** download e-Stat data, resolve
independent review, modify the Rust pipeline, or promote inspection data for
statistical publication. Do not present output as national production statistics.

From `data_preparation/`:

```sh
mkdir -p inspections/vegetable_production_relay/runs/demo-2024
python3 inspections/vegetable_production_relay/export_2024_daikon_demo.py \
  inspections/vegetable_production_relay/runs/m5a-daikon-2024/m5a_inspection_only.json \
  inspections/vegetable_production_relay/2024_daikon_codebook.json \
  inspections/vegetable_production_relay/runs/demo-2024/daikon_2024_demo.json
```

Output (new file only; no overwrite) is `PROVISIONAL_DEMO_ONLY` and
`publishable: false`. It contains 564 `prefecture_months` (12 months × 47
**consignor** prefectures) and 108 `consumer_region_months` (12 × nine major
consumer regions), with names, codes and numeric strings in tonnes.

`reported_tonnes_sum` adds **only** positive, reported numerical cells. The
`below_display_unit_cells` and `no_fact_cells` counts preserve excluded cells;
`completeness` is `partial` whenever any such cell occurs. **Do not treat
partial sums as complete totals or silently map missing cells to zero.** The
nine candidate consumer regions are not equivalent to nationwide production or
wholesale volume, and consignor prefecture is not necessarily growing location.

Integrate only behind a demo/development flag in the application. A production
path needs the separately tracked provenance, reconciliation, aggregation and
publication review work; this script does not remove any existing gates.

Run unit tests with:

```sh
python3 -m unittest discover -s inspections/vegetable_production_relay -p 'test_demo_export.py'
```
