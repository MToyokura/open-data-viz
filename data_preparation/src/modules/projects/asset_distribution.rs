//! Acquisition entry point for e-Stat table 0004056120.
//!
//! The table's classifications must be derived from the captured metadata. In
//! particular, no classification codes are duplicated in this module.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    error::Error,
    fs,
    path::PathBuf,
    thread,
    time::Duration,
};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::modules::estat_api::{
    get_meta_info::{GetMetaInfoParamsBuilder, MandatoryParams as MetaParams, get_meta_info},
    get_stats_data::{GetStatsDataParamsBuilder, MandatoryParams as DataParams, get_stats_data},
};

const STATS_DATA_ID: &str = "0004056120";
const OUTPUT_DIR: &str = "output/asset_distribution";
const PAGE_SIZE: u32 = 100_000;
const CHUNK_MAX_BYTES: usize = 1024 * 1024;
const CHUNK_TARGET_BYTES: usize = 256 * 1024;
const CHUNK_MAX_COMBINATIONS: usize = 128;

pub fn run(args: &[String]) -> bool {
    if args.first().map(String::as_str) != Some("asset-distribution") {
        return false;
    }

    if args
        .get(1)
        .is_some_and(|arg| arg == "--help" || arg == "-h")
    {
        println!("Usage: cargo run -- asset-distribution [--transform-only]");
        println!("Fetches and transforms e-Stat table {STATS_DATA_ID}.");
        return true;
    }

    let result = match args.get(1).map(String::as_str) {
        Some("--transform-only") if args.len() == 2 => transform(),
        None => {
            let app_id = match crate::modules::estat_api::app_id::retrieve_estat_app_id() {
                Ok(app_id) => app_id,
                Err(_) => {
                    eprintln!("ESTAT_APP_ID not found");
                    std::process::exit(1);
                }
            };
            acquire(&app_id).and_then(|_| transform())
        }
        Some(_) => {
            eprintln!("Unknown asset-distribution option; use --help for usage.");
            std::process::exit(2);
        }
    };
    if let Err(error) = result {
        eprintln!("Asset distribution acquisition failed: {error}");
        std::process::exit(1);
    }
    true
}

fn acquire(app_id: &str) -> Result<(), Box<dyn Error>> {
    let target = PathBuf::from(OUTPUT_DIR);
    if target.exists() {
        fs::remove_dir_all(&target)?;
    }
    fs::create_dir_all(target.join("meta_data"))?;
    fs::create_dir_all(target.join("raw_data"))?;

    let meta_response = get_meta_info(
        GetMetaInfoParamsBuilder::new(MetaParams {
            app_id,
            stats_data_id: STATS_DATA_ID,
        })
        .lang("J")
        .build(),
    )
    .map_err(|error| safe_api_error("metadata request", error.as_ref()))?;
    let mut metadata: Value = serde_json::from_str(&meta_response)?;
    redact_app_id(&mut metadata);
    fs::write(
        target.join("meta_data/get_meta_info.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;

    let mut start_position = 1_u32;
    let mut expected_total = None;
    let mut captured = 0_u64;
    loop {
        if start_position > 1 {
            thread::sleep(Duration::from_secs(2));
        }
        let data_response = get_stats_data(
            GetStatsDataParamsBuilder::new(DataParams { app_id })
                .stats_data_id(STATS_DATA_ID)
                .start_position(start_position)
                .limit(PAGE_SIZE)
                .meta_get_flg("N")
                .build(),
        )
        .map_err(|error| safe_api_error("data request", error.as_ref()))?;
        let mut page: Value = serde_json::from_str(&data_response)?;
        let statistical_data = &page["GET_STATS_DATA"]["STATISTICAL_DATA"];
        let result = &statistical_data["RESULT_INF"];
        let total = parse_u64(&result["TOTAL_NUMBER"])
            .ok_or("e-Stat response omitted a valid RESULT_INF.TOTAL_NUMBER")?;
        if expected_total.is_some_and(|expected| expected != total) {
            return Err("e-Stat changed TOTAL_NUMBER while paging".into());
        }
        expected_total = Some(total);
        let values = &statistical_data["DATA_INF"]["VALUE"];
        let count = match values {
            Value::Array(items) => items.len() as u64,
            Value::Object(_) => 1,
            Value::Null => 0,
            _ => return Err("Unexpected DATA_INF.VALUE shape".into()),
        };
        if count == 0 && total > 0 {
            return Err(format!("e-Stat returned an empty page at {start_position}").into());
        }
        if count > 0 {
            let from = parse_u64(&result["FROM_NUMBER"])
                .ok_or("e-Stat response omitted RESULT_INF.FROM_NUMBER")?;
            let to = parse_u64(&result["TO_NUMBER"])
                .ok_or("e-Stat response omitted RESULT_INF.TO_NUMBER")?;
            if from != u64::from(start_position) || to != from + count - 1 {
                return Err(format!("Unexpected e-Stat page boundary at {start_position}").into());
            }
        }
        redact_app_id(&mut page);
        fs::write(
            target
                .join("raw_data")
                .join(format!("get_stats_data_{start_position:010}.json")),
            serde_json::to_vec_pretty(&page)?,
        )?;
        captured += count;
        println!("Fetched records beginning at {start_position}: {count}");
        if captured == total {
            break;
        }
        if captured > total {
            return Err("Captured more e-Stat records than TOTAL_NUMBER".into());
        }
        start_position = start_position
            .checked_add(u32::try_from(count)?)
            .ok_or("e-Stat page position overflow")?;
    }

    if captured != expected_total.unwrap_or_default() {
        return Err("Captured record count does not match e-Stat TOTAL_NUMBER".into());
    }
    let manifest = serde_json::json!({
        "stats_data_id": STATS_DATA_ID,
        "captured_records": captured,
        "complete": true,
    });
    fs::write(
        target.join("capture_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;

    println!("Captured {captured} records in {OUTPUT_DIR}");
    Ok(())
}

#[derive(Clone)]
struct Category {
    code: String,
    label: String,
    kind: String,
    order: usize,
    api_level: Option<String>,
    api_parent_code: Option<String>,
}

#[derive(Clone)]
struct Dimension {
    id: String,
    label: String,
    role: String,
    order: usize,
    total_code: Option<String>,
    categories: Vec<Category>,
    comparison_sets: Vec<Value>,
    display_category_codes: Vec<String>,
}

pub fn transform() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(OUTPUT_DIR);
    let metadata: Value =
        serde_json::from_slice(&fs::read(root.join("meta_data/get_meta_info.json"))?)?;
    let class_objs = as_items(&metadata["GET_META_INFO"]["METADATA_INF"]["CLASS_INF"]["CLASS_OBJ"]);
    let measure_code = metadata_category_code(&class_objs, "tab", "世帯数分布")?;
    let area_code = metadata_category_code(&class_objs, "area", "全国")?;
    let year_code = metadata_category_code(&class_objs, "time", "2024年")?;
    let published_at = metadata["GET_META_INFO"]["METADATA_INF"]["TABLE_INF"]["OPEN_DATE"]
        .as_str()
        .ok_or("Metadata omitted TABLE_INF.OPEN_DATE")?
        .to_owned();
    let data_dims = ["cat01", "cat02", "cat03", "cat04", "cat05"];
    let labels = [
        ("cat01", "世帯主の年齢階級7区分"),
        ("cat02", "世帯主の学歴6区分"),
        ("cat03", "年間可処分所得階級23区分"),
        ("cat04", "資産額階級36区分"),
        ("cat05", "世帯人員7区分"),
    ];
    let mut dimensions = Vec::new();
    for (order, id) in data_dims.iter().enumerate() {
        let meta_dim = class_objs
            .iter()
            .find(|d| attr(d, "@id") == Some(id))
            .ok_or_else(|| format!("Missing inspected metadata axis {id}"))?;
        let label = required_attr(meta_dim, "@name")?.to_owned();
        let expected_label = labels.iter().find(|(dim_id, _)| dim_id == id).unwrap().1;
        if label != expected_label {
            return Err(format!("Unexpected classification axis name for {id}: {label}").into());
        }
        let cats = as_items(&meta_dim["CLASS"]);
        let mut categories = Vec::new();
        for (category_order, item) in cats.iter().enumerate() {
            let code = required_attr(item, "@code")?.to_owned();
            let name = required_attr(item, "@name")?.to_owned();
            let kind = classify_category(id, &name);
            categories.push(Category {
                code,
                label: name,
                kind,
                order: category_order,
                api_level: attr(item, "@level").map(str::to_owned),
                api_parent_code: attr(item, "@parentCode").map(str::to_owned),
            });
        }
        if id == &"cat02" {
            validate_education_categories(&categories)?;
        }
        let total_code = if id == &"cat04" {
            None
        } else {
            Some(
                categories
                    .iter()
                    .find(|c| c.label == "平均")
                    .ok_or("API metadata omitted the official 平均 total category")?
                    .code
                    .clone(),
            )
        };
        let role = if id == &"cat04" {
            "asset_class"
        } else {
            "filter"
        };
        let comparison_sets = if id == &"cat02" {
            vec![
                make_set(
                    "education-status",
                    "在学中・卒業者全体",
                    categories
                        .iter()
                        .filter(|c| c.label == "在学中" || c.label == "卒業")
                        .collect(),
                ),
                make_set(
                    "graduated-subgroups",
                    "卒業者の学歴内訳",
                    categories
                        .iter()
                        .filter(|c| {
                            ["小学校・中学校", "高校程度", "短大程度以上"]
                                .contains(&c.label.as_str())
                        })
                        .collect(),
                ),
            ]
        } else if id == &"cat03" {
            vec![
                make_set(
                    "income-brackets",
                    "年間可処分所得階級",
                    categories.iter().filter(|c| c.kind == "regular").collect(),
                ),
                make_set(
                    "income-deciles",
                    "年間可処分所得十分位階級",
                    categories.iter().filter(|c| c.kind == "decile").collect(),
                ),
            ]
        } else if id != &"cat04" {
            vec![make_set(
                "regular",
                "通常区分",
                categories.iter().filter(|c| c.kind == "regular").collect(),
            )]
        } else {
            Vec::new()
        };
        let display_category_codes = if id == &"cat04" {
            resolve_display_category_codes(&categories)?
        } else {
            Vec::new()
        };
        dimensions.push(Dimension {
            id: (*id).to_owned(),
            label,
            role: role.to_owned(),
            order,
            total_code,
            categories,
            comparison_sets,
            display_category_codes,
        });
    }
    if dimensions
        .iter()
        .filter(|d| d.role == "asset_class")
        .count()
        != 1
    {
        return Err("Could not uniquely identify the asset-class dimension".into());
    }

    let notes = load_notes(&root)?;
    let mut records: BTreeMap<Vec<String>, String> = BTreeMap::new();
    let mut expected_total = None;
    let mut captured = 0_u64;
    let mut raw_paths: Vec<_> = fs::read_dir(root.join("raw_data"))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    raw_paths.sort();
    if raw_paths.is_empty() {
        return Err("No captured API pages are available".into());
    }
    for path in &raw_paths {
        let page: Value = serde_json::from_slice(&fs::read(path)?)?;
        let statistical_data = &page["GET_STATS_DATA"]["STATISTICAL_DATA"];
        let total = parse_u64(&statistical_data["RESULT_INF"]["TOTAL_NUMBER"])
            .ok_or("Raw page omitted TOTAL_NUMBER")?;
        if expected_total.is_some_and(|prev| prev != total) {
            return Err("Raw pages disagree on TOTAL_NUMBER".into());
        }
        expected_total = Some(total);
        let values = as_items(&statistical_data["DATA_INF"]["VALUE"]);
        captured += values.len() as u64;
        for row in values {
            if attr(row, "@tab") != Some(measure_code.as_str())
                || attr(row, "@area") != Some(area_code.as_str())
                || attr(row, "@time") != Some(year_code.as_str())
            {
                return Err(
                    "Captured table contains an unexpected measure, area, or time code".into(),
                );
            }
            let key = data_dims
                .iter()
                .map(|id| required_attr(row, &format!("@{id}")).map(str::to_owned))
                .collect::<Result<Vec<_>, _>>()?;
            if records
                .insert(key, required_attr(row, "$")?.to_owned())
                .is_some()
            {
                return Err("Duplicate source record found across captured pages".into());
            }
        }
    }
    if Some(captured) != expected_total {
        return Err(format!(
            "Captured {captured} rows but e-Stat TOTAL_NUMBER is {}",
            expected_total.unwrap_or_default()
        )
        .into());
    }

    let axis_by_id: HashMap<_, _> = dimensions.iter().map(|d| (d.id.as_str(), d)).collect();
    let attribute_dims: Vec<_> = dimensions.iter().filter(|d| d.role == "filter").collect();
    let asset_dimension = dimensions.iter().find(|d| d.role == "asset_class").unwrap();
    let display_codes = &asset_dimension.display_category_codes;
    let expected_combinations = attribute_dims.iter().try_fold(1_usize, |total, d| {
        total
            .checked_mul(d.categories.len())
            .ok_or("Combination count overflow")
    })?;
    let mut combinations = Vec::with_capacity(expected_combinations);
    let mut items_by_combination = Vec::with_capacity(expected_combinations);
    let mut all_items = Vec::with_capacity(expected_combinations * display_codes.len());
    for codes in cartesian_codes(&attribute_dims) {
        let combination_id = codes
            .iter()
            .map(|(id, code)| format!("{id}={code}"))
            .collect::<Vec<_>>()
            .join("|");
        let mut combo_dims = serde_json::Map::new();
        for (id, code) in &codes {
            combo_dims.insert(id.clone(), Value::String(code.clone()));
        }
        combinations.push(serde_json::json!({"id": combination_id, "dimensions": combo_dims, "status": "published", "reason": null}));
        let mut combination_items = Vec::with_capacity(display_codes.len());
        for asset in &asset_dimension.categories {
            let mut key: Vec<String> = dimensions
                .iter()
                .filter(|d| d.id != "cat04")
                .take(3)
                .map(|d| codes.iter().find(|(id, _)| id == &d.id).unwrap().1.clone())
                .collect();
            key.push(asset.code.clone());
            key.push(
                codes
                    .iter()
                    .find(|(id, _)| id == "cat05")
                    .unwrap()
                    .1
                    .clone(),
            );
            let raw_value = records.remove(&key).ok_or_else(|| {
                format!("Source row missing for {} / {}", combination_id, asset.code)
            })?;
            if !display_codes.contains(&asset.code) {
                continue;
            }
            let (count, status, annotations) = classify_value(&raw_value, &notes)?;
            let item = serde_json::json!({"combination_id": combination_id, "asset_class_code": asset.code, "count": count, "status": status, "raw_value": raw_value, "annotations": annotations});
            combination_items.push(item.clone());
            all_items.push(item);
        }
        combination_items.sort_by_key(|item| {
            display_codes
                .iter()
                .position(|code| item["asset_class_code"] == *code)
                .unwrap()
        });
        items_by_combination.push(combination_items);
    }
    if !records.is_empty() {
        return Err(format!(
            "{} unexpected source rows were not classified",
            records.len()
        )
        .into());
    }
    let _ = &axis_by_id;

    let metadata_out = serde_json::json!({
        "year": 2024,
        "published_at": published_at,
        "area": "全国",
        "measure": "世帯数",
        "asset_basis": "純資産総額（純金融資産＋住宅・宅地）",
        "unit": "世帯",
        "unit_basis": "表題の世帯数。e-Stat APIの@unitは応答に含まれない",
        "source_url": "https://www.e-stat.go.jp/dbview?sid=0004056120"
    });
    let dimensions_out = dimensions.iter().map(dimension_json).collect::<Vec<_>>();
    // Keep the inspection sample workflow based on the same complete transformed
    // records, while only publishing the manifest and lazy value chunks.
    let inspection_output = serde_json::json!({"metadata": metadata_out.clone(), "dimensions": dimensions_out.clone(), "combinations": combinations.clone(), "items": all_items});
    write_inspection_samples(
        &metadata,
        &class_objs,
        &raw_paths,
        &inspection_output,
        &measure_code,
        &area_code,
        &year_code,
    )?;
    let chunk_count = write_distribution_bundle(
        &root,
        metadata_out,
        dimensions_out,
        &mut combinations,
        items_by_combination,
    )?;
    println!(
        "Transformed {} published combinations and {} display-class values into {} chunks in {}",
        expected_combinations,
        expected_combinations * display_codes.len(),
        chunk_count,
        root.display()
    );
    Ok(())
}

/// Write and validate the browser-facing manifest/chunks directly in the
/// generated output directory. Chunks are formed in deterministic combination
/// order and never split a series across files.
fn write_distribution_bundle(
    root: &PathBuf,
    metadata: Value,
    dimensions: Vec<Value>,
    combinations: &mut [Value],
    items_by_combination: Vec<Vec<Value>>,
) -> Result<usize, Box<dyn Error>> {
    if combinations.len() != items_by_combination.len() {
        return Err("Combination and item-series counts do not match".into());
    }

    let groups = plan_chunk_groups(combinations, &items_by_combination)?;
    // Estimate sizes while planning, then measure the real pretty-printed bytes.
    // If any estimate was low, split that candidate and remeasure its children.
    let finalized_chunks = serialize_planned_chunks(groups, &items_by_combination)?;

    let manifest_path = root.join("manifest.json");
    if manifest_path.exists() {
        fs::remove_file(&manifest_path)?;
    }
    let items_path = root.join("items");
    if items_path.exists() {
        fs::remove_dir_all(&items_path)?;
    }
    let legacy_path = root.join("asset_distribution.json");
    if legacy_path.exists() {
        fs::remove_file(legacy_path)?;
    }
    fs::create_dir_all(&items_path)?;

    let mut manifest_chunks = Vec::with_capacity(finalized_chunks.len());
    let mut next_start_index = 0_usize;
    for (chunk_index, (group, bytes)) in finalized_chunks.iter().enumerate() {
        if group.is_empty() || group[0] != next_start_index {
            return Err("Chunk groups are not contiguous combination ranges".into());
        }
        for (offset, &combination_index) in group.iter().enumerate() {
            if combination_index != next_start_index + offset {
                return Err("Chunk group is not a contiguous combination range".into());
            }
        }
        let relative_path = format!("items/{chunk_index:05}.json");
        fs::write(root.join(&relative_path), bytes)?;
        manifest_chunks.push(serde_json::json!({
            "path": relative_path,
            "bytes": bytes.len(),
            "start_index": group[0],
            "count": group.len()
        }));
        next_start_index += group.len();
    }
    if next_start_index != combinations.len() {
        return Err("Chunk ranges do not cover every combination".into());
    }

    let manifest = serde_json::json!({
        "schema_version": 3,
        "metadata": metadata,
        "dimensions": dimensions,
        "combination_index": {
            "dimension_ids": dimensions.iter().filter(|dimension| dimension["role"] == "filter").filter_map(|dimension| dimension["id"].as_str().map(str::to_owned)).collect::<Vec<_>>(),
            "count": combinations.len()
        },
        "chunks": manifest_chunks
    });
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
    validate_distribution_bundle(root, &manifest, combinations, &items_by_combination)?;
    Ok(finalized_chunks.len())
}

fn plan_chunk_groups(
    combinations: &[Value],
    items_by_combination: &[Vec<Value>],
) -> Result<Vec<Vec<usize>>, Box<dyn Error>> {
    let empty_chunk_bytes = chunk_json_bytes(&[])?.len();
    let mut estimates = Vec::with_capacity(items_by_combination.len());
    for (index, series) in items_by_combination.iter().enumerate() {
        let status = combinations[index]["status"]
            .as_str()
            .ok_or("Generated combination omitted status")?;
        if status != "published" {
            if !series.is_empty() {
                return Err(format!(
                    "Non-published combination {} has generated values",
                    combinations[index]["id"].as_str().unwrap_or("(unknown)")
                )
                .into());
            }
            continue;
        }
        let wrapped = serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": 3,
            "items": series
        }))?;
        // Each series serialized alone includes the same pretty-print nesting as
        // in a chunk. Add separator allowance for combining neighboring series.
        estimates.push((index, wrapped.len().saturating_sub(empty_chunk_bytes) + 16));
    }

    let mut groups = Vec::new();
    let mut current = Vec::new();
    let mut current_estimated_size = empty_chunk_bytes;
    for (index, series_size) in estimates {
        let candidate_size = current_estimated_size.saturating_add(series_size);
        if !current.is_empty()
            && (current.len() >= CHUNK_MAX_COMBINATIONS
                || candidate_size > CHUNK_MAX_BYTES
                || candidate_size > CHUNK_TARGET_BYTES)
        {
            groups.push(std::mem::take(&mut current));
            current_estimated_size = empty_chunk_bytes;
        }
        current.push(index);
        current_estimated_size = current_estimated_size.saturating_add(series_size);
        if current.len() >= CHUNK_MAX_COMBINATIONS || current_estimated_size >= CHUNK_TARGET_BYTES {
            groups.push(std::mem::take(&mut current));
            current_estimated_size = empty_chunk_bytes;
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }
    Ok(groups)
}

fn serialize_planned_chunks(
    groups: Vec<Vec<usize>>,
    items_by_combination: &[Vec<Value>],
) -> Result<Vec<(Vec<usize>, Vec<u8>)>, Box<dyn Error>> {
    let mut pending: VecDeque<_> = groups.into();
    let mut finalized = Vec::new();
    while let Some(group) = pending.pop_front() {
        let mut items = Vec::new();
        for &index in &group {
            items.extend(items_by_combination[index].iter().cloned());
        }
        let bytes = chunk_json_bytes(&items)?;
        if bytes.len() > CHUNK_MAX_BYTES && group.len() > 1 {
            let right = group[group.len() / 2..].to_vec();
            let left = group[..group.len() / 2].to_vec();
            pending.push_front(right);
            pending.push_front(left);
        } else {
            if bytes.len() > CHUNK_MAX_BYTES {
                return Err("A single display series exceeds the chunk size limit".into());
            }
            finalized.push((group, bytes));
        }
    }
    Ok(finalized)
}

fn chunk_json_bytes(items: &[Value]) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(serde_json::to_vec_pretty(&serde_json::json!({
        "schema_version": 3,
        "items": items
    }))?)
}

fn validate_distribution_bundle(
    output_dir: &PathBuf,
    manifest: &Value,
    expected_combinations: &[Value],
    expected_items: &[Vec<Value>],
) -> Result<(), Box<dyn Error>> {
    let on_disk_manifest: Value =
        serde_json::from_slice(&fs::read(output_dir.join("manifest.json"))?)?;
    if &on_disk_manifest != manifest {
        return Err("Written manifest does not match generated manifest".into());
    }
    if manifest["schema_version"] != 3 {
        return Err("Unexpected manifest schema version".into());
    }
    if manifest.get("combinations").is_some() {
        return Err("Schema v3 manifest must not include combinations".into());
    }
    let dimensions = manifest["dimensions"]
        .as_array()
        .ok_or("Manifest omitted dimensions")?;
    let dimension_ids = manifest["combination_index"]["dimension_ids"]
        .as_array()
        .ok_or("Manifest omitted combination_index.dimension_ids")?;
    let ids: Vec<_> = dimension_ids
        .iter()
        .map(|id| id.as_str().ok_or("Invalid combination axis ID"))
        .collect::<Result<_, _>>()?;
    let mut unique_ids = BTreeSet::new();
    if ids.iter().any(|id| !unique_ids.insert(*id)) {
        return Err("Combination index repeats a dimension ID".into());
    }
    let filter_ids: Vec<_> = dimensions
        .iter()
        .filter(|d| d["role"] == "filter")
        .map(|d| d["id"].as_str().ok_or("Dimension omitted ID"))
        .collect::<Result<_, _>>()?;
    let all_dimension_ids: Vec<_> = dimensions
        .iter()
        .map(|d| d["id"].as_str().ok_or("Dimension omitted ID"))
        .collect::<Result<_, _>>()?;
    if all_dimension_ids.iter().collect::<BTreeSet<_>>().len() != all_dimension_ids.len() {
        return Err("Manifest repeats a dimension ID".into());
    }
    if let Some(education_dimension) = dimensions
        .iter()
        .find(|dimension| dimension["id"] == "cat02")
    {
        let education_categories = education_dimension["categories"]
            .as_array()
            .ok_or("Manifest omitted cat02 categories")?
            .iter()
            .enumerate()
            .map(|(order, category)| Category {
                code: category["code"].as_str().unwrap_or_default().to_owned(),
                label: category["label"].as_str().unwrap_or_default().to_owned(),
                kind: category["kind"].as_str().unwrap_or_default().to_owned(),
                order,
                api_level: category["api_level"].as_str().map(str::to_owned),
                api_parent_code: category["api_parent_code"].as_str().map(str::to_owned),
            })
            .collect::<Vec<_>>();
        validate_education_categories(&education_categories)?;
        if education_dimension["display_hierarchy"]
            != serde_json::json!({
                "standalone_codes": ["0", "1"],
                "groups": [{"code": "2", "children": ["21", "22", "23"]}]
            })
            || education_dimension["comparison_sets"]
                != serde_json::json!([
                    {"id":"education-status","label":"在学中・卒業者全体","category_codes":["1","2"]},
                    {"id":"graduated-subgroups","label":"卒業者の学歴内訳","category_codes":["21","22","23"]}
                ])
        {
            return Err(
                "Education display hierarchy or comparison sets changed unexpectedly".into(),
            );
        }
    }
    if ids.len() != filter_ids.len() || ids.iter().any(|id| !filter_ids.contains(id)) {
        return Err(
            "Combination index axes must contain every filter dimension exactly once".into(),
        );
    }
    if ids
        .iter()
        .any(|id| dimensions.iter().find(|d| d["id"] == *id).is_none())
    {
        return Err("Combination index contains an unknown dimension".into());
    }
    if dimensions
        .iter()
        .any(|d| d["role"] == "asset_class" && ids.contains(&d["id"].as_str().unwrap_or("")))
    {
        return Err("Combination index must not contain the asset-class dimension".into());
    }
    let dimension_by_id: HashMap<_, _> = dimensions
        .iter()
        .map(|d| (d["id"].as_str().unwrap_or(""), d))
        .collect();
    let mut product = 1_usize;
    for id in &ids {
        let categories = dimension_by_id[id]["categories"]
            .as_array()
            .ok_or("Filter dimension omitted categories")?;
        if categories.is_empty() {
            return Err(format!("Dimension {id} has no categories").into());
        }
        let category_codes: Vec<_> = categories
            .iter()
            .map(|c| c["code"].as_str().ok_or("Category omitted code"))
            .collect::<Result<_, _>>()?;
        if category_codes.iter().collect::<BTreeSet<_>>().len() != category_codes.len() {
            return Err(format!("Dimension {id} repeats a category code").into());
        }
        product = product
            .checked_mul(categories.len())
            .ok_or("Combination count overflow")?;
    }
    if manifest["combination_index"]["count"].as_u64() != Some(product as u64)
        || expected_combinations.len() != product
        || expected_items.len() != product
    {
        return Err("Combination index count does not match the axis product".into());
    }
    let category_lists: Vec<Vec<&str>> = ids
        .iter()
        .map(|id| {
            dimension_by_id[id]["categories"]
                .as_array()
                .unwrap()
                .iter()
                .map(|category| category["code"].as_str().unwrap())
                .collect()
        })
        .collect();
    let mut expected_tuples = vec![Vec::<&str>::new()];
    for categories in &category_lists {
        let mut next = Vec::with_capacity(expected_tuples.len() * categories.len());
        for tuple in &expected_tuples {
            for code in categories {
                let mut expanded = tuple.clone();
                expanded.push(code);
                next.push(expanded);
            }
        }
        expected_tuples = next;
    }
    for (index, (combination, tuple)) in expected_combinations
        .iter()
        .zip(expected_tuples)
        .enumerate()
    {
        let expected_id = ids
            .iter()
            .zip(&tuple)
            .map(|(axis, code)| format!("{axis}={code}"))
            .collect::<Vec<_>>()
            .join("|");
        if combination["id"].as_str() != Some(expected_id.as_str()) {
            return Err(format!(
                "Generated combination {index} does not match the declared cartesian order"
            )
            .into());
        }
        let combination_dims = combination["dimensions"]
            .as_object()
            .ok_or("Generated combination omitted dimensions")?;
        if combination_dims.len() != ids.len()
            || ids.iter().zip(&tuple).any(|(axis, code)| {
                combination_dims.get(*axis).and_then(Value::as_str) != Some(code)
            })
        {
            return Err(format!("Generated combination {index} has invalid axis codes").into());
        }
    }
    let asset_dimension = manifest["dimensions"]
        .as_array()
        .ok_or("Manifest omitted dimensions")?
        .iter()
        .find(|dimension| dimension["id"] == "cat04")
        .ok_or("Manifest omitted cat04")?;
    let categories = asset_dimension["categories"]
        .as_array()
        .ok_or("Manifest omitted asset categories")?
        .iter()
        .enumerate()
        .map(|(order, category)| {
            Ok(Category {
                code: required_attr(category, "code")?.to_owned(),
                label: required_attr(category, "label")?.to_owned(),
                kind: required_attr(category, "kind")?.to_owned(),
                order,
                api_level: None,
                api_parent_code: None,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    let display_codes = resolve_display_category_codes(&categories)?;
    if asset_dimension["display_category_codes"] != serde_json::json!(display_codes) {
        return Err("Manifest display categories do not match the official breakdown".into());
    }
    let chunks = manifest["chunks"]
        .as_array()
        .ok_or("Generated manifest omitted chunks")?;
    let mut seen_combinations = BTreeSet::new();
    let mut seen_values = BTreeSet::new();
    let mut seen_item_records = BTreeMap::new();
    let mut item_counts_by_combination: HashMap<String, usize> = HashMap::new();
    let mut chunk_by_combination: HashMap<String, String> = HashMap::new();
    let mut next_start_index = 0_usize;
    let mut referenced_chunks = BTreeSet::new();
    for chunk_ref in chunks {
        let relative_path = chunk_ref["path"]
            .as_str()
            .ok_or("Manifest chunk omitted path")?;
        if !relative_path.starts_with("items/") || relative_path.contains("..") {
            return Err(format!("Unsafe chunk path in manifest: {relative_path}").into());
        }
        if fs::metadata(output_dir.join(relative_path))?.len() > CHUNK_MAX_BYTES as u64 {
            return Err(format!("Chunk {relative_path} exceeds the size limit").into());
        }
        if !referenced_chunks.insert(relative_path) {
            return Err("Manifest repeats a chunk path".into());
        }
        let start = chunk_ref["start_index"]
            .as_u64()
            .ok_or("Chunk omitted start_index")? as usize;
        let count = chunk_ref["count"].as_u64().ok_or("Chunk omitted count")? as usize;
        if count == 0
            || start != next_start_index
            || start.checked_add(count).map_or(true, |end| end > product)
        {
            return Err("Chunk ranges overlap, have gaps, or exceed the combination index".into());
        }
        next_start_index += count;
        let chunk: Value = serde_json::from_slice(&fs::read(output_dir.join(relative_path))?)?;
        if chunk["schema_version"] != 3 {
            return Err(format!("Unexpected schema version in {relative_path}").into());
        }
        let item_rows = chunk["items"]
            .as_array()
            .ok_or("Chunk omitted items array")?;
        let expected_ids: BTreeSet<_> = expected_combinations[start..start + count]
            .iter()
            .map(|c| c["id"].as_str().ok_or("Generated combination omitted ID"))
            .collect::<Result<_, _>>()?;
        if chunk_ref["bytes"].as_u64() != Some(fs::metadata(output_dir.join(relative_path))?.len())
        {
            return Err(
                format!("Chunk byte size does not match manifest for {relative_path}").into(),
            );
        }
        let mut actual_ids = BTreeSet::new();
        for item in item_rows {
            let combination_id = item["combination_id"]
                .as_str()
                .ok_or("Chunk item omitted combination_id")?;
            let asset_code = item["asset_class_code"]
                .as_str()
                .ok_or("Chunk item omitted asset_class_code")?;
            if !display_codes.iter().any(|code| code == asset_code) {
                return Err(format!("Unexpected display category {asset_code}").into());
            }
            let combo_key = combination_id.to_owned();
            if !expected_ids.contains(combination_id) {
                return Err(format!(
                    "Chunk {relative_path} contains unlisted combination {combination_id}"
                )
                .into());
            }
            let key = (combo_key, asset_code.to_owned());
            if !seen_values.insert(key) {
                return Err("Duplicate combination/asset value found across chunks".into());
            }
            let count = item_counts_by_combination
                .entry(combination_id.to_owned())
                .or_default();
            if display_codes.get(*count).map(String::as_str) != Some(asset_code) {
                return Err(
                    format!("Combination {combination_id} has unexpected display order").into(),
                );
            }
            *count += 1;
            seen_item_records.insert(
                (combination_id.to_owned(), asset_code.to_owned()),
                item.clone(),
            );
            actual_ids.insert(combination_id);
        }
        if actual_ids != expected_ids || actual_ids.len() != count {
            return Err(format!("Chunk {relative_path} does not match its range").into());
        }
        for combination_id in &expected_ids {
            if !seen_combinations.insert((*combination_id).to_owned()) {
                return Err(
                    format!("Combination {combination_id} is assigned to multiple chunks").into(),
                );
            }
            chunk_by_combination.insert((*combination_id).to_owned(), relative_path.to_owned());
        }
    }

    if next_start_index != product {
        return Err("Chunk ranges do not cover the combination index".into());
    }
    let on_disk_chunk_paths: BTreeSet<_> = fs::read_dir(output_dir.join("items"))?
        .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
        .collect::<Result<_, _>>()?;
    let referenced_chunk_paths: BTreeSet<_> = referenced_chunks
        .iter()
        .map(|path| {
            PathBuf::from(path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    if on_disk_chunk_paths != referenced_chunk_paths {
        return Err("Staged items directory contains missing or unreferenced chunks".into());
    }
    let mut expected_value_count = 0;
    let mut expected_value_keys = BTreeSet::new();
    for (index, combination) in expected_combinations.iter().enumerate() {
        let id = combination["id"]
            .as_str()
            .ok_or("Manifest combination omitted ID")?;
        let status = "published";
        let actual_for_combo = item_counts_by_combination
            .get(id)
            .copied()
            .unwrap_or_default();
        let expected_for_combo = if status == "published" {
            if expected_items[index].len() != display_codes.len() {
                return Err(format!("Combination {id} has an incomplete display series").into());
            }
            display_codes.len()
        } else {
            0
        };
        if actual_for_combo != expected_for_combo {
            return Err(format!(
                "Combination {id} has {actual_for_combo} values; expected {expected_for_combo}"
            )
            .into());
        }
        if status == "published" {
            if !seen_combinations.contains(id) {
                return Err(format!("Published combination {id} has no chunk reference").into());
            }
        }
        for expected_item in &expected_items[index] {
            let asset_code = expected_item["asset_class_code"]
                .as_str()
                .ok_or("Generated item omitted asset_class_code")?;
            expected_value_keys.insert((id.to_owned(), asset_code.to_owned()));
            if seen_item_records.get(&(id.to_owned(), asset_code.to_owned())) != Some(expected_item)
            {
                return Err(format!(
                    "Chunk item for {id}/{asset_code} differs from generated record"
                )
                .into());
            }
        }
        expected_value_count += expected_for_combo;
    }
    let published_count = expected_combinations.len();
    if seen_combinations.len() != published_count {
        return Err("Manifest contains an unreferenced or duplicate combination".into());
    }
    let source_expected_count: usize = expected_items.iter().map(Vec::len).sum();
    if expected_value_count != source_expected_count
        || seen_values.len() != source_expected_count
        || seen_values != expected_value_keys
    {
        return Err("Chunk values are missing or unexpected".into());
    }
    Ok(())
}

fn classify_category(axis: &str, name: &str) -> String {
    if axis != "cat04" && name == "平均" {
        return "total".into();
    }
    if axis == "cat04" && name == "平均" {
        return "total".into();
    }
    if name.starts_with("十分位") {
        return "decile".into();
    }
    if name.starts_with("（再掲）") || name.starts_with("平均（") {
        return "reprint".into();
    }
    "regular".into()
}

/// Resolve the official lowest-class breakdown by label, retaining metadata codes.
fn resolve_display_category_codes(categories: &[Category]) -> Result<Vec<String>, Box<dyn Error>> {
    let mut selected = Vec::new();
    for label in [
        "（再掲）0万円未満",
        "（再掲）0万円",
        "（再掲）0万円超25万円未満",
    ] {
        let matches: Vec<_> = categories
            .iter()
            .filter(|category| category.label == label)
            .collect();
        if matches.len() != 1 || matches[0].kind != "reprint" {
            return Err(format!("Missing or ambiguous official asset category {label}").into());
        }
        selected.push(matches[0].code.clone());
    }
    let lowest: Vec<_> = categories
        .iter()
        .filter(|category| category.label == "25万円未満" && category.kind == "regular")
        .collect();
    if lowest.len() != 1 {
        return Err("Missing or ambiguous lowest regular asset category".into());
    }
    selected.extend(
        categories
            .iter()
            .filter(|category| category.kind == "regular" && category.code != lowest[0].code)
            .map(|category| category.code.clone()),
    );
    if selected.len() != 21 || selected.iter().collect::<BTreeSet<_>>().len() != 21 {
        return Err("Display asset categories must contain 21 unique codes".into());
    }
    Ok(selected)
}

fn make_set(id: &str, label: &str, categories: Vec<&Category>) -> Value {
    serde_json::json!({"id": id, "label": label, "category_codes": categories.iter().map(|c| c.code.as_str()).collect::<Vec<_>>()})
}

fn dimension_json(d: &Dimension) -> Value {
    let mut dimension = serde_json::json!({
        "id": d.id, "label": d.label, "role": d.role, "order": d.order,
        "total_code": d.total_code, "default_code": d.total_code,
        "categories": d.categories.iter().map(|c| serde_json::json!({"code":c.code,"label":c.label,"kind":c.kind,"order":c.order,"api_level":c.api_level,"api_parent_code":c.api_parent_code})).collect::<Vec<_>>(),
        "comparison_sets": d.comparison_sets
    });
    if d.role == "asset_class" {
        dimension["display_category_codes"] = serde_json::json!(d.display_category_codes);
    }
    if d.id == "cat02" {
        dimension["display_hierarchy"] = serde_json::json!({
            "standalone_codes": ["0", "1"],
            "groups": [{"code": "2", "children": ["21", "22", "23"]}]
        });
    }
    dimension
}

fn validate_education_categories(categories: &[Category]) -> Result<(), Box<dyn Error>> {
    let expected = [
        ("0", "平均", "1", None),
        ("1", "在学中", "1", None),
        ("2", "卒業", "1", None),
        ("21", "小学校・中学校", "2", Some("2")),
        ("22", "高校程度", "2", Some("2")),
        ("23", "短大程度以上", "2", Some("2")),
    ];
    if categories.len() != expected.len() {
        return Err(format!("学歴区分の数が変わりました: {}", categories.len()).into());
    }
    for (index, (category, (code, label, level, parent))) in
        categories.iter().zip(expected).enumerate()
    {
        if category.order != index
            || category.code != code
            || category.label != label
            || category.api_level.as_deref() != Some(level)
            || category.api_parent_code.as_deref() != parent
        {
            return Err(format!(
                "想定外の全国家計構造調査学歴区分: {} ({})",
                category.code, category.label
            )
            .into());
        }
    }
    Ok(())
}

fn cartesian_codes(dimensions: &[&Dimension]) -> Vec<Vec<(String, String)>> {
    let mut rows = vec![Vec::new()];
    for dimension in dimensions {
        let mut next = Vec::new();
        for row in &rows {
            for category in &dimension.categories {
                let mut expanded = row.clone();
                expanded.push((dimension.id.clone(), category.code.clone()));
                next.push(expanded);
            }
        }
        rows = next;
    }
    rows
}

fn classify_value(
    raw: &str,
    notes: &HashMap<String, String>,
) -> Result<(Option<u64>, &'static str, Vec<String>), Box<dyn Error>> {
    if raw == "-" {
        return Ok((
            None,
            "missing",
            vec![
                notes
                    .get(raw)
                    .cloned()
                    .unwrap_or_else(|| "該当数字がない".into()),
            ],
        ));
    }
    if raw == "X" {
        return Ok((
            None,
            "suppressed",
            vec![
                notes
                    .get(raw)
                    .cloned()
                    .ok_or("Suppressed value has no API note")?,
            ],
        ));
    }
    let value: u64 = raw
        .parse()
        .map_err(|_| format!("Unrecognized statistical value: {raw}"))?;
    if value == 0 {
        return Ok((None, "missing", vec!["表章単位未満（公式凡例）".into()]));
    }
    Ok((Some(value), "observed", Vec::new()))
}

fn load_notes(root: &PathBuf) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut notes = HashMap::new();
    for entry in fs::read_dir(root.join("raw_data"))? {
        let page: Value = serde_json::from_slice(&fs::read(entry?.path())?)?;
        for note in as_items(&page["GET_STATS_DATA"]["STATISTICAL_DATA"]["DATA_INF"]["NOTE"]) {
            if let (Some(code), Some(label)) = (attr(note, "@char"), attr(note, "$")) {
                notes.insert(code.into(), label.into());
            }
        }
    }
    Ok(notes)
}

fn write_inspection_samples(
    metadata: &Value,
    class_objs: &[&Value],
    raw_paths: &[PathBuf],
    output: &Value,
    measure_code: &str,
    area_code: &str,
    year_code: &str,
) -> Result<(), Box<dyn Error>> {
    let inspection = PathBuf::from("inspections/asset_distribution");
    let samples = inspection.join("samples");
    fs::create_dir_all(&samples)?;
    fs::write(
        samples.join("meta_info.json"),
        serde_json::to_vec_pretty(metadata)?,
    )?;
    let mut chosen = Vec::new();
    let education_codes: BTreeSet<String> = class_objs
        .iter()
        .find(|dimension| attr(dimension, "@id") == Some("cat02"))
        .map(|dimension| {
            as_items(&dimension["CLASS"])
                .into_iter()
                .filter_map(|category| attr(category, "@code").map(str::to_owned))
                .collect()
        })
        .ok_or("Metadata omitted cat02 education classifications")?;
    for path in raw_paths {
        let page: Value = serde_json::from_slice(&fs::read(path)?)?;
        let stat = &page["GET_STATS_DATA"]["STATISTICAL_DATA"];
        for row in as_items(&stat["DATA_INF"]["VALUE"]) {
            if attr(row, "@tab") != Some(measure_code)
                || attr(row, "@area") != Some(area_code)
                || attr(row, "@time") != Some(year_code)
            {
                continue;
            }
            let total = ["cat01", "cat03", "cat05"].iter().all(|id| {
                let Some(dimension) = class_objs
                    .iter()
                    .find(|dimension| attr(dimension, "@id") == Some(id))
                else {
                    return false;
                };
                let Some(category) = as_items(&dimension["CLASS"])
                    .into_iter()
                    .find(|category| attr(category, "@name") == Some("平均"))
                else {
                    return false;
                };
                attr(row, &format!("@{id}")) == attr(category, "@code")
            }) && attr(row, "@cat02")
                .is_some_and(|code| education_codes.contains(code));
            if total {
                chosen.push((*row).clone());
            }
        }
    }
    let sample = serde_json::json!({"sample_note": "Actual e-Stat VALUE records extracted from the complete capture; all six education categories and the 36 asset-class values for average age, average disposable income, and average household size are retained.", "source_total_number": 243432, "selected_value_count": chosen.len(), "GET_STATS_DATA": {"STATISTICAL_DATA": {"DATA_INF": {"NOTE": [{"@char":"-","$":"該当数字がない"},{"@char":"X","$":"集計世帯数が少ないため秘匿しているもの"}], "VALUE": chosen}}}});
    fs::write(
        samples.join("stats_data.json"),
        serde_json::to_vec_pretty(&sample)?,
    )?;
    let digest = |path: &PathBuf| -> Result<String, Box<dyn Error>> {
        let bytes = fs::read(path)?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    };
    let mut manifest = serde_json::json!({
        "captured_at_utc": current_utc_timestamp(), "stats_data_id": STATS_DATA_ID,
        "filters": {"tab":measure_code,"area":area_code,"time":year_code},
        "result_total_number":243432,"pages":raw_paths.len(),"sample_value_count":chosen.len(),
        "samples": {
            "meta_info.json":{"sha256":digest(&samples.join("meta_info.json"))?},
            "stats_data.json":{"sha256":digest(&samples.join("stats_data.json"))?}
        },
        "output_counts": {"combinations":output["combinations"].as_array().map(Vec::len),"items":output["items"].as_array().map(Vec::len)}
    });
    let manifest_path = inspection.join("capture_manifest.json");
    if manifest_path.exists() {
        let previous: Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;
        for key in ["official_crosscheck", "negative_asset_crosscheck"] {
            if let Some(value) = previous.get(key) {
                manifest[key] = value.clone();
            }
        }
    }
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
    Ok(())
}

fn as_items(value: &Value) -> Vec<&Value> {
    match value {
        Value::Array(items) => items.iter().collect(),
        Value::Object(_) => vec![value],
        _ => Vec::new(),
    }
}

fn metadata_category_code(
    class_objs: &[&Value],
    dimension_id: &str,
    label: &str,
) -> Result<String, Box<dyn Error>> {
    let dimension = class_objs
        .iter()
        .find(|dimension| attr(dimension, "@id") == Some(dimension_id))
        .ok_or_else(|| format!("Metadata omitted {dimension_id}"))?;
    let category = as_items(&dimension["CLASS"])
        .into_iter()
        .find(|category| attr(category, "@name") == Some(label))
        .ok_or_else(|| format!("Metadata omitted {label} from {dimension_id}"))?;
    Ok(required_attr(category, "@code")?.to_owned())
}

fn attr<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}
fn required_attr<'a>(value: &'a Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    attr(value, key).ok_or_else(|| format!("e-Stat record is missing {key}").into())
}
fn current_utc_timestamp() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let days = (timestamp / 86_400) as i64;
    let seconds = timestamp % 86_400;
    let z = days + 719_468;
    let era = z / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        seconds / 3_600,
        seconds % 3_600 / 60,
        seconds % 60
    )
}

fn parse_u64(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
}

fn safe_api_error(endpoint: &str, error: &(dyn Error + 'static)) -> Box<dyn Error> {
    let reason = if let Some(error) = error.downcast_ref::<reqwest::Error>() {
        if error.is_timeout() {
            "timed out"
        } else if error.is_connect() {
            "could not connect"
        } else {
            "request failed"
        }
    } else {
        "request failed"
    };
    format!("e-Stat {endpoint} {reason}").into()
}

fn redact_app_id(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for key in ["appId", "@appId"] {
                if object.contains_key(key) {
                    object.insert(key.to_owned(), Value::String("[REDACTED]".to_owned()));
                }
            }
            for child in object.values_mut() {
                redact_app_id(child);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(redact_app_id),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset_categories() -> Vec<Category> {
        let metadata: Value = serde_json::from_str(include_str!(
            "../../../inspections/asset_distribution/samples/meta_info.json"
        ))
        .unwrap();
        let objects =
            as_items(&metadata["GET_META_INFO"]["METADATA_INF"]["CLASS_INF"]["CLASS_OBJ"]);
        let dimension = objects
            .into_iter()
            .find(|dimension| dimension["@id"] == "cat04")
            .unwrap();
        as_items(&dimension["CLASS"])
            .into_iter()
            .enumerate()
            .map(|(order, category)| Category {
                code: category["@code"].as_str().unwrap().to_owned(),
                label: category["@name"].as_str().unwrap().to_owned(),
                kind: classify_category("cat04", category["@name"].as_str().unwrap()),
                order,
                api_level: category
                    .get("@level")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                api_parent_code: category
                    .get("@parentCode")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            })
            .collect()
    }

    #[test]
    fn display_breakdown_replaces_lowest_class_without_double_counting() {
        let categories = asset_categories();
        let actual = resolve_display_category_codes(&categories).unwrap();
        let mut expected = vec!["R01".to_owned(), "R02".to_owned(), "R03".to_owned()];
        expected.extend((2..=19).map(|code| format!("{code:02}")));
        assert_eq!(actual, expected);
        assert!(!actual.contains(&"01".to_owned()));
        assert!(
            categories
                .iter()
                .filter(|category| actual[..3].contains(&category.code))
                .all(|category| category.kind == "reprint")
        );
    }

    #[test]
    fn missing_or_duplicate_official_breakdown_is_an_error() {
        let categories = asset_categories();
        for code in ["R01", "R02", "R03", "01", "02"] {
            let reduced: Vec<_> = categories
                .iter()
                .filter(|category| category.code != code)
                .cloned()
                .collect();
            assert!(resolve_display_category_codes(&reduced).is_err(), "{code}");
        }
        let mut duplicated = categories.clone();
        duplicated.push(
            categories
                .iter()
                .find(|category| category.code == "R01")
                .unwrap()
                .clone(),
        );
        assert!(resolve_display_category_codes(&duplicated).is_err());
    }

    #[test]
    fn official_non_numeric_values_are_not_filled_with_zero() {
        let notes = HashMap::from([("X".to_owned(), "秘匿".to_owned())]);
        for (raw, status) in [("X", "suppressed"), ("-", "missing"), ("0", "missing")] {
            let (count, actual_status, annotations) = classify_value(raw, &notes).unwrap();
            assert_eq!(count, None);
            assert_eq!(actual_status, status);
            assert!(!annotations.is_empty());
        }
        assert_eq!(
            classify_value("3973273", &notes).unwrap().0,
            Some(3_973_273)
        );
    }

    #[test]
    fn display_codes_are_serialized_only_for_asset_dimension() {
        let categories = asset_categories();
        let display_category_codes = resolve_display_category_codes(&categories).unwrap();
        let asset_dimension = Dimension {
            id: "cat04".into(),
            label: "資産額階級36区分".into(),
            role: "asset_class".into(),
            order: 3,
            total_code: None,
            categories,
            comparison_sets: Vec::new(),
            display_category_codes,
        };
        assert_eq!(
            dimension_json(&asset_dimension)["display_category_codes"],
            serde_json::json!(asset_dimension.display_category_codes)
        );
        let filter_dimension = Dimension {
            id: "cat01".into(),
            label: "年齢".into(),
            role: "filter".into(),
            order: 0,
            total_code: None,
            categories: Vec::new(),
            comparison_sets: Vec::new(),
            display_category_codes: Vec::new(),
        };
        assert!(
            dimension_json(&filter_dimension)
                .get("display_category_codes")
                .is_none()
        );
    }
    #[test]
    fn bundle_validator_rejects_old_schemas_missing_contract_and_bad_series() {
        let categories = asset_categories();
        let codes = resolve_display_category_codes(&categories).unwrap();
        let directory =
            std::env::temp_dir().join(format!("asset-distribution-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(directory.join("items")).unwrap();
        let series: Vec<_> = codes.iter().map(|code| serde_json::json!({"combination_id":"cat01=00", "asset_class_code":code, "count":1,"raw_value":"1","status":"observed","annotations":[]})).collect();
        let bytes = chunk_json_bytes(&series).unwrap();
        let expected_combinations =
            vec![serde_json::json!({"id":"cat01=00","dimensions":{"cat01":"00"}})];
        let manifest = serde_json::json!({"schema_version":3,"dimensions":[{
            "id":"cat04", "categories":categories.iter().map(|category| serde_json::json!({"code":category.code,"label":category.label,"kind":category.kind})).collect::<Vec<_>>(),
            "display_category_codes":codes
        }, {"id":"cat01", "role":"filter", "categories":[{"code":"00"}]}],
        "combination_index":{"dimension_ids":["cat01"],"count":1},
        "chunks":[{"path":"items/00000.json","bytes":bytes.len(),"start_index":0,"count":1}]});
        let check = |candidate: &Value, items: &[Value], expected: &[Vec<Value>]| {
            let chunk = chunk_json_bytes(items).unwrap();
            let mut candidate = candidate.clone();
            candidate["chunks"][0]["bytes"] = serde_json::json!(chunk.len());
            fs::write(
                directory.join("manifest.json"),
                serde_json::to_vec(&candidate).unwrap(),
            )
            .unwrap();
            fs::write(directory.join("items/00000.json"), chunk).unwrap();
            validate_distribution_bundle(&directory, &candidate, &expected_combinations, expected)
        };
        assert!(check(&manifest, &series, &[series.clone()]).is_ok());
        for version in [1, 2, 4] {
            let mut bad = manifest.clone();
            bad["schema_version"] = serde_json::json!(version);
            assert!(check(&bad, &series, &[series.clone()]).is_err());
        }
        for invalid_axes in [
            serde_json::json!(["cat01", "cat01"]),
            serde_json::json!(["cat00"]),
            serde_json::json!([]),
            serde_json::json!(["cat01", "cat04"]),
        ] {
            let mut bad = manifest.clone();
            bad["combination_index"]["dimension_ids"] = invalid_axes;
            assert!(check(&bad, &series, &[series.clone()]).is_err());
        }
        for (start, count) in [(1, 1), (0, 2), (0, 0)] {
            let mut bad = manifest.clone();
            bad["chunks"][0]["start_index"] = serde_json::json!(start);
            bad["chunks"][0]["count"] = serde_json::json!(count);
            assert!(check(&bad, &series, &[series.clone()]).is_err());
        }
        let mut bad = manifest.clone();
        bad["combination_index"]["count"] = serde_json::json!(2);
        assert!(check(&bad, &series, &[series.clone()]).is_err());
        let mut bad = manifest.clone();
        bad["dimensions"][0]
            .as_object_mut()
            .unwrap()
            .remove("display_category_codes");
        assert!(check(&bad, &series, &[series.clone()]).is_err());
        for invalid_codes in [
            vec!["R01"],
            vec!["R01", "R01"],
            vec!["01", "R01"],
            vec!["unknown"],
        ] {
            let mut bad = manifest.clone();
            bad["dimensions"][0]["display_category_codes"] = serde_json::json!(invalid_codes);
            assert!(check(&bad, &series, &[series.clone()]).is_err());
        }
        assert!(check(&manifest, &series[1..], &[series.clone()]).is_err());
        let mut duplicated = series.clone();
        duplicated[1] = duplicated[0].clone();
        assert!(check(&manifest, &duplicated, &[series.clone()]).is_err());
        let mut reordered = series.clone();
        reordered.swap(0, 1);
        assert!(check(&manifest, &reordered, &[series.clone()]).is_err());
        let mut old_chunk: Value = serde_json::from_slice(&bytes).unwrap();
        old_chunk["schema_version"] = serde_json::json!(1);
        let old_bytes = serde_json::to_vec(&old_chunk).unwrap();
        let mut bad = manifest.clone();
        bad["chunks"][0]["bytes"] = serde_json::json!(old_bytes.len());
        fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec(&bad).unwrap(),
        )
        .unwrap();
        fs::write(directory.join("items/00000.json"), old_bytes).unwrap();
        assert!(
            validate_distribution_bundle(&directory, &bad, &expected_combinations, &[series])
                .is_err()
        );
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn national_breakdown_matches_official_values_without_rounding_adjustment() {
        let sample: Value = serde_json::from_str(include_str!(
            "../../../inspections/asset_distribution/samples/stats_data.json"
        ))
        .unwrap();
        let rows = as_items(&sample["GET_STATS_DATA"]["STATISTICAL_DATA"]["DATA_INF"]["VALUE"]);
        let national: HashMap<_, _> = rows
            .iter()
            .filter(|row| {
                row["@cat01"] == "00"
                    && row["@cat02"] == "0"
                    && row["@cat03"] == "00"
                    && row["@cat05"] == "0"
            })
            .map(|row| (row["@cat04"].as_str().unwrap(), row["$"].as_str().unwrap()))
            .collect();
        let expected = [("R01", 3_973_273), ("R02", 1_520_744), ("R03", 1_039_774)];
        let mut sum = 0;
        for (code, value) in expected {
            let (count, status, _) = classify_value(national[code], &HashMap::new()).unwrap();
            assert_eq!(count, Some(value));
            assert_eq!(status, "observed");
            sum += value;
        }
        assert_eq!(sum, 6_533_791);
        assert_eq!(national["01"], "6533792");
    }
}
