#!/usr/bin/env python3
"""Offline M3/M5 gate runner. Writes a machine-readable report and exits nonzero on blockers."""
import argparse
import hashlib
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
RUNS = ROOT / "runs"
MAPPING = ROOT / "2021_2023_temporal_mapping.json"
CODEBOOK = ROOT / "2024_daikon_codebook.json"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--review", type=Path, help="independent semantic review JSON")
    parser.add_argument("--2024-workbooks", dest="workbooks", type=Path,
                        help="directory of 13 official workbooks and manifest.json")
    parser.add_argument("--output", type=Path, default=RUNS / "m3-m5-validation-report.json")
    args = parser.parse_args()
    checks = []
    def add(name, status, details):
        checks.append({"name": name, "status": status, "details": details})
    data_dir = ROOT.parents[1]

    mapping = json.loads(MAPPING.read_text())
    captures = {}
    for year, entry in mapping["years"].items():
        capture = RUNS.parent / entry["raw_capture"] / "source_observations.json"
        metadata_dirs = [p.parent for p in RUNS.glob("metadata-*/capture.json")
                         if json.loads(p.read_text()).get("sha256") == entry["metadata_sha256"]]
        if not capture.exists() or len(metadata_dirs) != 1:
            add(f"M4 replay {year}", "blocked", "raw capture or uniquely bound metadata capture missing")
            continue
        captures[year] = (capture.parent, metadata_dirs[0])
        replay = subprocess.run(["cargo", "run", "--quiet", "--", "vegetable-relay-acquisition-replay",
                                 str(capture.parent), str(metadata_dirs[0])], cwd=data_dir,
                                capture_output=True, text=True)
        add(f"M4 replay {year}", "passed" if replay.returncode == 0 else "failed",
            replay.stdout.strip() or replay.stderr[-1200:].strip())

    temporal = subprocess.run([sys.executable, str(ROOT / "verify_temporal_mapping.py")],
                              cwd=data_dir, capture_output=True, text=True)
    add("2021-2023 official workbook temporal and cell matching",
        "passed" if temporal.returncode == 0 else "failed",
        temporal.stdout.strip() if temporal.returncode == 0 else temporal.stderr[-2000:].strip())

    # M5-B is exercised only when the independently rerun workbook comparison passes.
    for year in mapping["years"]:
        if year not in captures or temporal.returncode != 0:
            add(f"M5-B historical normalization {year}", "blocked",
                "missing replay inputs or workbook verification did not pass")
            continue
        observation_dir, metadata_dir = captures[year]
        out_dir = RUNS / f"m5b-validation-{year}-{time.time_ns()}"
        out_dir.mkdir(parents=True, exist_ok=False)
        run = subprocess.run(["cargo", "run", "--quiet", "--", "vegetable-relay-normalize-2021-2023",
                              str(observation_dir), str(metadata_dir), str(MAPPING), str(out_dir)],
                             cwd=data_dir, capture_output=True, text=True)
        path = out_dir / "m5b_inspection_only.json"
        if run.returncode != 0 or not path.exists():
            add(f"M5-B historical normalization {year}", "failed", run.stderr[-2000:])
            continue
        evidence = json.loads(path.read_text())
        rows = evidence.get("rows", [])
        src = json.loads((observation_dir / "source_observations.json").read_text())
        expected = {r["position"]: r for r in src["rows"]}
        period_by_cat03 = {}
        for mapping_period in mapping["years"][year]["periods"]:
            first = int(mapping_period["cat03_first_code"])
            last = int(mapping_period["cat03_last_code"])
            for cat03 in range(first, last + 1):
                if cat03 in period_by_cat03:
                    period_by_cat03[cat03] = None
                else:
                    period_by_cat03[cat03] = mapping_period["period"]
        kinds = {"-": "no_fact", "…": "unknown_or_uncollected",
                 "0": "reported_zero_symbol_unspecified"}
        valid = (evidence.get("table_id") == mapping["years"][year]["table_id"]
                 and evidence.get("metadata_sha256") == mapping["years"][year]["metadata_sha256"]
                 and evidence.get("temporal_mapping_sha256") == sha(MAPPING)
                 and evidence.get("publishable") is False
                 and evidence.get("input_count") == 14976
                 and evidence.get("monthly_count") == 13824
                 and evidence.get("annual_summary_count") == 1152
                 and len(rows) == 14976 and len(expected) == 14976)
        if valid:
            seen = set()
            period_assignments_verified = 0
            for row in rows:
                pos = row["source_position"]
                original = expected.get(pos)
                if original is None or pos in seen:
                    valid = False
                    break
                seen.add(pos)
                raw = original["raw_value"]
                try:
                    source_cat03 = int(original["attributes"]["@cat03"])
                except (KeyError, TypeError, ValueError):
                    valid = False
                    break
                expected_period = period_by_cat03.get(source_cat03)
                expected_region = 1001 + (source_cat03 - 1001) % 12
                valid = (row["raw_value"] == raw
                         and row["source_attributes"] == original["attributes"]
                         and row.get("source_cat03_code") == str(source_cat03)
                         and row["consumer_region_category_code"] == str(expected_region)
                         and expected_period is not None
                         and row["period"] == expected_period
                         and row["annual_total"] == (expected_period == "total")
                         and row["source_unit"] == original.get("unit")
                         and row["value_kind"] == kinds.get(raw, "reported")
                         and row["annual_total"] == (row["period"] == "total"))
                if not valid:
                    break
                period_assignments_verified += 1
        add(f"M5-B historical normalization {year}", "passed" if valid else "failed",
            {"rows": len(rows), "period_assignments_verified": period_assignments_verified if valid else 0,
             "symbols_and_provenance_preserved": bool(valid)})

    codebook = json.loads(CODEBOOK.read_text())
    valid_book = (codebook.get("status") == "INSPECTION_ONLY_REVIEW_PENDING"
                  and codebook.get("table_id") == "0004044496"
                  and len(codebook.get("time", [])) == 13
                  and len(codebook.get("origin", [])) == 48
                  and len(codebook.get("measure", [])) == 2
                  and len(codebook.get("consumer_region", [])) == 12
                  and len(codebook.get("source_documents", [])) >= 2)
    add("2024 codebook structure and documentary references", "passed" if valid_book else "failed",
        {"status": codebook.get("status"), "axes": {k: len(codebook.get(k, [])) for k in ("time", "origin", "measure", "consumer_region")},
         "codebook_sha256": sha(CODEBOOK)})

    obs_id = "observations-1791678455-860323437-165827"
    meta_id = "metadata-1791675366-450693749-111423"
    out_dir = RUNS / f"m5-validation-replay-{int(time.time())}"
    out_dir.mkdir(exist_ok=True)
    normalized = subprocess.run(["cargo", "run", "--quiet", "--", "vegetable-relay-normalize-2024-daikon",
                                 str(RUNS / obs_id), str(RUNS / meta_id), str(CODEBOOK), str(out_dir)],
                                cwd=data_dir, capture_output=True, text=True)
    result_path = out_dir / "m5a_inspection_only.json"
    if normalized.returncode == 0 and result_path.exists():
        evidence = json.loads(result_path.read_text())
        add("2024 normalized cell grid, units, region roles, and symbols", "passed",
            {"input_count": evidence["validation"]["input_count"],
             "normalized_count": evidence["validation"]["normalized_count"],
             "reported": evidence["validation"]["reported_count"],
             "below_display_unit": evidence["validation"]["below_display_unit_count"],
             "no_fact": evidence["validation"]["no_fact_count"],
             "publishable": evidence["validation"]["publishable"]})
    else:
        add("2024 normalized cell grid, units, region roles, and symbols", "failed",
            normalized.stderr[-2000:].strip())

    if args.workbooks and (args.workbooks / "manifest.json").is_file():
        rec = subprocess.run([sys.executable, str(ROOT / "reconcile_2024.py"),
                              "--workbooks", str(args.workbooks),
                              "--capture", str(RUNS / obs_id / "source_observations.json"),
                              "--codebook", str(CODEBOOK)],
                             cwd=data_dir, capture_output=True, text=True)
        add("2024 official workbook reconciliation", "passed" if rec.returncode == 0 else "failed",
            rec.stdout.strip() if rec.returncode == 0 else rec.stderr[-2000:])
    else:
        add("2024 official workbook reconciliation", "blocked",
            "provide --2024-workbooks with 13 official XLSX files and their checksum manifest")

    review_ok = False
    if args.review and args.review.exists():
        review = json.loads(args.review.read_text())
        required = {"month_and_total", "consignor_vs_production", "consumer_regions_and_subsets",
                    "units_and_symbols", "cross_year_coverage", "temporal_mapping"}
        decisions = review.get("decisions", {})
        expected_hashes = {
            "temporal_mapping_sha256": sha(MAPPING),
            "2024_codebook_sha256": sha(CODEBOOK),
            "2024_metadata_sha256": codebook["metadata_sha256"],
            **{f"{year}_metadata_sha256": entry["metadata_sha256"]
               for year, entry in mapping["years"].items()},
        }
        review_complete = (review.get("reviewer") and review.get("reviewer_affiliation")
                           and review.get("reviewed_at") and review.get("source_references")
                           and all(isinstance(ref, str) and ref.strip() for ref in review["source_references"])
                           and review.get("evidence_hashes") == expected_hashes
                           and required <= decisions.keys()
                           and all(decisions[k] in ("approved", "rejected") for k in required))
        review_ok = review_complete and all(decisions[k] == "approved" for k in required)
        add("Independent M3 semantic approval", "passed" if review_ok else "failed",
            {"review_file": str(args.review), "required_decisions": sorted(required),
             "decision_count": len(decisions), "complete": bool(review_complete),
             "rejected": sorted(k for k in required if decisions.get(k) == "rejected")})
    else:
        add("Independent M3 semantic approval", "blocked",
            "no independent reviewer decision record supplied")

    blockers = [c["name"] for c in checks if c["status"] != "passed"]
    report = {"format_version": 1, "generated_at": datetime.now(timezone.utc).isoformat(),
              "status": "PASSED" if not blockers else "BLOCKED", "publishable": False,
              "scope": "M3/M5 validation only; M6 and M7 excluded", "checks": checks,
              "blockers": blockers}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"status": report["status"], "report": str(args.output), "blockers": blockers}, ensure_ascii=False))
    return 0 if not blockers else 1


if __name__ == "__main__":
    raise SystemExit(main())
