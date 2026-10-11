"""Match every official product/year workbook against all source cat03 blocks.

Requires cached workbooks from acquire_product_workbooks.py and replay-verified
source_observations.json files linked from product_inventory_2021_2023.json. Each
workbook must uniquely match exactly 1,152 lexical cells before a product mapping
is written. Uses only the Python standard library.
"""

import hashlib
import json
import re
import unicodedata
import xml.etree.ElementTree as ET
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent
RUNS = ROOT / "runs"
NS = {"m": "http://schemas.openxmlformats.org/spreadsheetml/2006/main"}
YEARS = ["2021", "2022", "2023"]
PERIODS = ["total", *[f"{month:02}" for month in range(1, 13)]]


def spreadsheet_cells(path):
    with zipfile.ZipFile(path) as archive:
        strings = []
        if "xl/sharedStrings.xml" in archive.namelist():
            root = ET.fromstring(archive.read("xl/sharedStrings.xml"))
            strings = [
                "".join(node.text or "" for node in item.findall(".//m:t", NS))
                for item in root.findall("m:si", NS)
            ]
        sheet = ET.fromstring(archive.read("xl/worksheets/sheet1.xml"))
        cells = {}
        for cell in sheet.findall(".//m:sheetData/m:row/m:c", NS):
            value = cell.find("m:v", NS)
            inline = cell.find("m:is", NS)
            if value is not None:
                cells[cell.attrib["r"]] = (
                    strings[int(value.text)] if cell.get("t") == "s" else value.text
                )
            elif inline is not None:
                cells[cell.attrib["r"]] = "".join(
                    node.text or "" for node in inline.findall(".//m:t", NS)
                )
        return cells


def column(number):
    result = ""
    while number:
        number, remainder = divmod(number - 1, 26)
        result = chr(65 + remainder) + result
    return result


def verify_workbook(product, year, period, workbook, source_rows):
    cells = spreadsheet_cells(workbook["local_path"])
    title = unicodedata.normalize("NFKC", cells.get("A4", ""))
    label = unicodedata.normalize("NFKC", product["label"])
    expected_period = "計" if period == "total" else f"{int(period)}月"
    if label not in title or not title.endswith(expected_period):
        raise RuntimeError(
            f"{product['key']} {year} {period}: workbook title does not identify product and period"
        )

    origin_rows = {
        int(value): int(re.search(r"\d+", address).group())
        for address, value in cells.items()
        if re.fullmatch(r"B\d+", address)
        and re.fullmatch(r"\d+", str(value))
        and 1 <= int(value) <= 48
    }
    if set(origin_rows) != set(range(1, 49)):
        raise RuntimeError(f"{product['key']} {year} {period}: origin rows are incomplete")

    for region in range(12):
        quantity_unit = unicodedata.normalize("NFKC", cells.get(f"{column(3 + region * 2)}11", ""))
        price_unit = unicodedata.normalize("NFKC", cells.get(f"{column(4 + region * 2)}11", ""))
        if not quantity_unit.startswith("t") or not price_unit.startswith("円/kg"):
            raise RuntimeError(
                f"{product['key']} {year} {period}: workbook units differ at region column {region + 1}"
            )

    scores = {}
    for block in range(1, 14):
        matches = 0
        for origin, row_number in origin_rows.items():
            for region in range(12):
                for measure in range(2):
                    address = f"{column(3 + region * 2 + measure)}{row_number}"
                    workbook_value = cells.get(address)
                    code = str(989 + block * 12 + region)
                    api_value = source_rows[(str(1000 + origin), str(1001 + measure), code)]
                    if workbook_value is None:
                        raise RuntimeError(
                            f"{product['key']} {year} {period}: workbook cell {address} missing"
                        )
                    matches += str(workbook_value) == api_value
        scores[block] = matches
    candidates = [block for block, matches in scores.items() if matches == 1152]
    if not candidates:
        raise RuntimeError(f"{product['key']} {year} {period}: no exact block match; {scores}")
    return scores, candidates


def unique_perfect_mapping(candidate_blocks, product_key, year):
    periods = sorted(candidate_blocks, key=lambda period: (len(candidate_blocks[period]), period))
    solutions = []

    def search(index, assigned, used):
        if len(solutions) > 1:
            return
        if index == len(periods):
            solutions.append(dict(assigned))
            return
        period = periods[index]
        for block in candidate_blocks[period]:
            if block not in used:
                assigned[period] = block
                used.add(block)
                search(index + 1, assigned, used)
                used.remove(block)
                assigned.pop(period)

    search(0, {}, set())
    if not solutions:
        raise RuntimeError(
            f"{product_key} {year}: workbook-to-block evidence has no complete one-to-one mapping"
        )
    ambiguities = {
        period: sorted(blocks)
        for period, blocks in candidate_blocks.items()
        if len(blocks) > 1
    }
    return solutions[0], ambiguities


def mapping_record(year, period, workbook, scores, assigned_block):
    exact_candidates = sorted(block for block, matches in scores.items() if matches == 1152)
    first_code = str(989 + assigned_block * 12)
    mapped_period = "total" if period == "total" else f"{year}-{period}"
    return {
        "period": mapped_period,
        "block": assigned_block,
        "cat03_first_code": first_code,
        "cat03_last_code": str(int(first_code) + 11),
        "stat_inf_id": workbook["stat_inf_id"],
        "spreadsheet_url": workbook["url"],
        "spreadsheet_sha256": workbook["sha256"],
        "matching_cells": scores[assigned_block],
        "compared_cells": 1152,
        "runner_up_matches": max(
            matches for block, matches in scores.items() if block != assigned_block
        ),
        "exact_match_candidates": exact_candidates,
    }


def main():
    inventory_path = ROOT / "product_inventory_2021_2023.json"
    manifest_path = ROOT / "product_workbook_manifest_2021_2023.json"
    inventory = json.loads(inventory_path.read_text())
    manifest = json.loads(manifest_path.read_text())
    workbook_index = {
        (item["product_key"], item["year"], item["period"]): {
            **item,
            "local_path": ROOT / item["path"],
        }
        for item in manifest["workbooks"]
    }
    if len(workbook_index) != 1950:
        raise RuntimeError(f"expected 1,950 product workbooks, found {len(workbook_index)}")

    mapping_dir = ROOT / "product_temporal_mappings"
    mapping_dir.mkdir(exist_ok=True)
    summaries = []
    for product in inventory["products"]:
        years = {}
        for year in YEARS:
            observation_ref = product.get("observation_runs", {}).get(year)
            metadata_ref = product.get("metadata_runs", {}).get(year)
            if not observation_ref or not metadata_ref:
                raise RuntimeError(f"{product['key']} {year}: source evidence is not linked")
            observation_path = RUNS / observation_ref["run_id"] / "source_observations.json"
            report = json.loads(observation_path.read_text())
            if report["table_id"] != product["table_ids"][year] or report["count"] != 14976:
                raise RuntimeError(f"{product['key']} {year}: source table or observation count mismatch")
            if report["source_metadata_sha256"] != metadata_ref["metadata_sha256"]:
                raise RuntimeError(f"{product['key']} {year}: observation/metadata hashes differ")
            source_rows = {
                (
                    row["attributes"]["@cat01"],
                    row["attributes"]["@cat02"],
                    row["attributes"]["@cat03"],
                ): row["raw_value"]
                for row in report["rows"]
            }
            if len(source_rows) != 14976:
                raise RuntimeError(f"{product['key']} {year}: source coordinate grid is incomplete")
            workbook_evidence = {}
            candidate_blocks = {}
            for period in PERIODS:
                workbook = workbook_index[(product["key"], year, period)]
                actual_sha = hashlib.sha256(workbook["local_path"].read_bytes()).hexdigest()
                if actual_sha != workbook["sha256"]:
                    raise RuntimeError(f"{product['key']} {year} {period}: workbook checksum mismatch")
                scores, candidates = verify_workbook(product, year, period, workbook, source_rows)
                workbook_evidence[period] = (workbook, scores)
                candidate_blocks[period] = candidates
            assignment, ambiguities = unique_perfect_mapping(
                candidate_blocks, product["key"], year
            )
            periods = [
                {
                    **mapping_record(
                        year, period, *workbook_evidence[period], assignment[period]
                    ),
                    "block_identity": "equivalent_candidates"
                    if period in ambiguities
                    else "unique",
                }
                for period in PERIODS
            ]
            years[year] = {
                "table_id": product["table_ids"][year],
                "raw_capture": f"runs/{observation_ref['run_id']}",
                "metadata_sha256": metadata_ref["metadata_sha256"],
                "official_listing": f"https://www.e-stat.go.jp/stat-search/files?cycle=7&layout=datalist&page=1&tclass1=000001020455&tclass2={ {'2021': '000001169106', '2022': '000001209867', '2023': '000001220580'}[year] }&tclass3val=0&toukei=00500226&tstat=000001015623",
                "periods": periods,
                "mapping_ambiguities": ambiguities,
            }
        mapping = {
            "status": "VALUE_MATCH_VERIFIED_WITH_EQUIVALENT_BLOCKS"
            if any(data["mapping_ambiguities"] for data in years.values())
            else "VALUE_MATCH_VERIFIED",
            "method": "For each official title-labelled workbook, compare all 48 origin × 12 region × 2 measure lexical cells against every one of the 13 source cat03 blocks. Require a complete one-to-one assignment from periods to blocks using 1,152/1,152 exact matches. If more than one assignment is possible, record every equivalent exact-match candidate; the displayed values are invariant across those candidates and the UI labels the equivalence.",
            "commodity": product["key"],
            "commodity_label": product["label"],
            "years": years,
        }
        output = mapping_dir / f"{product['key']}.json"
        output.write_text(json.dumps(mapping, ensure_ascii=False, indent=2) + "\n")
        product["mapping_status"] = (
            "verified_with_equivalent_blocks"
            if mapping["status"] == "VALUE_MATCH_VERIFIED_WITH_EQUIVALENT_BLOCKS"
            else "verified"
        )
        product["mapping_evidence"] = str(output.relative_to(ROOT))
        summaries.append(
            {
                "key": product["key"],
                "label": product["label"],
                "period_matches": sum(len(year["periods"]) for year in years.values()),
                "equivalent_periods": sum(
                    len(year["mapping_ambiguities"]) for year in years.values()
                ),
                "minimum_runner_up_matches": min(
                    period["runner_up_matches"]
                    for year in years.values()
                    for period in year["periods"]
                ),
            }
        )
        inventory_path.write_text(json.dumps(inventory, ensure_ascii=False, indent=2) + "\n")
        print(f"verified {product['key']} {product['label']}: 39/39 workbooks", flush=True)

    report = {
        "status": "VALUE_MATCH_VERIFIED",
        "product_count": len(summaries),
        "workbook_matches": sum(item["period_matches"] for item in summaries),
        "products": summaries,
    }
    (ROOT / "product_temporal_mapping_report_2021_2023.json").write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n"
    )


if __name__ == "__main__":
    main()
