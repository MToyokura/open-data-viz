#!/usr/bin/env python3
"""Export provisional app-friendly summaries from the 2024 M5-A inspection artifact.

Only standard-library dependencies. This is NOT a publication or national-production
statistics pipeline. Never interpret a no-fact cell as zero.
"""
import argparse
from collections import defaultdict
from decimal import Decimal
import hashlib
import json
from pathlib import Path

TABLE = "0004044496"
EXPECTED = 12 * 47 * 9


def require(ok, message):
    if not ok:
        raise ValueError(message)


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def labels(book, section, role):
    items = book[section]
    result = {x["code"]: x["label"] for x in items if x["role"] == role}
    require(len(result) == len([x for x in items if x["role"] == role]), f"duplicate {section} codes")
    return result


def number(value):
    require(isinstance(value, str), "reported tonnes must be lexical string")
    # Source normalizer already validates syntax; do not accept exotic number forms here.
    import re
    require(bool(re.fullmatch(r"(?:[0-9]+|[1-9][0-9]{0,2}(?:,[0-9]{3})+)(?:\.[0-9]+)?", value)), "invalid quantity format")
    result = Decimal(value.replace(",", ""))
    require(result > 0 and result.is_finite(), "reported quantity must be positive")
    return result


def aggregate(group_rows):
    summed = Decimal(0)
    counts = {"reported": 0, "below_display_unit": 0, "no_fact": 0}
    for row in group_rows:
        quantity = row["quantity"]
        kind = quantity["kind"]
        require(kind in counts, f"unknown quantity kind: {kind}")
        counts[kind] += 1
        if kind == "reported":
            summed += number(quantity["tonnes_lexical"])
        else:
            require("tonnes_lexical" not in quantity, "missing/rounded cells cannot carry numeric tonnes")
    return {
        "reported_tonnes_sum": format(summed, "f"),
        "reported_cells": counts["reported"],
        "below_display_unit_cells": counts["below_display_unit"],
        "no_fact_cells": counts["no_fact"],
        "contributing_cells": len(group_rows),
        "completeness": "all_reported" if counts["reported"] == len(group_rows) else "partial",
    }


def export(inspection, book, source_sha256):
    validation = inspection["validation"]
    require(validation["status"] == "M5A_INSPECTION_ONLY" and validation["publishable"] is False, "expected nonpublishable M5-A artifact")
    require(validation["table_id"] == TABLE and book["table_id"] == TABLE, "table ID mismatch")
    require(book["status"] == "INSPECTION_ONLY_REVIEW_PENDING", "unexpected codebook status")
    require(validation["normalized_count"] == EXPECTED and validation["input_count"] == 14976, "unexpected source counts")
    prefectures = labels(book, "origin", "prefecture")
    regions = labels(book, "consumer_region", "candidate_disjoint_contributor")
    require(len(prefectures) == 47 and len(regions) == 9, "unexpected origin/region codebook")
    require(validation["metadata_sha256"] == book["metadata_sha256"], "metadata mismatch")
    rows = inspection["rows"]
    require(len(rows) == EXPECTED, "unexpected row count")
    seen = set()
    by_prefecture = defaultdict(list)
    by_region = defaultdict(list)
    time_codes = {x["reporting_month"]: x["code"] for x in book["time"] if x["role"] == "monthly"}
    require(len(time_codes) == 12, "unexpected month mappings")
    for row in rows:
        month = row["month"]
        origin = row["consignor_prefecture_code"]
        region = row["major_consumer_region_code"]
        require(type(month) is int and 1 <= month <= 12, "invalid month")
        require(origin in prefectures and region in regions, "unknown origin or region")
        require(row["source_table_id"] == TABLE and row["source_unit"] == "t", "source ID/unit mismatch")
        require(row["source_attributes"]["@cat01"] == origin and row["source_attributes"]["@cat03"] == region, "coordinate mismatch")
        require(row["source_attributes"]["@cat02"] == "1001", "not quantity measure")
        require(row["source_attributes"]["@time"] == time_codes[f"2024-{month:02d}"], "source time mismatch")
        key = (month, origin, region)
        require(key not in seen, "duplicate monthly coordinate")
        seen.add(key)
        by_prefecture[(month, origin)].append(row)
        by_region[(month, region)].append(row)
    require(len(seen) == EXPECTED, "missing coordinates")
    prefecture_months = []
    for month in range(1, 13):
        for code, label in sorted(prefectures.items()):
            cells = by_prefecture[(month, code)]
            require(len(cells) == 9, "incomplete prefecture-month")
            prefecture_months.append({"month": f"2024-{month:02d}", "consignor_prefecture_code": code, "consignor_prefecture_name": label, **aggregate(cells)})
    region_months = []
    for month in range(1, 13):
        for code, label in sorted(regions.items()):
            cells = by_region[(month, code)]
            require(len(cells) == 47, "incomplete region-month")
            region_months.append({"month": f"2024-{month:02d}", "major_consumer_region_code": code, "major_consumer_region_name": label, **aggregate(cells)})
    return {
        "schema_version": 1,
        "status": "PROVISIONAL_DEMO_ONLY",
        "publishable": False,
        "warning": "Illustrative app data from unreviewed e-Stat classifications; sums cover selected major consumer regions, not national production or national wholesale totals. Partial sums omit no-fact cells and retain rounded-below-display-unit counts.",
        "commodity": "daikon",
        "year": 2024,
        "unit": "t",
        "source_table_id": TABLE,
        "source_m5a_sha256": source_sha256,
        "source_metadata_sha256": validation["metadata_sha256"],
        "source_codebook_sha256": validation["codebook_sha256"],
        "prefecture_months": prefecture_months,
        "consumer_region_months": region_months,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("m5a_json", type=Path)
    parser.add_argument("codebook_json", type=Path)
    parser.add_argument("output_json", type=Path)
    args = parser.parse_args()
    require(args.output_json.parent.is_dir(), "output directory must exist")
    data = args.m5a_json.read_bytes()
    doc = export(json.loads(data), load_json(args.codebook_json), hashlib.sha256(data).hexdigest())
    # Exclusive create: never silently overwrite an existing exported demo dataset.
    with args.output_json.open("x", encoding="utf-8") as output:
        json.dump(doc, output, ensure_ascii=False, indent=2)
        output.write("\n")
    print(f"Wrote provisional demo export: {args.output_json} ({len(doc['prefecture_months'])} prefecture-months, {len(doc['consumer_region_months'])} consumer-region-months)")


if __name__ == "__main__":
    main()
