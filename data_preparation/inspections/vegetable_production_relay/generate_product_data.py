"""Normalize cached source captures and generate one JSON per mapped product.

Run after product temporal mappings are verified. Existing product JSON files are
not overwritten; reruns can reuse the cached captures and put normalization outputs
in fresh inspection directories.
"""

import hashlib
import json
import os
import subprocess
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parent
RUNS = ROOT / "runs"
INVENTORY = ROOT / "product_inventory_2021_2023.json"
MAPPINGS = ROOT / "product_temporal_mappings"
DATA_DIR = Path(__file__).resolve().parents[2] / "output" / "vegetable_production_relay"
EXECUTABLE = Path(
    os.environ.get(
        "OPEN_DATA_VIZ_EXECUTABLE",
        Path(__file__).resolve().parents[2] / "target" / "debug" / "open_data_viz_2",
    )
)
YEARS = ["2021", "2022", "2023"]


def run(command):
    result = subprocess.run(command, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(result.stderr.strip()[-1000:] or result.stdout.strip()[-1000:])
    return result.stdout.strip()


def main():
    inventory = json.loads(INVENTORY.read_text())
    DATA_DIR.mkdir(parents=True, exist_ok=True)
    for product in inventory["products"]:
        if product.get("mapping_status") not in (
            "verified",
            "verified_with_equivalent_blocks",
        ):
            raise RuntimeError(f"{product['key']}: product mapping is not verified")
        mapping = MAPPINGS / f"{product['key']}.json"
        if not mapping.is_file():
            raise RuntimeError(f"{product['key']}: temporal mapping is missing")
        mapping_sha = hashlib.sha256(mapping.read_bytes()).hexdigest()

        normalized = []
        for year in YEARS:
            observation = product.get("observation_runs", {}).get(year)
            metadata = product.get("metadata_runs", {}).get(year)
            if not observation or not metadata:
                raise RuntimeError(f"{product['key']} {year}: cached source references are missing")
            output_dir = RUNS / "product-normalization" / product["key"] / f"{year}-{time.time_ns()}"
            output_dir.mkdir(parents=True, exist_ok=False)
            run(
                [
                    str(EXECUTABLE),
                    "vegetable-relay-normalize-2021-2023",
                    str(RUNS / observation["run_id"]),
                    str(RUNS / metadata["run_id"]),
                    str(mapping),
                    str(output_dir),
                ]
            )
            normalized.append(str(output_dir / "m5b_inspection_only.json"))
        normalized_hashes = [
            hashlib.sha256(Path(path).read_bytes()).hexdigest() for path in normalized
        ]

        product_json = DATA_DIR / f"{product['key']}.json"
        if product_json.exists():
            existing = json.loads(product_json.read_text())
            if (
                existing.get("commodity") != product["key"]
                or existing.get("status") not in ("valid", "incomplete")
                or len(existing.get("years", [])) != 3
            ):
                raise RuntimeError(f"{product['key']}: existing output is not a usable product file")
            same_mapping = all(
                year.get("temporal_mapping_sha256") == mapping_sha
                for year in existing["years"]
            )
            same_inputs = all(
                year.get("normalized_input_sha256") == input_hash
                for year, input_hash in zip(existing["years"], normalized_hashes)
            )
            if (
                existing.get("commodity_label") == product["label"]
                and same_mapping
                and same_inputs
            ):
                print(f"kept existing {product_json.name}", flush=True)
                continue
            product_json.unlink()

        run(
            [
                str(EXECUTABLE),
                "vegetable-relay-aggregate-2021-2023",
                *normalized,
                str(RUNS),
                str(mapping),
                str(DATA_DIR),
                product["key"],
                product["label"],
            ]
        )
        print(f"generated {product_json.name}", flush=True)

    run([str(EXECUTABLE), "vegetable-relay-product-index", str(DATA_DIR)])
    print(f"generated {DATA_DIR / 'products.json'}")


if __name__ == "__main__":
    main()
