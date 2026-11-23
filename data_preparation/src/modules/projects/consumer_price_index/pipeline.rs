use std::{
    collections::{BTreeMap, HashMap, HashSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use serde::Serialize;
use serde_json::Value;

use crate::modules::estat_api::{
    EstatClient,
    get_meta_info::{GetMetaInfoParamsBuilder, MandatoryParams as MetaParams},
    get_stats_data::{GetStatsDataParamsBuilder, MandatoryParams as DataParams},
};

use super::{
    ITEMS_PER_CHUNK, OUTPUT_DIR_META, OUTPUT_DIR_RAW, OUTPUT_DIR_TRANSFORMED, PAGE_SIZE,
    STATS_DATA_ID, path,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemLookup {
    code: String,
    name: String,
    level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_code: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RegionLookup {
    code: String,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MonthLookup {
    code: String,
    year_month: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SeriesDocument {
    item_code: String,
    area_code: String,
    time_value_pairs: Vec<[Value; 2]>,
}

fn parse_u64(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
}

fn as_items(value: &Value) -> Vec<&Value> {
    match value {
        Value::Array(items) => items.iter().collect(),
        Value::Object(_) => vec![value],
        _ => Vec::new(),
    }
}

fn attr<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn required_attr<'a>(value: &'a Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    attr(value, key).ok_or_else(|| format!("e-Stat classification is missing {key}").into())
}

fn redact_app_id(value: &mut Value) {
    match value {
        Value::Object(object) => {
            object.remove("appId");
            object.remove("@appId");
            for child in object.values_mut() {
                redact_app_id(child);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(redact_app_id),
        _ => {}
    }
}

fn remove_dir_if_exists(path: &Path) -> Result<(), Box<dyn Error>> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

/// Promote completed directories together, restoring the prior generation if a rename fails.
fn promote_directories(
    directories: &[(&Path, &Path)],
    backup_root: &Path,
) -> Result<(), Box<dyn Error>> {
    remove_dir_if_exists(backup_root)?;
    fs::create_dir_all(backup_root)?;
    let mut promoted: Vec<(PathBuf, PathBuf, bool)> = Vec::new();

    for (staged, active) in directories {
        let name = active
            .file_name()
            .ok_or("Active output directory had no final path component")?;
        let backup = backup_root.join(name);
        let had_previous = active.exists();
        if had_previous {
            if let Err(error) = fs::rename(active, &backup) {
                rollback_directories(&promoted)?;
                return Err(error.into());
            }
        }
        if let Err(error) = fs::rename(staged, active) {
            if had_previous {
                fs::rename(&backup, active)?;
            }
            rollback_directories(&promoted)?;
            return Err(error.into());
        }
        promoted.push((active.to_path_buf(), backup, had_previous));
    }

    remove_dir_if_exists(backup_root)?;
    Ok(())
}

fn rollback_directories(promoted: &[(PathBuf, PathBuf, bool)]) -> Result<(), Box<dyn Error>> {
    for (active, backup, had_previous) in promoted.iter().rev() {
        remove_dir_if_exists(active)?;
        if *had_previous {
            fs::rename(backup, active)?;
        }
    }
    Ok(())
}

/// Fetch metadata and all monthly index observations, retaining each raw page.
pub fn acquire(app_id: &str) -> Result<(), Box<dyn Error>> {
    let client = EstatClient::new()?;
    if app_id.is_empty() {
        return Err("e-Stat application ID is required".into());
    }
    let output_root = path(super::OUTPUT_DIR);
    let staging_root = output_root.join(".staging");
    let raw_dir = staging_root.join("raw_data");
    let meta_dir = staging_root.join("meta_data");
    let transformed_dir = staging_root.join("transformed_data");
    remove_dir_if_exists(&staging_root)?;
    fs::create_dir_all(&raw_dir)?;
    fs::create_dir_all(&meta_dir)?;
    let mut metadata: Value = serde_json::from_str(
        &client
            .get_meta_info(
                GetMetaInfoParamsBuilder::new(MetaParams {
                    app_id,
                    stats_data_id: STATS_DATA_ID,
                })
                .lang("J")
                .build(),
            )
            .map_err(|error| format!("metadata request failed: {error}"))?,
    )?;
    redact_app_id(&mut metadata);
    fs::write(
        raw_dir.join("get_meta_info.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;

    let mut start = 1_u32;
    let mut expected_total = None;
    loop {
        if start > 1 {
            thread::sleep(Duration::from_secs(2));
        }
        let response = client
            .get_stats_data(
                GetStatsDataParamsBuilder::new(DataParams { app_id })
                    .stats_data_id(STATS_DATA_ID)
                    .cd_tab("1")
                    .lv_time("4")
                    .start_position(start)
                    .limit(PAGE_SIZE)
                    .meta_get_flg("N")
                    .build(),
            )
            .map_err(|error| format!("data request at position {start} failed: {error}"))?;
        let mut page: Value = serde_json::from_str(&response)?;
        let stats = &page["GET_STATS_DATA"]["STATISTICAL_DATA"];
        let info = &stats["RESULT_INF"];
        let total = parse_u64(&info["TOTAL_NUMBER"])
            .ok_or("e-Stat page omitted RESULT_INF.TOTAL_NUMBER")?;
        if expected_total.is_some_and(|value| value != total) {
            return Err("e-Stat TOTAL_NUMBER changed while paging".into());
        }
        expected_total = Some(total);
        let values = as_items(&stats["DATA_INF"]["VALUE"]);
        if values.is_empty() {
            return Err(format!("e-Stat returned an empty page at position {start}").into());
        }
        let from = parse_u64(&info["FROM_NUMBER"]).ok_or("Missing FROM_NUMBER")?;
        let to = parse_u64(&info["TO_NUMBER"]).ok_or("Missing TO_NUMBER")?;
        let expected_to = u64::from(start) + values.len() as u64 - 1;
        if from != u64::from(start) || to != expected_to || to > total {
            return Err(format!(
                "Invalid e-Stat page range {from}-{to} at requested position {start}"
            )
            .into());
        }
        let next = if to == total {
            None
        } else {
            Some(
                parse_u64(&info["NEXT_KEY"])
                    .ok_or("Missing RESULT_INF.NEXT_KEY on non-final page")?,
            )
        };
        redact_app_id(&mut page);
        fs::write(
            raw_dir.join(format!("get_stats_data_{start:010}.json")),
            serde_json::to_vec(&page)?,
        )?;
        println!("Saved CPI records {from}-{to} of {total}");
        let Some(next) = next else { break };
        if next != to + 1 {
            return Err(format!("Unexpected NEXT_KEY {next}; expected {}", to + 1).into());
        }
        start = u32::try_from(next).map_err(|_| "NEXT_KEY exceeded supported position")?;
    }
    let total = expected_total.ok_or("No CPI data pages were acquired")?;
    fs::write(
        meta_dir.join("acquisition.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "tableId": STATS_DATA_ID,
            "measureCode": "1",
            "monthlyTimeLevel": "4",
            "pageSize": PAGE_SIZE,
            "totalNumber": total,
        }))?,
    )?;
    transform_into(&raw_dir, &transformed_dir)?;
    promote_directories(
        &[
            (raw_dir.as_path(), Path::new(OUTPUT_DIR_RAW)),
            (meta_dir.as_path(), Path::new(OUTPUT_DIR_META)),
            (transformed_dir.as_path(), Path::new(OUTPUT_DIR_TRANSFORMED)),
        ],
        &output_root.join(".previous"),
    )?;
    remove_dir_if_exists(&staging_root)?;
    Ok(())
}

/// Rebuild lookup and series files from saved metadata and raw page responses.
pub fn transform() -> Result<(), Box<dyn Error>> {
    let output_root = path(super::OUTPUT_DIR);
    let staging_root = output_root.join(".staging");
    remove_dir_if_exists(&staging_root)?;
    fs::create_dir_all(&staging_root)?;
    let transformed_dir = staging_root.join("transformed_data");
    transform_into(Path::new(OUTPUT_DIR_RAW), &transformed_dir)?;
    promote_directories(
        &[(transformed_dir.as_path(), Path::new(OUTPUT_DIR_TRANSFORMED))],
        &output_root.join(".previous"),
    )?;
    remove_dir_if_exists(&staging_root)?;
    Ok(())
}

fn transform_into(raw_dir: &Path, transformed_dir: &Path) -> Result<(), Box<dyn Error>> {
    let metadata: Value = serde_json::from_slice(&fs::read(raw_dir.join("get_meta_info.json"))?)?;
    let meta = &metadata["GET_META_INFO"]["METADATA_INF"];
    let class_objects = meta["CLASS_INF"]["CLASS_OBJ"]
        .as_array()
        .ok_or("Metadata CLASS_INF.CLASS_OBJ was not an array")?;
    let items = lookup_dimension(class_objects, "cat01")?;
    let regions = lookup_regions(class_objects)?;
    let time_entries = lookup_dimension(class_objects, "time")?;
    let mut item_chunks = HashMap::new();
    for (index, item) in items.iter().enumerate() {
        item_chunks.insert(item.code.clone(), index / ITEMS_PER_CHUNK);
    }
    let mut months = Vec::new();
    let mut month_by_code = HashMap::new();
    for entry in time_entries {
        if entry.level.as_deref() != Some("4") {
            continue;
        }
        let year_month = parse_year_month(&entry.name)
            .ok_or_else(|| format!("Could not parse monthly label {}", entry.name))?;
        month_by_code.insert(entry.code.clone(), year_month.clone());
        months.push(MonthLookup {
            code: entry.code,
            year_month,
        });
    }
    if months.is_empty() {
        return Err("Metadata contained no monthly time entries".into());
    }
    months.sort_by(|left, right| left.year_month.cmp(&right.year_month));

    let mut raw_paths: Vec<PathBuf> = fs::read_dir(raw_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().and_then(|ext| ext.to_str()) == Some("json")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("get_stats_data_"))
        })
        .collect();
    raw_paths.sort();
    if raw_paths.is_empty() {
        return Err("No saved CPI raw pages are available to transform".into());
    }

    // (region code, item chunk) -> item code -> ascending month/value pairs.
    let mut grouped: BTreeMap<(String, usize), BTreeMap<String, Vec<(String, f64)>>> =
        BTreeMap::new();
    let mut observed_total = 0_u64;
    let mut source_record_total = 0_u64;
    let mut expected_total = None;
    let mut expected_from = 1_u64;
    let mut first_observed_month: Option<String> = None;
    let mut last_observed_month: Option<String> = None;
    for raw_path in raw_paths {
        let page: Value = serde_json::from_slice(&fs::read(&raw_path)?)?;
        let stats = &page["GET_STATS_DATA"]["STATISTICAL_DATA"];
        let info = &stats["RESULT_INF"];
        let total = parse_u64(&info["TOTAL_NUMBER"]).ok_or("Raw page omitted TOTAL_NUMBER")?;
        if expected_total.is_some_and(|value| value != total) {
            return Err("Saved raw pages disagree on TOTAL_NUMBER".into());
        }
        expected_total = Some(total);
        let from = parse_u64(&info["FROM_NUMBER"]).ok_or("Raw page omitted FROM_NUMBER")?;
        let to = parse_u64(&info["TO_NUMBER"]).ok_or("Raw page omitted TO_NUMBER")?;
        let records = as_items(&stats["DATA_INF"]["VALUE"]);
        if from != expected_from || to < from || to > total {
            return Err(format!(
                "Non-contiguous saved CPI page: expected start {expected_from}, got {from}-{to}"
            )
            .into());
        }
        if records.is_empty() || to - from + 1 != records.len() as u64 {
            return Err(format!("Invalid saved page boundaries in {}", raw_path.display()).into());
        }
        expected_from = to + 1;
        let missing_markers: HashSet<&str> = as_items(&stats["DATA_INF"]["NOTE"])
            .into_iter()
            .filter(|note| {
                attr(note, "$").is_some_and(|description| description.contains("該当データがない"))
            })
            .filter_map(|note| attr(note, "@char"))
            .collect();
        for record in records {
            source_record_total += 1;
            if attr(record, "@tab") != Some("1") {
                return Err("Saved CPI page contained a non-index measure".into());
            }
            let item_code = required_attr(record, "@cat01")?;
            let area_code = required_attr(record, "@area")?;
            let time_code = required_attr(record, "@time")?;
            let value_text = required_attr(record, "$")?;
            let month = month_by_code.get(time_code).ok_or_else(|| {
                format!("Raw observation references non-monthly time code {time_code}")
            })?;
            let chunk = *item_chunks
                .get(item_code)
                .ok_or_else(|| format!("Unknown CPI item code {item_code}"))?;
            let numeric_value = match value_text.parse::<f64>() {
                Ok(value) => value,
                Err(_) if missing_markers.contains(value_text) => continue,
                Err(_) => {
                    return Err(format!(
                        "Non-numeric index observation {value_text:?} at time {time_code}"
                    )
                    .into());
                }
            };
            observed_total += 1;
            if first_observed_month
                .as_ref()
                .is_none_or(|current| month < current)
            {
                first_observed_month = Some(month.clone());
            }
            if last_observed_month
                .as_ref()
                .is_none_or(|current| month > current)
            {
                last_observed_month = Some(month.clone());
            }
            grouped
                .entry((area_code.to_owned(), chunk))
                .or_default()
                .entry(item_code.to_owned())
                .or_default()
                .push((month.clone(), numeric_value));
        }
    }
    let expected_total = expected_total.ok_or("No records found in saved CPI pages")?;
    if expected_from != expected_total + 1 {
        return Err(format!(
            "Incomplete saved CPI pagination: covered through {}, expected {expected_total}",
            expected_from - 1
        )
        .into());
    }
    if source_record_total != expected_total {
        return Err(format!(
            "Read {source_record_total} source records, expected {expected_total}"
        )
        .into());
    }

    remove_dir_if_exists(transformed_dir)?;
    fs::create_dir_all(transformed_dir.join("series"))?;
    write_json(&transformed_dir.join("items.json"), &items)?;
    write_json(&transformed_dir.join("regions.json"), &regions)?;
    write_json(&transformed_dir.join("months.json"), &months)?;

    let mut files = 0;
    for ((area_code, chunk), mut series) in grouped {
        let chunk_start = chunk * ITEMS_PER_CHUNK;
        let chunk_end = (chunk_start + ITEMS_PER_CHUNK).min(items.len()) - 1;
        let mut documents = Vec::with_capacity(series.len());
        for (item_code, pairs) in &mut series {
            pairs.sort_by(|left, right| left.0.cmp(&right.0));
            if pairs.windows(2).any(|pair| pair[0].0 == pair[1].0) {
                return Err(format!(
                    "Duplicate CPI observation for item {item_code} in region {area_code}"
                )
                .into());
            }
            let time_value_pairs = pairs
                .iter()
                .map(|(month, value)| [Value::String(month.clone()), serde_json::json!(value)])
                .collect();
            documents.push(SeriesDocument {
                item_code: item_code.clone(),
                area_code: area_code.clone(),
                time_value_pairs,
            });
        }
        let filename = format!("area_{area_code}_items_{chunk_start:04}_{chunk_end:04}.json");
        write_json(&transformed_dir.join("series").join(filename), &documents)?;
        files += 1;
    }
    let observed_at = serde_json::json!({
        "first": first_observed_month,
        "last": last_observed_month,
    });
    write_json(
        &transformed_dir.join("manifest.json"),
        &serde_json::json!({
            "tableId": STATS_DATA_ID,
            "measureCode": "1",
            "monthlyTimeLevel": "4",
            "observationCount": observed_total,
            "sourceRecordCount": source_record_total,
            "missingObservationCount": source_record_total - observed_total,
            "itemCount": items.len(),
            "regionCount": regions.len(),
            "monthCount": months.len(),
            "seriesChunkCount": files,
            "seriesChunkItems": ITEMS_PER_CHUNK,
            "observationRange": observed_at,
            "indexBase": "2025",
            "source": "https://www.e-stat.go.jp/dbview?sid=0004052037"
        }),
    )?;
    verify_representative_samples(&month_by_code, &items, transformed_dir)?;
    println!("Prepared {observed_total} CPI observations in {files} series chunks.");
    Ok(())
}

fn lookup_dimension(class_objects: &[Value], id: &str) -> Result<Vec<ItemLookup>, Box<dyn Error>> {
    let dimension = class_objects
        .iter()
        .find(|item| attr(item, "@id") == Some(id))
        .ok_or_else(|| format!("Missing {id} classification in metadata"))?;
    as_items(&dimension["CLASS"])
        .into_iter()
        .map(|entry| {
            Ok(ItemLookup {
                code: required_attr(entry, "@code")?.to_owned(),
                name: required_attr(entry, "@name")?.to_owned(),
                level: attr(entry, "@level")
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned),
                parent_code: attr(entry, "@parentCode").map(str::to_owned),
            })
        })
        .collect()
}

fn lookup_regions(class_objects: &[Value]) -> Result<Vec<RegionLookup>, Box<dyn Error>> {
    let dimension = class_objects
        .iter()
        .find(|item| attr(item, "@id") == Some("area"))
        .ok_or("Missing area classification in metadata")?;
    as_items(&dimension["CLASS"])
        .into_iter()
        .map(|entry| {
            Ok(RegionLookup {
                code: required_attr(entry, "@code")?.to_owned(),
                name: required_attr(entry, "@name")?.to_owned(),
            })
        })
        .collect()
}

fn parse_year_month(label: &str) -> Option<String> {
    let (year, rest) = label.split_once('年')?;
    let (month, _) = rest.split_once('月')?;
    let year: u32 = year
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()?;
    let month: u32 = month
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()?;
    if !(1..=12).contains(&month) {
        return None;
    }
    Some(format!("{year:04}-{month:02}"))
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), Box<dyn Error>> {
    fs::write(path, serde_json::to_vec(value)?)?;
    Ok(())
}

fn verify_representative_samples(
    month_by_code: &HashMap<String, String>,
    items: &[ItemLookup],
    transformed_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let samples = Path::new("inspections/consumer_price_index/samples");
    let mut mismatches = 0usize;
    let mut examples = Vec::new();
    for (item_code, area_code) in [
        ("0002", "00000"),
        ("0002", "13100"),
        ("1001", "00000"),
        ("1001", "13100"),
    ] {
        let filename = format!("index_item_{item_code}_area_{area_code}_series.json");
        let sample_path = samples.join(filename);
        if !sample_path.exists() {
            continue;
        }
        let sample: Value = serde_json::from_slice(&fs::read(sample_path)?)?;
        let Some(item_index) = items.iter().position(|item| item.code == item_code) else {
            mismatches += 1;
            if examples.len() < 5 {
                examples.push(format!("item {item_code} missing from current metadata"));
            }
            continue;
        };
        let chunk_start = item_index / ITEMS_PER_CHUNK * ITEMS_PER_CHUNK;
        let chunk_end = (chunk_start + ITEMS_PER_CHUNK).min(items.len()) - 1;
        let transformed = transformed_dir.join("series").join(format!(
            "area_{area_code}_items_{chunk_start:04}_{chunk_end:04}.json"
        ));
        let Ok(bytes) = fs::read(&transformed) else {
            mismatches += 1;
            if examples.len() < 5 {
                examples.push(format!("{item_code}/{area_code} missing from current data"));
            }
            continue;
        };
        let docs: Value = serde_json::from_slice(&bytes)?;
        let Some(series) = as_items(&docs)
            .into_iter()
            .find(|doc| attr(doc, "itemCode") == Some(item_code))
        else {
            mismatches += 1;
            if examples.len() < 5 {
                examples.push(format!("{item_code}/{area_code} missing from current data"));
            }
            continue;
        };
        let values = as_items(&sample["GET_STATS_DATA"]["STATISTICAL_DATA"]["DATA_INF"]["VALUE"]);
        for record in values {
            if attr(record, "@tab") != Some("1") || attr(record, "@cat01") != Some(item_code) {
                continue;
            }
            let time_code = required_attr(record, "@time")?;
            let Some(month) = month_by_code.get(time_code) else {
                // Older captures also included annual rows.
                continue;
            };
            let pair = as_items(&series["timeValuePairs"])
                .into_iter()
                .find(|pair| pair[0].as_str() == Some(month.as_str()));
            let Ok(source_value) = required_attr(record, "$")?.parse::<f64>() else {
                continue;
            };
            if pair.and_then(|pair| pair[1].as_f64()) != Some(source_value) {
                mismatches += 1;
                if examples.len() < 5 {
                    examples.push(format!("{item_code}/{area_code}/{month}"));
                }
            }
        }
    }
    if mismatches > 0 {
        eprintln!(
            "Warning: {mismatches} representative sample values differ from the saved historical capture (possible e-Stat revisions); examples: {}",
            examples.join(", ")
        );
    }
    Ok(())
}
