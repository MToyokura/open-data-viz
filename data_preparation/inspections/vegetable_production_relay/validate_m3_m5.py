#!/usr/bin/env python3
"""Offline input, workbook-mapping, and normalization checks for 2021–2023."""
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


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
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
                 "0": "below_display_unit"}
        valid = (evidence.get("table_id") == mapping["years"][year]["table_id"]
                 and evidence.get("metadata_sha256") == mapping["years"][year]["metadata_sha256"]
                 and evidence.get("temporal_mapping_sha256") == sha(MAPPING)
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

    blockers = [c["name"] for c in checks if c["status"] != "passed"]
    report = {"format_version": 1, "generated_at": datetime.now(timezone.utc).isoformat(),
              "status": "PASSED" if not blockers else "BLOCKED",
              "scope": "2021–2023 source replay, workbook mapping, and normalization", "checks": checks,
              "blockers": blockers}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"status": report["status"], "report": str(args.output), "blockers": blockers}, ensure_ascii=False))
    return 0 if not blockers else 1


if __name__ == "__main__":
    raise SystemExit(main())
