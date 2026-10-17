//! Acquisition entry point for e-Stat table 0004056120.
//!
//! The table's classifications must be derived from the captured metadata. In
//! particular, no classification codes are duplicated in this module.

use std::{
    collections::{BTreeMap, HashMap},
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
    let staging = PathBuf::from(format!("{OUTPUT_DIR}.staging"));
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(staging.join("meta_data"))?;
    fs::create_dir_all(staging.join("raw_data"))?;

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
        staging.join("meta_data/get_meta_info.json"),
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
            staging
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
        staging.join("capture_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;

    let target = PathBuf::from(OUTPUT_DIR);
    let backup = PathBuf::from(format!("{OUTPUT_DIR}.previous"));
    let _ = fs::remove_dir_all(&backup);
    if target.exists() {
        fs::rename(&target, &backup)?;
    }
    if let Err(error) = fs::rename(&staging, &target) {
        if backup.exists() {
            let _ = fs::rename(&backup, &target);
        }
        return Err(error.into());
    }
    let _ = fs::remove_dir_all(backup);
    println!("Captured {captured} records in {OUTPUT_DIR}");
    Ok(())
}

#[derive(Clone)]
struct Category {
    code: String,
    label: String,
    kind: String,
    order: usize,
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
            });
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
                    "在学中・卒業",
                    categories
                        .iter()
                        .filter(|c| c.label == "在学中" || c.label == "卒業")
                        .collect(),
                ),
                make_set(
                    "graduated-subgroups",
                    "卒業者の学歴",
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
        dimensions.push(Dimension {
            id: (*id).to_owned(),
            label,
            role: role.to_owned(),
            order,
            total_code,
            categories,
            comparison_sets,
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
    let regular_asset: Vec<_> = asset_dimension
        .categories
        .iter()
        .filter(|c| c.kind == "regular")
        .collect();
    let expected_combinations = attribute_dims.iter().try_fold(1_usize, |total, d| {
        total
            .checked_mul(d.categories.len())
            .ok_or("Combination count overflow")
    })?;
    let mut combinations = Vec::with_capacity(expected_combinations);
    let mut items = Vec::with_capacity(expected_combinations * regular_asset.len());
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
            if asset.kind != "regular" {
                continue;
            }
            let (count, status, annotations) = classify_value(&raw_value, &notes)?;
            items.push(serde_json::json!({"combination_id": combination_id, "asset_class_code": asset.code, "count": count, "status": status, "raw_value": raw_value, "annotations": annotations}));
        }
    }
    if !records.is_empty() {
        return Err(format!(
            "{} unexpected or non-regular source rows were not classified",
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
    let output = serde_json::json!({"metadata": metadata_out, "dimensions": dimensions.iter().map(dimension_json).collect::<Vec<_>>(), "combinations": combinations, "items": items});
    let transformed_path = root.join("asset_distribution.json");
    fs::write(&transformed_path, serde_json::to_vec_pretty(&output)?)?;
    write_inspection_samples(
        &metadata,
        &class_objs,
        &raw_paths,
        &output,
        &measure_code,
        &area_code,
        &year_code,
    )?;
    println!(
        "Transformed {} published combinations and {} regular-class values to {}",
        expected_combinations,
        expected_combinations * regular_asset.len(),
        transformed_path.display()
    );
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

fn make_set(id: &str, label: &str, categories: Vec<&Category>) -> Value {
    serde_json::json!({"id": id, "label": label, "category_codes": categories.iter().map(|c| c.code.as_str()).collect::<Vec<_>>()})
}

fn dimension_json(d: &Dimension) -> Value {
    serde_json::json!({
        "id": d.id, "label": d.label, "role": d.role, "order": d.order,
        "total_code": d.total_code, "default_code": d.total_code,
        "categories": d.categories.iter().map(|c| serde_json::json!({"code":c.code,"label":c.label,"kind":c.kind,"order":c.order})).collect::<Vec<_>>(),
        "comparison_sets": d.comparison_sets
    })
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
    let age_code = metadata_category_code(class_objs, "cat01", "35歳未満")?;
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
            let total = ["cat01", "cat02", "cat03", "cat05"].iter().all(|id| {
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
            });
            let age_subgroup = attr(row, "@cat01") == Some(age_code.as_str())
                && ["cat02", "cat03", "cat05"].iter().all(|id| {
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
                });
            if total || age_subgroup {
                chosen.push((*row).clone());
            }
        }
    }
    let sample = serde_json::json!({"sample_note": "Actual e-Stat VALUE records extracted from the complete capture; two full 36-category series (overall and age-under-35) are retained.", "source_total_number": 243432, "selected_value_count": chosen.len(), "GET_STATS_DATA": {"STATISTICAL_DATA": {"DATA_INF": {"NOTE": [{"@char":"-","$":"該当数字がない"},{"@char":"X","$":"集計世帯数が少ないため秘匿しているもの"}], "VALUE": chosen}}}});
    fs::write(
        samples.join("stats_data.json"),
        serde_json::to_vec_pretty(&sample)?,
    )?;
    let digest = |path: &PathBuf| -> Result<String, Box<dyn Error>> {
        let bytes = fs::read(path)?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    };
    let manifest = serde_json::json!({
        "captured_at_utc": current_utc_timestamp(), "stats_data_id": STATS_DATA_ID,
        "filters": {"tab":measure_code,"area":area_code,"time":year_code},
        "result_total_number":243432,"pages":raw_paths.len(),"sample_value_count":chosen.len(),
        "samples": {
            "meta_info.json":{"sha256":digest(&samples.join("meta_info.json"))?},
            "stats_data.json":{"sha256":digest(&samples.join("stats_data.json"))?}
        },
        "output_counts": {"combinations":output["combinations"].as_array().map(Vec::len),"items":output["items"].as_array().map(Vec::len)}
    });
    fs::write(
        inspection.join("capture_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
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
