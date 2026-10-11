"""Cache the official period workbooks used to identify each product's 13 cat03 blocks.

This records workbook references and checksums only. It does not infer block-to-period
mappings; run the comparison tool after matching these workbooks to replayed API rows.
"""

import concurrent.futures
import hashlib
import html
import json
import re
import unicodedata
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent
RUNS = ROOT / "runs" / "official-product-workbooks"
INVENTORY = ROOT / "product_inventory_2021_2023.json"
LISTINGS = {
    "2021": ("000001169106", "20210"),
    "2022": ("000001209867", "20220"),
    "2023": ("000001220580", "20230"),
}
BASE = "https://www.e-stat.go.jp"


def sha256(content):
    return hashlib.sha256(content).hexdigest()


def fetch(url):
    request = urllib.request.Request(url, headers={"User-Agent": "open-data-viz-source-validation/1.0"})
    with urllib.request.urlopen(request, timeout=60) as response:
        if response.status != 200:
            raise RuntimeError(f"HTTP {response.status}: {url}")
        return response.read()


def listing_url(year, page):
    class_id, year_value = LISTINGS[year]
    query = urllib.parse.urlencode(
        {
            "cycle": "7",
            "layout": "datalist",
            "page": page,
            "tclass1": "000001020455",
            "tclass2": class_id,
            "tclass3val": "0",
            "toukei": "00500226",
            "tstat": "000001015623",
            "year": year_value,
        }
    )
    return f"{BASE}/stat-search/files?{query}"


def parse_listing(content, labels):
    text = content.decode("utf-8", "replace")
    records = {}
    for article in re.findall(r'<article class="stat-dataset_list-item">.*?</article>', text, re.S):
        period = re.search(r'data-value="(\d{12})"[^>]*>\s*(.*?)\s*</a>', article, re.S)
        excel = re.search(
            r'href="([^"]*file-download\?[^"]*fileKind=0[^"]*)"[^>]*data-file_type="EXCEL"',
            article,
            re.S,
        )
        if not period or not excel:
            continue
        spans = re.findall(r"<span[^>]*>(.*?)</span>", article[: period.start()], re.S)
        values = [html.unescape(re.sub(r"<.*?>", "", value)).replace("\xa0", " ").strip() for value in spans]
        values = [value for value in values if value and value != "表番号"]
        if not values:
            continue
        label = values[-1]
        if label not in labels:
            continue
        period_label = unicodedata.normalize(
            "NFKC", html.unescape(re.sub(r"<.*?>", "", period.group(2))).strip()
        )
        if period_label == "計":
            period_key = "total"
        else:
            month = re.fullmatch(r"(\d{1,2})月", period_label)
            if not month or not 1 <= int(month.group(1)) <= 12:
                raise RuntimeError(f"unrecognized period label {period_label!r} for {label}")
            period_key = f"{int(month.group(1)):02}"
        stat_inf_id = period.group(1)
        file_url = urllib.parse.urljoin(BASE, html.unescape(excel.group(1)))
        query = urllib.parse.parse_qs(urllib.parse.urlparse(file_url).query)
        if query.get("statInfId") != [stat_inf_id]:
            raise RuntimeError(f"workbook/table reference mismatch for {label} {period_key}")
        key = (label, period_key)
        if key in records:
            raise RuntimeError(f"duplicate official workbook listing: {key}")
        records[key] = {"stat_inf_id": stat_inf_id, "url": file_url}
    return records


def download_one(task):
    year, key, period, source, path = task
    if path.exists():
        content = path.read_bytes()
    else:
        content = fetch(source["url"])
    if len(content) < 1000 or not content.startswith(b"PK"):
        raise RuntimeError(f"invalid Excel workbook bytes: {year} {key} {period}")
    if not path.exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        temporary = path.with_suffix(".xlsx.tmp")
        temporary.write_bytes(content)
        temporary.replace(path)
    if not zipfile.is_zipfile(path):
        raise RuntimeError(f"cached workbook is not a ZIP archive: {path}")
    return {
        "year": year,
        "product_key": key,
        "period": period,
        "stat_inf_id": source["stat_inf_id"],
        "url": source["url"],
        "path": str(path.relative_to(ROOT)),
        "sha256": sha256(content),
        "byte_count": len(content),
    }


def main():
    inventory = json.loads(INVENTORY.read_text())
    products = inventory["products"]
    labels = {product["label"] for product in products}
    records_by_year = {}
    listing_hashes = {}
    for year in LISTINGS:
        records = {}
        for page in range(1, 10):
            url = listing_url(year, page)
            content = fetch(url)
            listing_dir = RUNS / "listings"
            listing_dir.mkdir(parents=True, exist_ok=True)
            (listing_dir / f"{year}-{page:02}.html").write_bytes(content)
            listing_hashes[f"{year}-{page:02}"] = {"url": url, "sha256": sha256(content)}
            page_records = parse_listing(content, labels)
            if not page_records:
                break
            overlap = records.keys() & page_records.keys()
            if overlap:
                raise RuntimeError(f"duplicate product periods across listing pages: {sorted(overlap)[:3]}")
            records.update(page_records)
        expected = {(product["label"], period) for product in products for period in ["total", *[f"{month:02}" for month in range(1, 13)]]}
        if records.keys() != expected:
            missing = sorted(expected - records.keys())
            extra = sorted(records.keys() - expected)
            raise RuntimeError(f"{year} workbook list incomplete: missing={missing[:5]} extra={extra[:5]}")
        records_by_year[year] = records

    tasks = []
    for year, records in records_by_year.items():
        for product in products:
            for period in ["total", *[f"{month:02}" for month in range(1, 13)]]:
                source = records[(product["label"], period)]
                path = RUNS / year / product["key"] / f"{period}.xlsx"
                tasks.append((year, product["key"], period, source, path))

    results = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as executor:
        futures = [executor.submit(download_one, task) for task in tasks]
        for index, future in enumerate(concurrent.futures.as_completed(futures), 1):
            results.append(future.result())
            if index % 100 == 0:
                print(f"cached {index}/{len(tasks)} official workbooks", flush=True)

    manifest = {
        "schema_version": 1,
        "status": "WORKBOOKS_CACHED_MAPPING_PENDING",
        "product_count": len(products),
        "workbook_count": len(results),
        "source_listings": listing_hashes,
        "workbooks": sorted(results, key=lambda record: (record["product_key"], record["year"], record["period"])),
    }
    output = ROOT / "product_workbook_manifest_2021_2023.json"
    output.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    print(f"saved {len(results)} workbook references and hashes to {output}")


if __name__ == "__main__":
    main()
