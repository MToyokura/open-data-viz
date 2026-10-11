"""Acquire and replay product table observations for the 2021–2023 inventory.

Requires ESTAT_APP_ID. Captures are raw, replay-verified evidence only; this script
does not assign calendar meaning to cat03 blocks or generate display data.
"""

import concurrent.futures
import hashlib
import json
import os
import re
import subprocess
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parent
RUNS = ROOT / "runs"
INVENTORY = ROOT / "product_inventory_2021_2023.json"
EXECUTABLE = Path(__file__).resolve().parents[2] / "target" / "debug" / "open_data_viz_2"


def capture_for_table(table_id, metadata_run):
    existing = []
    for ledger_path in RUNS.glob("observations-*/ledger.json"):
        ledger = json.loads(ledger_path.read_text())
        if ledger.get("table_id") == table_id:
            existing.append(ledger_path.parent)
    if existing:
        run = max(existing, key=lambda path: path.stat().st_mtime_ns)
        replay = subprocess.run(
            [str(EXECUTABLE), "vegetable-relay-acquisition-replay", str(run), str(metadata_run)],
            text=True,
            capture_output=True,
        )
        if replay.returncode == 0:
            source_path = run / "source_observations.json"
            return run, source_path

    result = subprocess.run(
        [str(EXECUTABLE), "vegetable-relay-acquire", str(metadata_run)],
        text=True,
        capture_output=True,
    )
    if result.returncode:
        raise RuntimeError(f"{table_id}: acquisition failed: {result.stderr.strip()[-500:]}")
    match = re.search(r"Captured raw e-Stat observations: (.+)", result.stdout)
    if not match:
        raise RuntimeError(f"{table_id}: acquisition output did not include a run directory")
    run = Path(match.group(1).strip())
    replay = subprocess.run(
        [str(EXECUTABLE), "vegetable-relay-acquisition-replay", str(run), str(metadata_run)],
        text=True,
        capture_output=True,
    )
    if replay.returncode:
        raise RuntimeError(f"{table_id}: offline replay failed: {replay.stderr.strip()[-500:]}")
    return run, run / "source_observations.json"


def main():
    if not os.environ.get("ESTAT_APP_ID"):
        raise RuntimeError("ESTAT_APP_ID is required")
    if not EXECUTABLE.is_file():
        raise RuntimeError("build data_preparation first")
    inventory = json.loads(INVENTORY.read_text())
    tasks = []
    for product in inventory["products"]:
        for year, table_id in product["table_ids"].items():
            metadata = product.get("metadata_runs", {}).get(year)
            if not metadata:
                raise RuntimeError(f"{product['key']} {year}: metadata capture is not linked")
            metadata_run = RUNS / metadata["run_id"]
            tasks.append((product, year, table_id, metadata_run))

    errors = []
    complete = 0
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:
        futures = {
            executor.submit(capture_for_table, table_id, metadata_run): (product, year, table_id)
            for product, year, table_id, metadata_run in tasks
        }
        for future in concurrent.futures.as_completed(futures):
            product, year, table_id = futures[future]
            try:
                run, source_path = future.result()
                source = json.loads(source_path.read_text())
                product.setdefault("observation_runs", {})[year] = {
                    "run_id": run.name,
                    "observation_sha256": hashlib.sha256(source_path.read_bytes()).hexdigest(),
                    "count": source["count"],
                    "metadata_sha256": source["source_metadata_sha256"],
                }
                complete += 1
            except Exception as error:
                errors.append((table_id, str(error)))
            if complete % 5 == 0 and complete:
                INVENTORY.write_text(json.dumps(inventory, ensure_ascii=False, indent=2) + "\n")
                print(f"captured and replayed {complete}/{len(tasks)} product tables", flush=True)
            time.sleep(0.05)

    INVENTORY.write_text(json.dumps(inventory, ensure_ascii=False, indent=2) + "\n")
    print(f"finished {complete}/{len(tasks)} tables; errors: {len(errors)}")
    for table_id, error in errors:
        print(table_id, error)
    if errors:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
