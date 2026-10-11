# 2021–2023 daikon inspection-only normalization

## Reproducible validation suite

Run the offline M3/M5 gate suite from `data_preparation/`:

```sh
python3 inspections/vegetable_production_relay/validate_m3_m5.py
```

It replays the 2021–2023 captures against matching metadata, verifies official workbook temporal mapping and normalizes the three tables. It writes a JSON report under the ignored `runs/` directory and exits nonzero for any failed or blocked gate.

Independent semantic approval is recorded separately. Copy `semantic_review.template.json`, fill the reviewer identity, date, references, and approve/reject decision for every listed interpretation, then run:

```sh
python3 inspections/vegetable_production_relay/validate_m3_m5.py \
  --review inspections/vegetable_production_relay/semantic_review.json
```

The template is not an approval. Rejected or pending interpretations keep the overall gate closed.

## 2021–2023 inspection normalization (M5-B)

After replay-verified metadata and observations are available, the temporal evidence can drive an inspection-only output:

```sh
cargo run -- vegetable-relay-normalize-2021-2023 \
  inspections/vegetable_production_relay/runs/observations-<run-id> \
  inspections/vegetable_production_relay/runs/metadata-<run-id> \
  inspections/vegetable_production_relay/2021_2023_temporal_mapping.json \
  inspections/vegetable_production_relay/runs/<new-output-directory>
```

The command replays raw M4 pages, binds the mapping to the table and metadata hash, checks all 13 exact workbook-match records, and writes `m5b_inspection_only.json` with create-new semantics. `source_cat03_code` preserves the complete source code, which combines the temporal block and region position. `consumer_region_category_code` is derived explicitly as `1001 + ((cat03 - 1001) mod 12)`: each verified period block contains the same 12 region positions, and the official workbook matching establishes their order. The validation runner independently checks every output row's period against the verified `cat03_first_code`/`cat03_last_code` ranges, as well as the region-code derivation. It also retains lexical values, source units (including missing unit metadata), annual-summary identity, and provenance hashes. `0`, `-`, and `…` remain distinct source states; the output remains `publishable: false` pending the broader semantic review.
