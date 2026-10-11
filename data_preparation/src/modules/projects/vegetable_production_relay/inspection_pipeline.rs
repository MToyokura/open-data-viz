use super::xlsx::{column_name, is_zip_archive, spreadsheet_cells};
use super::{acquisition, normalization};
use regex::Regex;
use reqwest::blocking::Client;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    error::Error,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    thread,
    time::Duration,
};
use unicode_normalization::UnicodeNormalization;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const ROOT: &str = "inspections/vegetable_production_relay";
const RUNS: &str = "inspections/vegetable_production_relay/runs";
const YEARS: [&str; 3] = ["2021", "2022", "2023"];
const PERIODS: [&str; 13] = [
    "total", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12",
];
const BASE: &str = "https://www.e-stat.go.jp";

fn fail(message: impl Into<String>) -> Box<dyn Error> {
    message.into().into()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    let mut file = fs::OpenOptions::new().append(true).open(path)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn html_unescape(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", "\u{00a0}")
        .replace("&#160;", "\u{00a0}")
}

fn strip_tags(value: &str) -> Result<String> {
    Ok(Regex::new(r"(?s)<.*?>")?
        .replace_all(value, "")
        .into_owned())
}

#[derive(Clone)]
struct WorkbookSource {
    table_id: String,
    url: String,
}

fn parse_listing(
    content: &[u8],
    labels: &BTreeSet<String>,
) -> Result<BTreeMap<(String, String), WorkbookSource>> {
    let text = String::from_utf8_lossy(content);
    let article_re = Regex::new(r#"(?s)<article class="stat-dataset_list-item">.*?</article>"#)?;
    let period_re = Regex::new(r#"(?s)data-value="(\d{12})"[^>]*>\s*(.*?)\s*</a>"#)?;
    let excel_re = Regex::new(
        r#"href="([^"]*file-download\?[^"]*fileKind=0[^"]*)"[^>]*data-file_type="EXCEL""#,
    )?;
    let span_re = Regex::new(r"(?s)<span[^>]*>(.*?)</span>")?;
    let tag_re = Regex::new(r"(?s)<.*?>")?;
    let mut records = BTreeMap::new();

    for article_match in article_re.find_iter(&text) {
        let article = article_match.as_str();
        let Some(period) = period_re.captures(article) else {
            continue;
        };
        let Some(excel) = excel_re.captures(article) else {
            continue;
        };
        let period_match = period.get(0).ok_or("period capture missing")?;
        let mut labels_in_article = Vec::new();
        for span in span_re.captures_iter(&article[..period_match.start()]) {
            let stripped = tag_re.replace_all(&span[1], "");
            let value = html_unescape(&stripped).replace('\u{00a0}', " ");
            let value = value.trim();
            if !value.is_empty() && value != "表番号" {
                labels_in_article.push(value.to_owned());
            }
        }
        let Some(label) = labels_in_article
            .last()
            .filter(|label| labels.contains(*label))
        else {
            continue;
        };
        let period_label = strip_tags(&period[2])?;
        let period_label = html_unescape(period_label.trim())
            .nfkc()
            .collect::<String>();
        let period_key = if period_label == "計" {
            "total".to_owned()
        } else {
            let month_re = Regex::new(r"^(\d{1,2})月$")?;
            let month = month_re
                .captures(&period_label)
                .and_then(|c| c[1].parse::<u8>().ok())
                .filter(|month| (1..=12).contains(month))
                .ok_or_else(|| {
                    fail(format!(
                        "unrecognized period label {period_label:?} for {label}"
                    ))
                })?;
            format!("{month:02}")
        };
        let table_id = period[1].to_owned();
        let href = html_unescape(&excel[1]);
        let url = if href.starts_with("http://") || href.starts_with("https://") {
            href
        } else if href.starts_with('/') {
            format!("{BASE}{href}")
        } else {
            format!("{BASE}/{href}")
        };
        let stat_inf_re = Regex::new(r"(?:\?|&)statInfId=([^&]+)")?;
        if stat_inf_re
            .captures(&url)
            .map(|c| c[1].to_owned())
            .as_deref()
            != Some(&table_id)
        {
            return Err(fail(format!(
                "workbook/table reference mismatch for {label} {period_key}"
            )));
        }
        let key = (label.clone(), period_key.clone());
        if records
            .insert(key.clone(), WorkbookSource { table_id, url })
            .is_some()
        {
            return Err(fail(format!(
                "duplicate official workbook listing: {key:?}"
            )));
        }
    }
    Ok(records)
}

fn listing_url(year: &str, page: usize) -> Result<String> {
    let (class_id, year_value) = match year {
        "2021" => ("000001169106", "20210"),
        "2022" => ("000001209867", "20220"),
        "2023" => ("000001220580", "20230"),
        _ => return Err(fail(format!("unsupported year {year}"))),
    };
    Ok(format!(
        "{BASE}/stat-search/files?cycle=7&layout=datalist&page={page}&tclass1=000001020455&tclass2={class_id}&tclass3val=0&toukei=00500226&tstat=000001015623&year={year_value}"
    ))
}

fn http_client(agent: &str) -> Result<Client> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(60))
        .user_agent(agent)
        .build()?)
}

fn fetch(client: &Client, url: &str) -> Result<Vec<u8>> {
    let response = client.get(url).send()?;
    let status = response.status();
    if !status.is_success() {
        return Err(fail(format!("HTTP {status}: {url}")));
    }
    Ok(response.bytes()?.to_vec())
}

struct DownloadTask {
    year: String,
    key: String,
    period: String,
    source: WorkbookSource,
    path: PathBuf,
}

fn download_workbook(client: &Client, task: &DownloadTask, root: &Path) -> Result<Value> {
    let content = if task.path.exists() {
        fs::read(&task.path)?
    } else {
        fetch(client, &task.source.url)?
    };
    if content.len() < 1000 || !is_zip_archive(&content) {
        return Err(fail(format!(
            "invalid Excel workbook bytes: {} {} {}",
            task.year, task.key, task.period
        )));
    }
    if !task.path.exists() {
        let parent = task.path.parent().ok_or("workbook path has no parent")?;
        fs::create_dir_all(parent)?;
        let temporary = task.path.with_extension("xlsx.tmp");
        fs::write(&temporary, &content)?;
        fs::rename(&temporary, &task.path)?;
    }
    let relative = task.path.strip_prefix(root)?;
    Ok(json!({
        "year": task.year,
        "product_key": task.key,
        "period": task.period,
        "stat_inf_id": task.source.table_id,
        "url": task.source.url,
        "path": relative.to_string_lossy(),
        "sha256": sha256(&content),
        "byte_count": content.len(),
    }))
}

/// Download/cache all official period workbooks used for temporal mapping inspection.
pub fn acquire_product_workbooks() -> Result<()> {
    let root = Path::new(ROOT);
    let inventory = read_json(&root.join("product_inventory_2021_2023.json"))?;
    let products = inventory
        .get("products")
        .and_then(Value::as_array)
        .ok_or_else(|| fail("product inventory is missing products"))?;
    let mut labels = BTreeSet::new();
    for product in products {
        labels.insert(
            product
                .get("label")
                .and_then(Value::as_str)
                .ok_or("product label missing")?
                .to_owned(),
        );
    }

    let client = http_client("open-data-viz-source-validation/1.0")?;
    let runs = root.join("runs/official-product-workbooks");
    let mut records_by_year = BTreeMap::new();
    let mut listing_hashes = BTreeMap::new();
    for year in YEARS {
        let mut records = BTreeMap::new();
        for page in 1..=9 {
            let url = listing_url(year, page)?;
            let content = fetch(&client, &url)?;
            let listing_dir = runs.join("listings");
            fs::create_dir_all(&listing_dir)?;
            fs::write(listing_dir.join(format!("{year}-{page:02}.html")), &content)?;
            listing_hashes.insert(
                format!("{year}-{page:02}"),
                json!({"url": url, "sha256": sha256(&content)}),
            );
            let page_records = parse_listing(&content, &labels)?;
            if page_records.is_empty() {
                break;
            }
            if records.keys().any(|key| page_records.contains_key(key)) {
                return Err(fail(format!(
                    "duplicate product periods across listing pages in {year}"
                )));
            }
            records.extend(page_records);
        }
        let expected: BTreeSet<_> = products
            .iter()
            .flat_map(|product| {
                let label = product["label"].as_str().unwrap_or_default().to_owned();
                PERIODS
                    .iter()
                    .map(move |period| (label.clone(), (*period).to_owned()))
            })
            .collect();
        let actual: BTreeSet<_> = records.keys().cloned().collect();
        if actual != expected {
            let missing: Vec<_> = expected.difference(&actual).take(5).cloned().collect();
            let extra: Vec<_> = actual.difference(&expected).take(5).cloned().collect();
            return Err(fail(format!(
                "{year} workbook list incomplete: missing={missing:?} extra={extra:?}"
            )));
        }
        records_by_year.insert(year, records);
    }

    let mut tasks = Vec::new();
    for (year, records) in records_by_year {
        for product in products {
            let key = product["key"].as_str().ok_or("product key missing")?;
            let label = product["label"].as_str().ok_or("product label missing")?;
            for period in PERIODS {
                let source = records
                    .get(&(label.to_owned(), period.to_owned()))
                    .ok_or_else(|| fail(format!("{label} {period}: workbook listing missing")))?
                    .clone();
                tasks.push(DownloadTask {
                    year: year.to_owned(),
                    key: key.to_owned(),
                    period: period.to_owned(),
                    source,
                    path: runs.join(year).join(key).join(format!("{period}.xlsx")),
                });
            }
        }
    }

    let results = Mutex::new(Vec::<std::result::Result<Value, String>>::with_capacity(
        tasks.len(),
    ));
    let worker_count = 8.min(tasks.len().max(1));
    thread::scope(|scope| {
        for worker in 0..worker_count {
            let tasks = &tasks;
            let results = &results;
            let client = &client;
            let root = root.to_path_buf();
            scope.spawn(move || {
                for task in tasks.iter().skip(worker).step_by(worker_count) {
                    let result = download_workbook(client, task, &root).map_err(|e| e.to_string());
                    results
                        .lock()
                        .expect("workbook result lock poisoned")
                        .push(result);
                }
            });
        }
    });
    let results = results
        .into_inner()
        .map_err(|_| fail("workbook result lock poisoned"))?;
    let mut workbook_records = Vec::with_capacity(results.len());
    let mut errors = Vec::new();
    for result in results {
        match result {
            Ok(record) => workbook_records.push(record),
            Err(error) => errors.push(error),
        }
    }
    if !errors.is_empty() {
        return Err(fail(format!(
            "{} workbook download(s) failed:\n{}",
            errors.len(),
            errors.join("\n")
        )));
    }
    workbook_records.sort_by_key(|record| {
        (
            record["product_key"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            record["year"].as_str().unwrap_or_default().to_owned(),
            record["period"].as_str().unwrap_or_default().to_owned(),
        )
    });
    let manifest = json!({
        "schema_version": 1,
        "status": "WORKBOOKS_CACHED_MAPPING_PENDING",
        "product_count": products.len(),
        "workbook_count": workbook_records.len(),
        "source_listings": listing_hashes,
        "workbooks": workbook_records,
    });
    let output = root.join("product_workbook_manifest_2021_2023.json");
    write_json(&output, &manifest)?;
    println!(
        "saved workbook references and hashes to {}",
        output.display()
    );
    Ok(())
}

fn normalized(value: &str) -> String {
    value.nfkc().collect()
}

fn source_value<'a>(
    source_rows: &'a BTreeMap<(String, String, String), String>,
    origin: usize,
    measure: usize,
    cat03: usize,
) -> Result<&'a str> {
    source_rows
        .get(&(
            (1000 + origin).to_string(),
            (1001 + measure).to_string(),
            cat03.to_string(),
        ))
        .map(String::as_str)
        .ok_or_else(|| {
            fail(format!(
                "source observation coordinate missing: {origin}/{measure}/{cat03}"
            ))
        })
}

fn verify_workbook(
    product_key: &str,
    product_label: &str,
    year: &str,
    period: &str,
    workbook: &Value,
    source_rows: &BTreeMap<(String, String, String), String>,
    root: &Path,
) -> Result<(BTreeMap<usize, usize>, Vec<usize>)> {
    let workbook_path = root.join(
        workbook
            .get("path")
            .and_then(Value::as_str)
            .ok_or("workbook manifest path missing")?,
    );
    let cells = spreadsheet_cells(&workbook_path)?;
    let title = normalized(cells.get("A4").map(String::as_str).unwrap_or_default());
    let label = normalized(product_label);
    let expected_period = if period == "total" {
        "計".to_owned()
    } else {
        format!("{}月", period.parse::<usize>()?)
    };
    if !title.contains(&label) || !title.ends_with(&expected_period) {
        return Err(fail(format!(
            "{product_key} {year} {period}: workbook title does not identify product and period"
        )));
    }

    let origin_re = Regex::new(r"^B([0-9]+)$")?;
    let mut origin_rows = BTreeMap::new();
    for (address, value) in &cells {
        let Some(capture) = origin_re.captures(address) else {
            continue;
        };
        let Ok(origin) = value.parse::<usize>() else {
            continue;
        };
        if (1..=48).contains(&origin) {
            origin_rows.insert(origin, capture[1].parse::<usize>()?);
        }
    }
    if origin_rows.keys().copied().collect::<BTreeSet<_>>() != (1..=48).collect() {
        return Err(fail(format!(
            "{product_key} {year} {period}: origin rows are incomplete"
        )));
    }

    for region in 0..12 {
        let quantity_address = format!("{}11", column_name(3 + region * 2));
        let price_address = format!("{}11", column_name(4 + region * 2));
        let quantity_unit = normalized(
            cells
                .get(&quantity_address)
                .map(String::as_str)
                .unwrap_or_default(),
        );
        let price_unit = normalized(
            cells
                .get(&price_address)
                .map(String::as_str)
                .unwrap_or_default(),
        );
        if !quantity_unit.starts_with('t') || !price_unit.starts_with("円/kg") {
            return Err(fail(format!(
                "{product_key} {year} {period}: workbook units differ at region column {}",
                region + 1
            )));
        }
    }

    let mut scores = BTreeMap::new();
    for block in 1..=13 {
        let mut matches = 0;
        for (origin, row) in &origin_rows {
            for region in 0..12 {
                for measure in 0..2 {
                    let address = format!("{}{}", column_name(3 + region * 2 + measure), row);
                    let workbook_value = cells.get(&address).ok_or_else(|| {
                        fail(format!(
                            "{product_key} {year} {period}: workbook cell {address} missing"
                        ))
                    })?;
                    let api_value =
                        source_value(source_rows, *origin, measure, 989 + block * 12 + region)?;
                    matches += usize::from(workbook_value == api_value);
                }
            }
        }
        scores.insert(block, matches);
    }
    let candidates: Vec<_> = scores
        .iter()
        .filter_map(|(block, matches)| (*matches == 1152).then_some(*block))
        .collect();
    if candidates.is_empty() {
        return Err(fail(format!(
            "{product_key} {year} {period}: no exact block match; {scores:?}"
        )));
    }
    Ok((scores, candidates))
}

fn unique_perfect_assignment(
    candidate_blocks: &BTreeMap<String, Vec<usize>>,
    key: &str,
    year: &str,
) -> Result<(BTreeMap<String, usize>, BTreeMap<String, Vec<usize>>)> {
    fn search(
        periods: &[String],
        candidates: &BTreeMap<String, Vec<usize>>,
        index: usize,
        assigned: &mut BTreeMap<String, usize>,
        used: &mut BTreeSet<usize>,
        solutions: &mut Vec<BTreeMap<String, usize>>,
    ) {
        if solutions.len() > 1 {
            return;
        }
        if index == periods.len() {
            solutions.push(assigned.clone());
            return;
        }
        let period = &periods[index];
        for block in candidates.get(period).into_iter().flatten() {
            if used.insert(*block) {
                assigned.insert(period.clone(), *block);
                search(periods, candidates, index + 1, assigned, used, solutions);
                assigned.remove(period);
                used.remove(block);
            }
        }
    }

    let mut periods: Vec<_> = candidate_blocks.keys().cloned().collect();
    periods.sort_by_key(|period| (candidate_blocks[period].len(), period.clone()));
    let mut solutions = Vec::new();
    search(
        &periods,
        candidate_blocks,
        0,
        &mut BTreeMap::new(),
        &mut BTreeSet::new(),
        &mut solutions,
    );
    let assignment = solutions.into_iter().next().ok_or_else(|| {
        fail(format!(
            "{key} {year}: workbook-to-block evidence has no complete one-to-one mapping"
        ))
    })?;
    let ambiguities = candidate_blocks
        .iter()
        .filter(|(_, blocks)| blocks.len() > 1)
        .map(|(period, blocks)| (period.clone(), blocks.clone()))
        .collect();
    Ok((assignment, ambiguities))
}

fn mapping_record(
    year: &str,
    period: &str,
    workbook: &Value,
    scores: &BTreeMap<usize, usize>,
    assigned_block: usize,
) -> Result<Value> {
    let exact_candidates: Vec<_> = scores
        .iter()
        .filter_map(|(block, matches)| (*matches == 1152).then_some(*block))
        .collect();
    let first_code = 989 + assigned_block * 12;
    let mapped_period = if period == "total" {
        "total".to_owned()
    } else {
        format!("{year}-{period}")
    };
    let runner_up = scores
        .iter()
        .filter(|(block, _)| **block != assigned_block)
        .map(|(_, matches)| *matches)
        .max()
        .ok_or("mapping scores are empty")?;
    Ok(json!({
        "period": mapped_period,
        "block": assigned_block,
        "cat03_first_code": first_code.to_string(),
        "cat03_last_code": (first_code + 11).to_string(),
        "stat_inf_id": workbook["stat_inf_id"],
        "spreadsheet_url": workbook["url"],
        "spreadsheet_sha256": workbook["sha256"],
        "matching_cells": scores[&assigned_block],
        "compared_cells": 1152,
        "runner_up_matches": runner_up,
        "exact_match_candidates": exact_candidates,
    }))
}

/// Re-run official workbook matching and write verified product/year temporal mappings.
pub fn verify_product_temporal_mappings() -> Result<()> {
    let root = Path::new(ROOT);
    let inventory_path = root.join("product_inventory_2021_2023.json");
    let manifest_path = root.join("product_workbook_manifest_2021_2023.json");
    let mut inventory = read_json(&inventory_path)?;
    let manifest = read_json(&manifest_path)?;
    let workbook_records = manifest
        .get("workbooks")
        .and_then(Value::as_array)
        .ok_or("workbook manifest has no workbook records")?;
    let mut workbook_index: HashMap<(String, String, String), Value> = HashMap::new();
    for workbook in workbook_records {
        let key = (
            workbook["product_key"]
                .as_str()
                .ok_or("product key missing from workbook")?
                .to_owned(),
            workbook["year"]
                .as_str()
                .ok_or("year missing from workbook")?
                .to_owned(),
            workbook["period"]
                .as_str()
                .ok_or("period missing from workbook")?
                .to_owned(),
        );
        let mut record = workbook.clone();
        let relative_path = record["path"].as_str().ok_or("workbook path missing")?;
        record["local_path"] = json!(root.join(relative_path));
        if workbook_index.insert(key, record).is_some() {
            return Err(fail("duplicate workbook manifest key"));
        }
    }
    if workbook_index.len() != 1950 {
        return Err(fail(format!(
            "expected 1,950 product workbooks, found {}",
            workbook_index.len()
        )));
    }

    let mapping_dir = root.join("product_temporal_mappings");
    fs::create_dir_all(&mapping_dir)?;
    let products = inventory
        .get_mut("products")
        .and_then(Value::as_array_mut)
        .ok_or("product inventory has no products")?;
    let mut summaries = Vec::new();

    for product in products {
        let key = product["key"]
            .as_str()
            .ok_or("product key missing")?
            .to_owned();
        let label = product["label"]
            .as_str()
            .ok_or("product label missing")?
            .to_owned();
        let mut years = serde_json::Map::new();
        let mut equivalent_period_count = 0;
        let mut period_match_count = 0;
        let mut minimum_runner_up = usize::MAX;

        for year in YEARS {
            let table_id = product["table_ids"][year]
                .as_str()
                .ok_or_else(|| fail(format!("{key} {year}: table ID missing")))?
                .to_owned();
            let observation_run_id = product["observation_runs"][year]["run_id"]
                .as_str()
                .ok_or_else(|| fail(format!("{key} {year}: source evidence is not linked")))?;
            let metadata_hash = product["metadata_runs"][year]["metadata_sha256"]
                .as_str()
                .ok_or_else(|| fail(format!("{key} {year}: metadata reference is missing")))?
                .to_owned();
            let observation_path = Path::new(RUNS)
                .join(observation_run_id)
                .join("source_observations.json");
            let report = read_json(&observation_path)?;
            if report["table_id"].as_str() != Some(&table_id)
                || report["count"].as_u64() != Some(14976)
            {
                return Err(fail(format!(
                    "{key} {year}: source table or observation count mismatch"
                )));
            }
            if report["source_metadata_sha256"].as_str() != Some(&metadata_hash) {
                return Err(fail(format!(
                    "{key} {year}: observation/metadata hashes differ"
                )));
            }
            let mut source_rows = BTreeMap::new();
            for row in report["rows"]
                .as_array()
                .ok_or("source observation rows missing")?
            {
                let attributes = &row["attributes"];
                let coordinate = (
                    attributes["@cat01"]
                        .as_str()
                        .ok_or("@cat01 missing")?
                        .to_owned(),
                    attributes["@cat02"]
                        .as_str()
                        .ok_or("@cat02 missing")?
                        .to_owned(),
                    attributes["@cat03"]
                        .as_str()
                        .ok_or("@cat03 missing")?
                        .to_owned(),
                );
                let raw_value = row["raw_value"]
                    .as_str()
                    .ok_or("source raw_value missing")?
                    .to_owned();
                if source_rows.insert(coordinate, raw_value).is_some() {
                    return Err(fail(format!("{key} {year}: duplicate source coordinate")));
                }
            }
            if source_rows.len() != 14976 {
                return Err(fail(format!(
                    "{key} {year}: source coordinate grid is incomplete"
                )));
            }

            let mut candidate_blocks = BTreeMap::new();
            let mut evidence = BTreeMap::new();
            for period in PERIODS {
                let workbook = workbook_index
                    .get(&(key.clone(), year.to_owned(), period.to_owned()))
                    .ok_or_else(|| {
                        fail(format!(
                            "{key} {year} {period}: workbook missing from manifest"
                        ))
                    })?;
                let path = Path::new(
                    workbook["local_path"]
                        .as_str()
                        .ok_or("local workbook path missing")?,
                );
                let content = fs::read(path)?;
                if sha256(&content) != workbook["sha256"].as_str().ok_or("workbook hash missing")? {
                    return Err(fail(format!(
                        "{key} {year} {period}: workbook checksum mismatch"
                    )));
                }
                let (scores, candidates) =
                    verify_workbook(&key, &label, year, period, workbook, &source_rows, root)?;
                evidence.insert(period.to_owned(), (workbook.clone(), scores));
                candidate_blocks.insert(period.to_owned(), candidates);
            }
            let (assignment, ambiguities) =
                unique_perfect_assignment(&candidate_blocks, &key, year)?;
            let mut mapped_periods = Vec::new();
            let mut ambiguity_json = serde_json::Map::new();
            for (period, blocks) in &ambiguities {
                ambiguity_json.insert(period.clone(), json!(blocks));
            }
            for period in PERIODS {
                let block = assignment[period];
                let (workbook, scores) = &evidence[period];
                let mut record = mapping_record(year, period, workbook, scores, block)?;
                record["block_identity"] = json!(if ambiguities.contains_key(period) {
                    "equivalent_candidates"
                } else {
                    "unique"
                });
                minimum_runner_up = minimum_runner_up
                    .min(record["runner_up_matches"].as_u64().unwrap_or_default() as usize);
                mapped_periods.push(record);
            }
            equivalent_period_count += ambiguities.len();
            period_match_count += mapped_periods.len();
            years.insert(
                year.to_owned(),
                json!({
                    "table_id": table_id,
                    "raw_capture": format!("runs/{observation_run_id}"),
                    "metadata_sha256": metadata_hash,
                    "official_listing": official_listing(year)?,
                    "periods": mapped_periods,
                    "mapping_ambiguities": ambiguity_json,
                }),
            );
        }

        let status = if equivalent_period_count > 0 {
            "VALUE_MATCH_VERIFIED_WITH_EQUIVALENT_BLOCKS"
        } else {
            "VALUE_MATCH_VERIFIED"
        };
        let mapping = json!({
            "status": status,
            "method": "For each official title-labelled workbook, compare all 48 origin × 12 region × 2 measure lexical cells against every one of the 13 source cat03 blocks. Require a complete one-to-one assignment from periods to blocks using 1,152/1,152 exact matches. If more than one assignment is possible, record every equivalent exact-match candidate; the displayed values are invariant across those candidates and the UI labels the equivalence.",
            "commodity": key,
            "commodity_label": label,
            "years": years,
        });
        let output = mapping_dir.join(format!("{key}.json"));
        write_json(&output, &mapping)?;
        product["mapping_status"] =
            json!(if status == "VALUE_MATCH_VERIFIED_WITH_EQUIVALENT_BLOCKS" {
                "verified_with_equivalent_blocks"
            } else {
                "verified"
            });
        product["mapping_evidence"] = json!(format!("product_temporal_mappings/{key}.json"));
        summaries.push(json!({
            "key": key,
            "label": label,
            "period_matches": period_match_count,
            "equivalent_periods": equivalent_period_count,
            "minimum_runner_up_matches": minimum_runner_up,
        }));
        println!("verified {key} {label}: 39/39 workbooks");
    }

    write_json(&inventory_path, &inventory)?;
    let workbook_matches = summaries
        .iter()
        .map(|item| item["period_matches"].as_u64().unwrap_or_default())
        .sum::<u64>();
    let report = json!({
        "status": "VALUE_MATCH_VERIFIED",
        "product_count": summaries.len(),
        "workbook_matches": workbook_matches,
        "products": summaries,
    });
    write_json(
        &root.join("product_temporal_mapping_report_2021_2023.json"),
        &report,
    )?;
    Ok(())
}

fn official_listing(year: &str) -> Result<String> {
    let class_id = match year {
        "2021" => "000001169106",
        "2022" => "000001209867",
        "2023" => "000001220580",
        _ => return Err(fail(format!("unsupported year {year}"))),
    };
    Ok(format!(
        "https://www.e-stat.go.jp/stat-search/files?cycle=7&layout=datalist&page=1&tclass1=000001020455&tclass2={class_id}&tclass3val=0&toukei=00500226&tstat=000001015623"
    ))
}

fn observation_rows(report: &Value) -> Result<BTreeMap<(String, String, String), String>> {
    let mut rows = BTreeMap::new();
    for row in report["rows"]
        .as_array()
        .ok_or("source observation rows missing")?
    {
        let attributes = &row["attributes"];
        let key = (
            attributes["@cat01"]
                .as_str()
                .ok_or("@cat01 missing")?
                .to_owned(),
            attributes["@cat02"]
                .as_str()
                .ok_or("@cat02 missing")?
                .to_owned(),
            attributes["@cat03"]
                .as_str()
                .ok_or("@cat03 missing")?
                .to_owned(),
        );
        let value = row["raw_value"]
            .as_str()
            .ok_or("raw_value missing")?
            .to_owned();
        if rows.insert(key, value).is_some() {
            return Err(fail("duplicate source observation coordinate"));
        }
    }
    Ok(rows)
}

fn compare_daikon_workbook(
    year: &str,
    record: &Value,
    rows: &BTreeMap<(String, String, String), String>,
) -> Result<((usize, usize), (usize, usize))> {
    let period = record["period"].as_str().ok_or("mapping period missing")?;
    let period_number = if period == "total" {
        0
    } else {
        period
            .get(period.len().saturating_sub(2)..)
            .ok_or("invalid mapping period")?
            .parse::<usize>()?
    };
    let path = Path::new(ROOT)
        .join("runs/official-daikon-2021-2023")
        .join(year)
        .join(format!("{period_number:02}.xlsx"));
    let bytes = fs::read(&path)?;
    if sha256(&bytes)
        != record["spreadsheet_sha256"]
            .as_str()
            .ok_or("mapping workbook hash missing")?
    {
        return Err(fail(format!("{year} {period}: workbook checksum mismatch")));
    }
    let cells = spreadsheet_cells(&path)?;
    let title = cells.get("A4").map(String::as_str).unwrap_or_default();
    if !title.contains("だいこん") {
        return Err(fail(format!(
            "{year} {period}: workbook title does not identify daikon"
        )));
    }
    let title = normalized(title);
    let expected_period = if period_number == 0 {
        "計".to_owned()
    } else {
        format!("{period_number}月")
    };
    if !title.ends_with(&expected_period) {
        return Err(fail(format!(
            "{year} {period}: workbook period title mismatch"
        )));
    }
    let origin_re = Regex::new(r"^B([0-9]+)$")?;
    let mut origin_rows = BTreeMap::new();
    for (address, value) in &cells {
        let Some(capture) = origin_re.captures(address) else {
            continue;
        };
        let Ok(origin) = value.parse::<usize>() else {
            continue;
        };
        if (1..=48).contains(&origin) {
            origin_rows.insert(origin, capture[1].parse::<usize>()?);
        }
    }
    if origin_rows.keys().copied().collect::<BTreeSet<_>>() != (1..=48).collect() {
        return Err(fail(format!(
            "{year} {period}: workbook origin rows incomplete"
        )));
    }

    let mut scores = Vec::new();
    for block in 1..=13 {
        let mut matches = 0;
        for (origin, row) in &origin_rows {
            for region in 0..12 {
                for measure in 0..2 {
                    let address = format!("{}{}", column_name(3 + region * 2 + measure), row);
                    let workbook_value = cells.get(&address).ok_or_else(|| {
                        fail(format!("{year} {period}: workbook cell {address} missing"))
                    })?;
                    let api_value = rows
                        .get(&(
                            (1000 + origin).to_string(),
                            (1001 + measure).to_string(),
                            (989 + block * 12 + region).to_string(),
                        ))
                        .ok_or("matching API value missing")?;
                    matches += usize::from(workbook_value == api_value);
                }
            }
        }
        scores.push((matches, block));
    }
    scores.sort_by(|left, right| right.cmp(left));
    let best = scores[0];
    let second = scores[1];
    if best
        != (
            1152,
            record["block"].as_u64().ok_or("mapping block missing")? as usize,
        )
        || second.0
            != record["runner_up_matches"]
                .as_u64()
                .ok_or("runner-up score missing")? as usize
    {
        return Err(fail(format!(
            "{year} {period}: workbook match differs from saved mapping"
        )));
    }
    Ok((best, second))
}

/// Independently replay the original daikon workbook-to-block evidence.
pub fn verify_daikon_temporal_mapping() -> Result<()> {
    let mapping = read_json(&Path::new(ROOT).join("2021_2023_temporal_mapping.json"))?;
    for year in YEARS {
        let year_data = &mapping["years"][year];
        let report = read_json(
            &Path::new(ROOT)
                .join(
                    year_data["raw_capture"]
                        .as_str()
                        .ok_or("raw capture path missing")?,
                )
                .join("source_observations.json"),
        )?;
        let rows = observation_rows(&report)?;
        if rows.len() != 14976 {
            return Err(fail(format!(
                "{year}: source coordinate grid is incomplete"
            )));
        }
        for record in year_data["periods"]
            .as_array()
            .ok_or("mapping periods missing")?
        {
            let (best, second) = compare_daikon_workbook(year, record, &rows)?;
            println!(
                "{year} {} {:?} {:?}",
                record["period"].as_str().unwrap_or_default(),
                best,
                second
            );
        }
    }
    Ok(())
}

fn iso_timestamp() -> String {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let seconds = duration.as_secs() as i64;
    let days = seconds.div_euclid(86_400);
    let day_seconds = seconds.rem_euclid(86_400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        day_seconds / 3600,
        (day_seconds % 3600) / 60,
        day_seconds % 60
    )
}

fn unique_stamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

/// Run the historical M3/M5 offline validation and write its JSON report.
pub fn validate_m3_m5(output_path: Option<&Path>) -> Result<bool> {
    let root = Path::new(ROOT);
    let runs = root.join("runs");
    let mapping_path = root.join("2021_2023_temporal_mapping.json");
    let mapping = read_json(&mapping_path)?;
    let mut checks = Vec::new();
    let mut captures = BTreeMap::<String, (PathBuf, PathBuf)>::new();

    for year in YEARS {
        let entry = &mapping["years"][year];
        let relative_capture = entry["raw_capture"]
            .as_str()
            .ok_or("raw capture path missing")?;
        let capture = root.join(relative_capture).join("source_observations.json");
        let expected_metadata = entry["metadata_sha256"]
            .as_str()
            .ok_or("metadata hash missing")?;
        let mut metadata_matches = Vec::new();
        for item in fs::read_dir(&runs)? {
            let item = item?;
            if !item.file_name().to_string_lossy().starts_with("metadata-") {
                continue;
            }
            let capture_path = item.path().join("capture.json");
            if !capture_path.is_file() {
                continue;
            }
            if read_json(&capture_path)?["sha256"].as_str() == Some(expected_metadata) {
                metadata_matches.push(item.path());
            }
        }
        if !capture.is_file() || metadata_matches.len() != 1 {
            checks.push(json!({
                "name": format!("M4 replay {year}"),
                "status": "blocked",
                "details": "raw capture or uniquely bound metadata capture missing",
            }));
            continue;
        }
        let observation_dir = capture
            .parent()
            .ok_or("capture has no parent")?
            .to_path_buf();
        let metadata_dir = metadata_matches.remove(0);
        match acquisition::replay(&observation_dir, &metadata_dir) {
            Ok(_) => {
                checks.push(json!({"name": format!("M4 replay {year}"), "status": "passed", "details": "replay verified"}));
                captures.insert(year.to_owned(), (observation_dir, metadata_dir));
            }
            Err(error) => checks.push(json!({
                "name": format!("M4 replay {year}"),
                "status": "failed",
                "details": error.to_string(),
            })),
        }
    }

    let temporal_result = verify_daikon_temporal_mapping();
    let temporal_passed = temporal_result.is_ok();
    checks.push(json!({
        "name": "2021-2023 official workbook temporal and cell matching",
        "status": if temporal_passed { "passed" } else { "failed" },
        "details": temporal_result.err().map(|error| error.to_string()).unwrap_or_else(|| "all official workbook comparisons passed".to_owned()),
    }));

    let output_root = output_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| runs.join("m3-m5-validation-report.json"));
    for year in YEARS {
        let Some((observation_dir, metadata_dir)) = captures.get(year) else {
            checks.push(json!({
                "name": format!("M5-B historical normalization {year}"),
                "status": "blocked",
                "details": "missing replay inputs or workbook verification did not pass",
            }));
            continue;
        };
        if !temporal_passed {
            checks.push(json!({
                "name": format!("M5-B historical normalization {year}"),
                "status": "blocked",
                "details": "missing replay inputs or workbook verification did not pass",
            }));
            continue;
        }
        let out_dir = runs.join(format!("m5b-validation-{year}-{}", unique_stamp()));
        fs::create_dir_all(&out_dir)?;
        let normalized_path = match normalization::run_2021_2023(
            observation_dir,
            metadata_dir,
            &mapping_path,
            &out_dir,
        ) {
            Ok(path) => path,
            Err(error) => {
                checks.push(json!({
                    "name": format!("M5-B historical normalization {year}"),
                    "status": "failed",
                    "details": error.to_string(),
                }));
                continue;
            }
        };
        let evidence = read_json(&normalized_path)?;
        let source_path = observation_dir.join("source_observations.json");
        let source = read_json(&source_path)?;
        let expected_rows: HashMap<usize, &Value> = source["rows"]
            .as_array()
            .ok_or("source rows missing")?
            .iter()
            .filter_map(|row| {
                row["position"]
                    .as_u64()
                    .map(|position| (position as usize, row))
            })
            .collect();
        let mut period_by_cat03: HashMap<usize, Option<String>> = HashMap::new();
        for period_record in mapping["years"][year]["periods"]
            .as_array()
            .ok_or("mapping periods missing")?
        {
            let period = period_record["period"]
                .as_str()
                .ok_or("period name missing")?
                .to_owned();
            let first = period_record["cat03_first_code"]
                .as_str()
                .ok_or("first cat03 missing")?
                .parse::<usize>()?;
            let last = period_record["cat03_last_code"]
                .as_str()
                .ok_or("last cat03 missing")?
                .parse::<usize>()?;
            for cat03 in first..=last {
                if period_by_cat03
                    .insert(cat03, Some(period.clone()))
                    .is_some()
                {
                    period_by_cat03.insert(cat03, None);
                }
            }
        }
        let rows = evidence["rows"]
            .as_array()
            .ok_or("normalized rows missing")?;
        let mapping_sha = sha256(&fs::read(&mapping_path)?);
        let mut valid = evidence["table_id"].as_str()
            == mapping["years"][year]["table_id"].as_str()
            && evidence["metadata_sha256"].as_str()
                == mapping["years"][year]["metadata_sha256"].as_str()
            && evidence["temporal_mapping_sha256"].as_str() == Some(mapping_sha.as_str())
            && evidence["input_count"].as_u64() == Some(14976)
            && evidence["monthly_count"].as_u64() == Some(13824)
            && evidence["annual_summary_count"].as_u64() == Some(1152)
            && rows.len() == 14976
            && expected_rows.len() == 14976;
        let mut seen = BTreeSet::new();
        let mut period_assignments_verified = 0;
        if valid {
            for row in rows {
                let Some(position) = row["source_position"].as_u64().map(|value| value as usize)
                else {
                    valid = false;
                    break;
                };
                let Some(original) = expected_rows.get(&position) else {
                    valid = false;
                    break;
                };
                if !seen.insert(position) {
                    valid = false;
                    break;
                }
                let raw = original["raw_value"].as_str().unwrap_or_default();
                let Some(source_cat03) = original["attributes"]["@cat03"]
                    .as_str()
                    .and_then(|code| code.parse::<usize>().ok())
                else {
                    valid = false;
                    break;
                };
                if !(1001..=1156).contains(&source_cat03) {
                    valid = false;
                    break;
                }
                let expected_period = period_by_cat03
                    .get(&source_cat03)
                    .and_then(Option::as_deref);
                let expected_region = 1001 + (source_cat03 - 1001) % 12;
                let source_cat03_text = source_cat03.to_string();
                let expected_region_text = expected_region.to_string();
                let expected_kind = match raw {
                    "-" => "no_fact",
                    "…" => "unknown_or_uncollected",
                    "0" => "below_display_unit",
                    _ => "reported",
                };
                let period = row["period"].as_str().unwrap_or_default();
                valid = row["raw_value"].as_str() == Some(raw)
                    && row["source_attributes"] == original["attributes"]
                    && row["source_cat03_code"].as_str() == Some(source_cat03_text.as_str())
                    && row["consumer_region_category_code"].as_str()
                        == Some(expected_region_text.as_str())
                    && expected_period == Some(period)
                    && row["annual_total"].as_bool() == Some(period == "total")
                    && row["source_unit"] == original["unit"]
                    && row["value_kind"].as_str() == Some(expected_kind);
                if !valid {
                    break;
                }
                period_assignments_verified += 1;
            }
        }
        checks.push(json!({
            "name": format!("M5-B historical normalization {year}"),
            "status": if valid { "passed" } else { "failed" },
            "details": {
                "rows": rows.len(),
                "period_assignments_verified": if valid { period_assignments_verified } else { 0 },
                "symbols_and_provenance_preserved": valid,
            }
        }));
    }

    let blockers: Vec<_> = checks
        .iter()
        .filter(|check| check["status"].as_str() != Some("passed"))
        .map(|check| check["name"].clone())
        .collect();
    let report = json!({
        "format_version": 1,
        "generated_at": iso_timestamp(),
        "status": if blockers.is_empty() { "PASSED" } else { "BLOCKED" },
        "scope": "2021–2023 source replay, workbook mapping, and normalization",
        "checks": checks,
        "blockers": blockers,
    });
    if let Some(parent) = output_root.parent() {
        fs::create_dir_all(parent)?;
    }
    write_json(&output_root, &report)?;
    println!(
        "{}",
        serde_json::to_string(&json!({
            "status": report["status"],
            "report": output_root,
            "blockers": report["blockers"],
        }))?
    );
    Ok(report["status"].as_str() == Some("PASSED"))
}
