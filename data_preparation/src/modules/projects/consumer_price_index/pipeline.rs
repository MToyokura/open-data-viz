use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::modules::estat_api::{
    get_meta_info::{GetMetaInfoParamsBuilder, MandatoryParams as MetaParams, get_meta_info},
    get_stats_data::{GetStatsDataParamsBuilder, MandatoryParams as DataParams, get_stats_data},
};

use super::{
    ITEMS_PER_CHUNK, OUTPUT_DIR_META, OUTPUT_DIR_RAW, OUTPUT_DIR_TRANSFORMED, PAGE_SIZE,
    STATS_DATA_ID, path,
};

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
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

fn safe_api_error(endpoint: &str, error: &(dyn Error + 'static)) -> String {
    if let Some(error) = error.downcast_ref::<reqwest::Error>() {
        let reason = if error.is_timeout() {
            "timed out"
        } else if error.is_connect() {
            "could not connect"
        } else {
            "HTTP request failed"
        };
        return format!("e-Stat {endpoint} {reason}");
    }
    if error.downcast_ref::<serde_json::Error>().is_some() {
        return format!("e-Stat {endpoint} returned invalid JSON");
    }
    format!("e-Stat {endpoint} failed")
}

/// Fetch metadata and all monthly index observations, retaining each raw page.
pub fn acquire(app_id: &str) -> Result<(), Box<dyn Error>> {
    if app_id.is_empty() {
        return Err("e-Stat application ID is required".into());
    }
    let raw_dir = path(OUTPUT_DIR_RAW);
    let meta_dir = path(OUTPUT_DIR_META);
    if raw_dir.exists() {
        fs::remove_dir_all(&raw_dir)?;
    }
    if meta_dir.exists() {
        fs::remove_dir_all(&meta_dir)?;
    }
    fs::create_dir_all(&raw_dir)?;
    fs::create_dir_all(&meta_dir)?;
    let mut metadata: Value = serde_json::from_str(
        &get_meta_info(
            GetMetaInfoParamsBuilder::new(MetaParams {
                app_id,
                stats_data_id: STATS_DATA_ID,
            })
            .lang("J")
            .build(),
        )
        .map_err(|error| safe_api_error("metadata request", error.as_ref()))?,
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
        let response = get_stats_data(
            GetStatsDataParamsBuilder::new(DataParams { app_id })
                .stats_data_id(STATS_DATA_ID)
                .cd_tab("1")
                .lv_time("4")
                .start_position(start)
                .limit(PAGE_SIZE)
                .meta_get_flg("N")
                .build(),
        )
        .map_err(|error| {
            format!(
                "{} at position {start}",
                safe_api_error("data request", error.as_ref())
            )
        })?;
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
        if from != u64::from(start) || to != expected_to {
            return Err(format!(
                "Invalid e-Stat page range {from}-{to} at requested position {start}"
            )
            .into());
        }
        let next = parse_u64(&info["NEXT_KEY"]).ok_or("Missing RESULT_INF.NEXT_KEY")?;
        redact_app_id(&mut page);
        fs::write(
            raw_dir.join(format!("get_stats_data_{start:010}.json")),
            serde_json::to_vec(&page)?,
        )?;
        println!("Saved CPI records {from}-{to} of {total}");
        if to == total {
            if next <= total {
                return Err(
                    "e-Stat final page NEXT_KEY did not advance beyond TOTAL_NUMBER".into(),
                );
            }
            break;
        }
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
    transform()
}

/// Rebuild lookup and series files from saved metadata and raw page responses.
pub fn transform() -> Result<(), Box<dyn Error>> {
    let metadata: Value = serde_json::from_slice(&fs::read(
        Path::new(OUTPUT_DIR_RAW).join("get_meta_info.json"),
    )?)?;
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

    let mut raw_paths: Vec<PathBuf> = fs::read_dir(OUTPUT_DIR_RAW)?
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
    let mut expected_total = None;
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
        if records.is_empty() || to - from + 1 != records.len() as u64 {
            return Err(format!("Invalid saved page boundaries in {}", raw_path.display()).into());
        }
        for record in records {
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
            let numeric_value: f64 = value_text.parse().map_err(|_| {
                format!("Non-numeric index observation {value_text:?} at time {time_code}")
            })?;
            let chunk = *item_chunks
                .get(item_code)
                .ok_or_else(|| format!("Unknown CPI item code {item_code}"))?;
            observed_total += 1;
            grouped
                .entry((area_code.to_owned(), chunk))
                .or_default()
                .entry(item_code.to_owned())
                .or_default()
                .push((month.clone(), numeric_value));
        }
    }
    let expected_total = expected_total.ok_or("No records found in saved CPI pages")?;
    if observed_total != expected_total {
        return Err(format!(
            "Transformed {observed_total} observations, expected {expected_total}"
        )
        .into());
    }

    let transformed_dir = path(OUTPUT_DIR_TRANSFORMED);
    if transformed_dir.exists() {
        fs::remove_dir_all(&transformed_dir)?;
    }
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
    let observed_at = raw_observation_range(&transformed_dir.join("months.json"))?;
    write_json(
        &transformed_dir.join("manifest.json"),
        &serde_json::json!({
            "tableId": STATS_DATA_ID,
            "measureCode": "1",
            "monthlyTimeLevel": "4",
            "observationCount": observed_total,
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
    verify_representative_samples(&month_by_code, &items)?;
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

fn raw_observation_range(months_path: &Path) -> Result<Value, Box<dyn Error>> {
    let months: Vec<MonthLookup> = serde_json::from_slice(&fs::read(months_path)?)?;
    let mut values: Vec<String> = months.into_iter().map(|entry| entry.year_month).collect();
    values.sort();
    Ok(serde_json::json!({"first": values.first(), "last": values.last()}))
}

fn verify_representative_samples(
    month_by_code: &HashMap<String, String>,
    items: &[ItemLookup],
) -> Result<(), Box<dyn Error>> {
    let samples = Path::new("inspections/consumer_price_index/samples");
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
        let item_index = items
            .iter()
            .position(|item| item.code == item_code)
            .ok_or("Representative item was missing from metadata lookup")?;
        let chunk_start = item_index / ITEMS_PER_CHUNK * ITEMS_PER_CHUNK;
        let chunk_end = (chunk_start + ITEMS_PER_CHUNK).min(items.len()) - 1;
        let transformed = path(OUTPUT_DIR_TRANSFORMED).join("series").join(format!(
            "area_{area_code}_items_{chunk_start:04}_{chunk_end:04}.json"
        ));
        let docs: Value = serde_json::from_slice(&fs::read(&transformed)?)?;
        let series = as_items(&docs)
            .into_iter()
            .find(|doc| attr(doc, "itemCode") == Some(item_code))
            .ok_or("Representative item series missing from transformed chunk")?;
        let values = as_items(&sample["GET_STATS_DATA"]["STATISTICAL_DATA"]["DATA_INF"]["VALUE"]);
        for record in values {
            if attr(record, "@tab") != Some("1") || attr(record, "@cat01") != Some(item_code) {
                continue;
            }
            let time_code = required_attr(record, "@time")?;
            let month = month_by_code
                .get(time_code)
                .ok_or("Representative sample referenced an unknown monthly code")?;
            let pair = as_items(&series["timeValuePairs"])
                .into_iter()
                .find(|pair| pair[0].as_str() == Some(month.as_str()))
                .ok_or("Representative sample month missing from transformed series")?;
            let source_value: f64 = required_attr(record, "$")?.parse()?;
            if pair[1].as_f64() != Some(source_value) {
                return Err(format!("Sample mismatch for {item_code}/{area_code}/{month}").into());
            }
        }
    }
    Ok(())
}
