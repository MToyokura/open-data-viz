"""Compare official dated daikon spreadsheets with saved 2021–2023 API rows.

Download source files listed in 2021_2023_temporal_mapping.json to the ignored
runs/official-daikon-2021-2023/<year>/<period-index>.xlsx directories first.
Uses only the Python standard library. Does not write or publish data.
"""

import hashlib
import json
import re
import unicodedata
import xml.etree.ElementTree as ET
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent
NS = {"m": "http://schemas.openxmlformats.org/spreadsheetml/2006/main"}


def spreadsheet_cells(path):
    with zipfile.ZipFile(path) as archive:
        strings = []
        if "xl/sharedStrings.xml" in archive.namelist():
            root = ET.fromstring(archive.read("xl/sharedStrings.xml"))
            strings = [
                "".join(t.text or "" for t in item.findall(".//m:t", NS))
                for item in root.findall("m:si", NS)
            ]
        root = ET.fromstring(archive.read("xl/worksheets/sheet1.xml"))
        cells = {}
        for cell in root.findall(".//m:sheetData/m:row/m:c", NS):
            value = cell.find("m:v", NS)
            inline = cell.find("m:is", NS)
            if value is not None:
                cells[cell.attrib["r"]] = (
                    strings[int(value.text)] if cell.get("t") == "s" else value.text
                )
            elif inline is not None:
                cells[cell.attrib["r"]] = "".join(
                    t.text or "" for t in inline.findall(".//m:t", NS)
                )
        return cells


def column(number):
    result = ""
    while number:
        number, remainder = divmod(number - 1, 26)
        result = chr(65 + remainder) + result
    return result


def compare(year, record, rows):
    period = record["period"]
    index = 0 if period == "total" else int(period[-2:])
    path = ROOT / "runs/official-daikon-2021-2023" / year / f"{index:02}.xlsx"
    assert hashlib.sha256(path.read_bytes()).hexdigest() == record["spreadsheet_sha256"]
    cells = spreadsheet_cells(path)
    assert "だいこん" in cells["A4"]
    title = unicodedata.normalize("NFKC", cells["A4"])
    assert title.endswith("計" if index == 0 else f"{index}月")
    origin_rows = {
        int(value): int(re.search(r"\d+", address).group())
        for address, value in cells.items()
        if re.fullmatch(r"B\d+", address)
        and re.fullmatch(r"\d+", str(value))
        and 1 <= int(value) <= 48
    }
    assert set(origin_rows) == set(range(1, 49))
    scores = []
    for block in range(1, 14):
        matches = 0
        for origin, row in origin_rows.items():
            for region in range(12):
                for measure in range(2):
                    value = cells.get(f"{column(3 + region * 2 + measure)}{row}")
                    api = rows[(str(1000 + origin), str(1001 + measure), str(989 + block * 12 + region))]
                    assert value is not None
                    matches += str(value) == api
        scores.append((matches, block))
    scores.sort(reverse=True)
    assert scores[0] == (1152, record["block"])
    assert scores[1][0] == record["runner_up_matches"]
    return scores[0], scores[1]


def main():
    mapping = json.loads((ROOT / "2021_2023_temporal_mapping.json").read_text())
    for year, source in mapping["years"].items():
        report = json.loads((ROOT / source["raw_capture"] / "source_observations.json").read_text())
        rows = {
            (r["attributes"]["@cat01"], r["attributes"]["@cat02"], r["attributes"]["@cat03"]): r["raw_value"]
            for r in report["rows"]
        }
        assert len(rows) == 14976
        for record in source["periods"]:
            best, second = compare(year, record, rows)
            print(year, record["period"], best, second)


if __name__ == "__main__":
    main()
