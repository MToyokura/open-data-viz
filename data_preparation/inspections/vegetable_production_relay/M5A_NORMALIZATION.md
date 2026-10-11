# M5-A: 2024 daikon inspection-only normalization

This command is **not publication** and **not mapping approval**. It requires the completed 15-page 14,976-observation M4 capture of table `0004044496` and the matching replay-verified metadata, plus the supplied repository `2024_daikon_codebook.json`. No source codes are guessed. Do not commit source observation exports or run directories.

From `data_preparation/`:

```sh
cargo run -- vegetable-relay-normalize-2024-daikon \
  inspections/vegetable_production_relay/runs/observations-<run-id> \
  inspections/vegetable_production_relay/runs/metadata-<run-id> \
  inspections/vegetable_production_relay/2024_daikon_codebook.json \
  inspections/vegetable_production_relay/runs/<inspection-output-directory>
```

The output directory must already exist and should be Git-ignored. The command creates **only** `m5a_inspection_only.json` with `create_new` semantics; any validation failure exits nonzero without producing that file. Replays M4 pages and metadata again offline rather than trusting an editable `source_observations.json`.

## Codebook contract (existing repository file)

Use `inspections/vegetable_production_relay/2024_daikon_codebook.json` from the supplied project diff **without editing its mapping to obtain approval**. The normalizer reads its `time`, `origin`, `measure`, and `consumer_region` arrays and preserves its explicit roles (`monthly`, `summary_excluded`, `prefecture`, `total_excluded`, `quantity`, `price_excluded`, `candidate_disjoint_contributor`, `overlapping_subset_excluded`). The explicit table ID and metadata SHA-256 must match the replayed capture. The codebook status remains `INSPECTION_ONLY_REVIEW_PENDING`.

All axis codes must be represented exactly once, and 12 month codes, one aggregate code, 47 prefecture codes, nine major regions, and three excluded overlapping subsets are required. This mapping is an **inspection-only input**, not independently approved evidence. Unknown categories, malformed coordinates, or a changed status fail closed.

`0` is `below_display_unit` and `-` is `no_fact`. Numeric quantities are represented lexically as `reported` rather than converted to floating point. The output excludes source annual/aggregate time, non-quantity measures, summary origin codes, and subordinate consumer regions, but retains all source coordinates and row positions for the selected 12 × 47 × 9 records. Consumer regions must **never** be interpreted as production prefectures; origin refers to **consignor prefecture**, not farm location.

## Gates intentionally left open

Independent codebook review, reconciliation against the official 2024 spreadsheet (including `0`, `-`, and numeric cases), independent M6 aggregation, and M7 publication approval remain necessary. No output here should be consumed by a public UI or publication workflow. The codebook SHA-256 and metadata SHA-256 are recorded for audit. Confirm the official source unit is encoded as exactly `t` in captured quantity rows before using this contract; otherwise update the unit contract after documentary verification rather than weakening the unit check.

## Reproducible validation suite

Run the offline M3/M5 gate suite from `data_preparation/`:

```sh
python3 inspections/vegetable_production_relay/validate_m3_m5.py
```

It replays each 2021–2023 capture against its matching metadata, compares all 39 official period workbooks with all 13 candidate blocks, validates the 2024 codebook, and replays/normalizes the 2024 capture. It writes a JSON report under the ignored `runs/` directory and exits nonzero if any gate fails or remains blocked. The 2024 workbook reconciliation remains blocked until a 2024 official workbook and a reconciliation adapter are available.

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
