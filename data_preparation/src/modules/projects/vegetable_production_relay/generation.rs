use super::{OUTPUT_DIR, acquisition, aggregation, metadata, normalization, product_index};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const YEARS: [&str; 3] = ["2021", "2022", "2023"];
const ROOT: &str = "inspections/vegetable_production_relay";
const INVENTORY: &str = "inspections/vegetable_production_relay/product_inventory_2021_2023.json";

fn fail(message: impl Into<String>) -> Box<dyn Error> {
    message.into().into()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn write_inventory(inventory: &Value) -> Result<()> {
    fs::write(INVENTORY, serde_json::to_vec_pretty(inventory)?)?;
    let mut file = fs::OpenOptions::new().append(true).open(INVENTORY)?;
    use std::io::Write;
    file.write_all(b"\n")?;
    Ok(())
}

fn newest_observation_runs(runs_dir: &Path) -> Result<HashMap<String, PathBuf>> {
    let mut newest: HashMap<String, (SystemTime, PathBuf)> = HashMap::new();
    for entry in fs::read_dir(runs_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir()
            || !entry
                .file_name()
                .to_string_lossy()
                .starts_with("observations-")
        {
            continue;
        }
        let run = entry.path();
        let ledger_path = run.join("ledger.json");
        let Ok(ledger) = read_json(&ledger_path) else {
            continue;
        };
        let Some(table_id) = ledger.get("table_id").and_then(Value::as_str) else {
            continue;
        };
        let modified = entry.metadata()?.modified().unwrap_or(UNIX_EPOCH);
        if newest
            .get(table_id)
            .is_none_or(|(existing, _)| modified > *existing)
        {
            newest.insert(table_id.to_owned(), (modified, run));
        }
    }
    Ok(newest
        .into_iter()
        .map(|(table_id, (_, path))| (table_id, path))
        .collect())
}

fn newest_metadata_runs(runs_dir: &Path) -> Result<HashMap<String, PathBuf>> {
    let mut newest: HashMap<String, (SystemTime, PathBuf)> = HashMap::new();
    for entry in fs::read_dir(runs_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir()
            || !entry.file_name().to_string_lossy().starts_with("metadata-")
        {
            continue;
        }
        let run = entry.path();
        let Ok(capture) = metadata::replay(&run) else {
            continue;
        };
        let modified = entry.metadata()?.modified().unwrap_or(UNIX_EPOCH);
        if newest
            .get(&capture.stats_data_id)
            .is_none_or(|(existing, _)| modified > *existing)
        {
            newest.insert(capture.stats_data_id, (modified, run));
        }
    }
    Ok(newest
        .into_iter()
        .map(|(table_id, (_, path))| (table_id, path))
        .collect())
}

fn replay_metadata_for_table(
    path: PathBuf,
    table_id: &str,
) -> Option<(PathBuf, metadata::Inventory)> {
    metadata::replay(&path)
        .ok()
        .filter(|capture| capture.stats_data_id == table_id)
        .map(|capture| (path, capture))
}

struct CaptureTask {
    product_index: usize,
    year: String,
    table_id: String,
    metadata_run: PathBuf,
}

struct CaptureResult {
    product_index: usize,
    year: String,
    run: PathBuf,
    report: acquisition::SourceReport,
}

fn ensure_product_metadata_captures(
    inventory: &mut Value,
    app_id: &str,
    persist_inventory: bool,
) -> Result<()> {
    let runs_dir = Path::new(ROOT).join("runs");
    let mut cached_runs = None;
    let product_count = inventory
        .get("products")
        .and_then(Value::as_array)
        .ok_or_else(|| fail("product inventory is missing products"))?
        .len();

    for product_index in 0..product_count {
        let key = value_string(&inventory["products"][product_index], "key")?.to_owned();
        for year in YEARS {
            let table_id = inventory["products"][product_index]["table_ids"][year]
                .as_str()
                .ok_or_else(|| fail(format!("{key} {year}: table ID is missing")))?
                .to_owned();
            let existing_run_id =
                inventory["products"][product_index]["metadata_runs"][year]["run_id"]
                    .as_str()
                    .map(str::to_owned);

            let existing = existing_run_id
                .as_deref()
                .map(|run_id| runs_dir.join(run_id))
                .and_then(|path| replay_metadata_for_table(path, &table_id));
            let existing = match existing {
                Some(existing) => Some(existing),
                None => {
                    if cached_runs.is_none() {
                        cached_runs = Some(newest_metadata_runs(&runs_dir)?);
                    }
                    cached_runs
                        .as_ref()
                        .and_then(|runs| runs.get(&table_id))
                        .cloned()
                        .and_then(|path| replay_metadata_for_table(path, &table_id))
                }
            };
            let (run, capture) = match existing {
                Some(existing) => existing,
                None => {
                    println!("fetching metadata for {key} {year} ({table_id})");
                    let run = metadata::capture(app_id, &table_id)?;
                    let capture = metadata::replay(&run)?;
                    (run, capture)
                }
            };
            let run_id = run
                .file_name()
                .ok_or_else(|| fail(format!("{key} {year}: metadata run has no name")))?
                .to_string_lossy()
                .into_owned();
            let record = json!({
                "run_id": run_id,
                "metadata_sha256": capture.source_sha256,
                "inventory_status": capture.status,
            });
            if inventory["products"][product_index]["metadata_runs"][year] != record {
                inventory["products"][product_index]["metadata_runs"][year] = record;
                if persist_inventory {
                    write_inventory(inventory)?;
                }
            }
        }
    }
    Ok(())
}

fn capture_table(
    task: &CaptureTask,
    app_id: &str,
    existing_runs: &HashMap<String, PathBuf>,
) -> std::result::Result<CaptureResult, String> {
    if let Some(run) = existing_runs.get(&task.table_id) {
        if let Ok(report) = acquisition::replay(run, &task.metadata_run) {
            return Ok(CaptureResult {
                product_index: task.product_index,
                year: task.year.clone(),
                run: run.clone(),
                report,
            });
        }
    }

    let run = acquisition::acquire(app_id, &task.metadata_run).map_err(|e| e.to_string())?;
    let report = acquisition::replay(&run, &task.metadata_run).map_err(|e| e.to_string())?;
    Ok(CaptureResult {
        product_index: task.product_index,
        year: task.year.clone(),
        run,
        report,
    })
}

/// Acquire or reuse replay-verified e-Stat observations and refresh the source inventory.
pub fn acquire_product_observations() -> Result<()> {
    let app_id = std::env::var("ESTAT_APP_ID").map_err(|_| fail("ESTAT_APP_ID is required"))?;
    let inventory = read_json(Path::new(INVENTORY))?;
    acquire_product_observations_with_inventory(inventory, &app_id, true)?;
    Ok(())
}

fn acquire_product_observations_with_inventory(
    mut inventory: Value,
    app_id: &str,
    persist_inventory: bool,
) -> Result<Value> {
    ensure_product_metadata_captures(&mut inventory, app_id, persist_inventory)?;
    let products = inventory
        .get("products")
        .and_then(Value::as_array)
        .ok_or_else(|| fail("product inventory is missing products"))?;
    let runs_dir = Path::new(ROOT).join("runs");
    let existing_runs = newest_observation_runs(&runs_dir)?;
    let mut tasks = Vec::new();

    for (product_index, product) in products.iter().enumerate() {
        let key = product
            .get("key")
            .and_then(Value::as_str)
            .ok_or_else(|| fail("product inventory entry is missing key"))?;
        for (year, table_id) in product
            .get("table_ids")
            .and_then(Value::as_object)
            .ok_or_else(|| fail(format!("{key}: table_ids missing")))?
        {
            let metadata_run_id = product
                .get("metadata_runs")
                .and_then(|runs| runs.get(year))
                .and_then(|run| run.get("run_id"))
                .and_then(Value::as_str)
                .ok_or_else(|| fail(format!("{key} {year}: metadata capture is not linked")))?;
            let table_id = table_id
                .as_str()
                .ok_or_else(|| fail(format!("{key} {year}: table ID is invalid")))?;
            tasks.push(CaptureTask {
                product_index,
                year: year.clone(),
                table_id: table_id.to_owned(),
                metadata_run: runs_dir.join(metadata_run_id),
            });
        }
    }

    let completed = Mutex::new(Vec::with_capacity(tasks.len()));
    let workers = 4.min(tasks.len().max(1));
    thread::scope(|scope| {
        for worker in 0..workers {
            let completed = &completed;
            let existing_runs = &existing_runs;
            let tasks = &tasks;
            let app_id = &app_id;
            scope.spawn(move || {
                for task in tasks.iter().skip(worker).step_by(workers) {
                    let result = capture_table(task, app_id, existing_runs);
                    completed
                        .lock()
                        .expect("capture result lock poisoned")
                        .push(result);
                }
            });
        }
    });
    let results = completed
        .into_inner()
        .map_err(|_| fail("capture result lock poisoned"))?;
    let mut errors = Vec::new();
    for result in results {
        match result {
            Ok(capture) => {
                let source_path = capture.run.join("source_observations.json");
                let source_hash = sha256(&fs::read(&source_path)?);
                let product = &mut inventory["products"][capture.product_index];
                product
                    .as_object_mut()
                    .ok_or_else(|| fail("product inventory entry is not an object"))?
                    .entry("observation_runs")
                    .or_insert_with(|| json!({}))[&capture.year] = json!({
                    "run_id": capture.run.file_name().unwrap_or_default().to_string_lossy(),
                    "observation_sha256": source_hash,
                    "count": capture.report.count,
                    "metadata_sha256": capture.report.source_metadata_sha256,
                });
            }
            Err(error) => errors.push(error),
        }
    }
    if persist_inventory {
        write_inventory(&inventory)?;
    }
    if !errors.is_empty() {
        return Err(fail(format!(
            "{} product observation capture(s) failed:\n{}",
            errors.len(),
            errors.join("\n")
        )));
    }
    println!(
        "Captured or replayed {} product/year observations",
        tasks.len()
    );
    Ok(inventory)
}

fn value_string<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| fail(format!("missing or invalid {key}")))
}

fn timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

/// Normalize all verified products, aggregate their years, and refresh the frontend index.
pub fn generate_product_data(inventory: &Value) -> Result<()> {
    let products = inventory
        .get("products")
        .and_then(Value::as_array)
        .ok_or_else(|| fail("product inventory is missing products"))?;
    let runs_dir = Path::new(ROOT).join("runs");
    let mappings_dir = Path::new(ROOT).join("product_temporal_mappings");
    let data_dir = Path::new(OUTPUT_DIR);
    fs::create_dir_all(data_dir)?;

    for product in products {
        let key = value_string(product, "key")?;
        let label = value_string(product, "label")?;
        if !matches!(
            product.get("mapping_status").and_then(Value::as_str),
            Some("verified" | "verified_with_equivalent_blocks")
        ) {
            return Err(fail(format!("{key}: product mapping is not verified")));
        }
        let mapping_path = mappings_dir.join(format!("{key}.json"));
        if !mapping_path.is_file() {
            return Err(fail(format!("{key}: temporal mapping is missing")));
        }
        let mapping_sha = sha256(&fs::read(&mapping_path)?);

        let mut normalized_paths = Vec::new();
        let mut normalized_hashes = Vec::new();
        let unique = timestamp();
        for year in YEARS {
            let observation_run = product
                .get("observation_runs")
                .and_then(|runs| runs.get(year))
                .and_then(|run| run.get("run_id"))
                .and_then(Value::as_str)
                .ok_or_else(|| fail(format!("{key} {year}: observation capture is missing")))?;
            let metadata_run = product
                .get("metadata_runs")
                .and_then(|runs| runs.get(year))
                .and_then(|run| run.get("run_id"))
                .and_then(Value::as_str)
                .ok_or_else(|| fail(format!("{key} {year}: metadata capture is missing")))?;
            let output_dir = runs_dir
                .join("product-normalization")
                .join(key)
                .join(format!("{year}-{unique}"));
            fs::create_dir_all(&output_dir)?;
            let output = normalization::run_2021_2023(
                &runs_dir.join(observation_run),
                &runs_dir.join(metadata_run),
                &mapping_path,
                &output_dir,
            )?;
            normalized_hashes.push(sha256(&fs::read(&output)?));
            normalized_paths.push(output);
        }

        let product_json = data_dir.join(format!("{key}.json"));
        if product_json.exists() {
            let existing = read_json(&product_json)?;
            let existing_years = existing
                .get("years")
                .and_then(Value::as_array)
                .ok_or_else(|| fail(format!("{key}: existing output has no yearly data")))?;
            if existing.get("commodity").and_then(Value::as_str) != Some(key)
                || !matches!(
                    existing.get("status").and_then(Value::as_str),
                    Some("valid" | "incomplete")
                )
                || existing_years.len() != YEARS.len()
            {
                return Err(fail(format!(
                    "{key}: existing output is not a usable product file"
                )));
            }
            let same_mapping = existing_years.iter().all(|year| {
                year.get("temporal_mapping_sha256").and_then(Value::as_str)
                    == Some(mapping_sha.as_str())
            });
            let same_inputs =
                existing_years
                    .iter()
                    .zip(&normalized_hashes)
                    .all(|(year, input_hash)| {
                        year.get("normalized_input_sha256").and_then(Value::as_str)
                            == Some(input_hash.as_str())
                    });
            if existing.get("commodity_label").and_then(Value::as_str) == Some(label)
                && same_mapping
                && same_inputs
            {
                println!(
                    "kept existing {}",
                    product_json
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                );
                continue;
            }
            fs::remove_file(&product_json)?;
        }

        aggregation::run_2021_2023_product(
            &normalized_paths[0],
            &normalized_paths[1],
            &normalized_paths[2],
            &runs_dir,
            &mapping_path,
            &product_json,
            key,
            label,
        )?;
        println!(
            "generated {}",
            product_json
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        );
    }

    let index = product_index::generate(data_dir)?;
    println!("Generated product index: {}", index.display());
    Ok(())
}

/// Run the full Rust data workflow: capture observations, then generate product files.
pub fn run() -> Result<()> {
    let app_id = std::env::var("ESTAT_APP_ID").map_err(|_| fail("ESTAT_APP_ID is required"))?;
    let inventory = read_json(Path::new(INVENTORY))?;
    let inventory = acquire_product_observations_with_inventory(inventory, &app_id, false)?;
    generate_product_data(&inventory)
}
