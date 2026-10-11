# M5-C offline validation improvements

Run `python3 -m unittest test_reconcile_2024` from this directory to exercise synthetic XLSX fixtures, including checksum and cell corruption.

Run the full gate from `data_preparation/`:

```sh
python3 inspections/vegetable_production_relay/validate_m3_m5.py \
  --2024-workbooks inspections/vegetable_production_relay/runs/official-daikon-2024 \
  --review inspections/vegetable_production_relay/semantic_review.json
```

The workbook directory must include `manifest.json`, with exactly 13 `periods` entries. Each entry is `{ "period": "total" | "2024-MM", "file": "00.xlsx", "sha256": "<sha256>" }`; use the actual names and digests of independently downloaded **official** e-Stat files. The files must match the verified 2021–2023 48-origin/12-region/two-measure workbook layout (A4 title, B-column origin index, C:Z region-measure values). Layout changes fail closed; adapt only after independently documenting changed structure. The workbook and source values are compared lexically, including `0`, `-`, and `…`.

The semantic review `evidence_hashes` fields must equal the SHA-256 hashes of the current temporal mapping and 2024 codebook plus the captured metadata hashes for each year. Reviewer name, affiliation, date, source references and explicit decisions are mandatory. This is *evidence binding*, not authentication of reviewer identity: an independent human reviewer must still verify and approve the evidence.

M5-B is run for all 2021–2023 captures after the separate full-workbook verifier succeeds. Its 14,976 rows/year are checked against the replayed source for symbol, unit, source coordinate and provenance preservation. Direct calls to the inspection-only Rust command remain inspection-only and do not imply that independent review passed.

The gate remains fail-closed and `publishable: false` even when all M3/M5 checks pass. M6 aggregation and M7 publication gates are not implemented here.
