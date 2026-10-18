use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    error::Error,
    fs,
    path::PathBuf,
    thread,
    time::Duration,
};

use serde_json::Value;

use crate::modules::{
    estat_api::{
        get_meta_info::{
            GetMetaInfoParamsBuilder, MandatoryParams as MetaMandatoryParams, get_meta_info,
        },
        get_stats_data::{
            GetStatsDataParamsBuilder, MandatoryParams as DataMandatoryParams, get_stats_data,
        },
    },
    projects::workers_by_age_employment_type_education_sex::{
        OUTPUT_DIR_META, OUTPUT_DIR_RAW, OUTPUT_DIR_TRANSFORMED, STATS_DATA_ID,
    },
};

const PAGE_SIZE: u32 = 100_000;

/// Stable frontend contract candidate. The finalized field names and dimension
/// labels are to be confirmed against the captured e-Stat response.
#[allow(dead_code)]
#[derive(Debug, serde::Serialize)]
pub struct OptionsDocument {
    pub metadata: OptionsMetadata,
    pub dimensions: DimensionOptions,
    pub education_display_hierarchy: Vec<DisplayNode>,
    pub education_comparison_sets: Vec<ComparisonSet>,
    pub series: Vec<SeriesReference>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Serialize)]
pub struct OptionsMetadata {
    pub stats_data_id: String,
    pub title: String,
    pub statistics_name: String,
    pub unit: String,
    pub period: String,
    pub fetched_at: String,
}

#[allow(dead_code)]
#[derive(Debug, serde::Serialize)]
pub struct DimensionOptions {
    pub age: Vec<DimensionOption>,
    pub employment_type: Vec<DimensionOption>,
    pub education: Vec<DimensionOption>,
    pub sex: Vec<DimensionOption>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Serialize)]
pub struct DimensionOption {
    pub code: String,
    pub name: String,
    pub order: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_parent_code: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct DisplayNode {
    pub code: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<DisplayNode>,
}

#[derive(Debug, serde::Serialize)]
pub struct ComparisonSet {
    pub id: String,
    pub label: String,
    pub category_codes: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Serialize)]
pub struct SeriesReference {
    pub age_code: String,
    pub employment_type_code: String,
    pub education_code: String,
    pub sex_code: String,
    pub path: String,
}

#[derive(Debug, serde::Serialize)]
struct SeriesDocument {
    age_code: String,
    age_name: String,
    employment_type_code: String,
    employment_type_name: String,
    education_code: String,
    education_name: String,
    sex_code: String,
    sex_name: String,
    unit: String,
    time_value_pairs: Vec<YearValue>,
}

#[derive(Debug, serde::Serialize)]
struct YearValue {
    year: String,
    value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

struct NormalizedRecord {
    codes: (String, String, String, String),
    time_code: String,
    unit: String,
    year_value: YearValue,
}

fn safe_api_error(endpoint: &str, error: &(dyn Error + 'static)) -> String {
    if let Some(error) = error.downcast_ref::<reqwest::Error>() {
        let reason = if error.is_timeout() {
            "timed out"
        } else if error.is_connect() {
            "could not connect"
        } else if error.is_body() {
            "response body could not be read"
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

/// Fetches and preserves the complete metadata and paged data response. This
/// acquisition stage is useful on its own and forms the input to normalization.
/// Each acquisition replaces the prior capture. A failed request may leave an
/// incomplete capture, which must be fetched again before transforming.
pub fn acquire(app_id: &str) -> Result<(), Box<dyn Error>> {
    let raw_dir = PathBuf::from(OUTPUT_DIR_RAW);
    let meta_dir = PathBuf::from(OUTPUT_DIR_META);
    if raw_dir.exists() {
        fs::remove_dir_all(&raw_dir)?;
    }
    if meta_dir.exists() {
        fs::remove_dir_all(&meta_dir)?;
    }
    fs::create_dir_all(&raw_dir)?;
    fs::create_dir_all(&meta_dir)?;
    let acquired_at = current_utc_timestamp()?;

    let metadata = get_meta_info(
        GetMetaInfoParamsBuilder::new(MetaMandatoryParams {
            app_id,
            stats_data_id: STATS_DATA_ID,
        })
        .lang("J")
        .build(),
    )
    .map_err(|error| safe_api_error("metadata request", error.as_ref()))?;
    let mut metadata: Value = serde_json::from_str(&metadata)?;
    redact_app_id(&mut metadata);
    fs::write(
        meta_dir.join("get_meta_info.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    fs::write(
        meta_dir.join("acquisition.json"),
        serde_json::json!({"fetched_at": acquired_at}).to_string(),
    )?;

    let mut start_position = 1_u32;
    let mut total = None;
    loop {
        if start_position > 1 {
            thread::sleep(Duration::from_secs(2));
        }
        let response = get_stats_data(
            GetStatsDataParamsBuilder::new(DataMandatoryParams { app_id })
                .stats_data_id(STATS_DATA_ID)
                .start_position(start_position)
                .limit(PAGE_SIZE)
                .build(),
        )
        .map_err(|error| {
            format!(
                "{} at position {start_position}",
                safe_api_error("data request", error.as_ref())
            )
        })?;
        let mut parsed: Value = serde_json::from_str(&response)?;
        let statistical_data = &parsed["GET_STATS_DATA"]["STATISTICAL_DATA"];
        let page_info = &statistical_data["RESULT_INF"];
        let result = &page_info["TOTAL_NUMBER"];
        let page_total = result
            .as_str()
            .and_then(|value| value.parse::<u64>().ok())
            .or_else(|| result.as_u64())
            .ok_or("e-Stat response omitted a valid RESULT_INF.TOTAL_NUMBER")?;
        if total.is_some_and(|expected| expected != page_total) {
            return Err("e-Stat changed TOTAL_NUMBER while paging".into());
        }
        total = Some(page_total);

        let values = &statistical_data["DATA_INF"]["VALUE"];
        let page_count = match values {
            Value::Array(items) => items.len() as u64,
            Value::Object(_) => 1,
            Value::Null => 0,
            Value::String(value) if value.is_empty() => 0,
            _ => return Err("Unexpected DATA_INF.VALUE shape".into()),
        };
        if page_count > 0 {
            let from = parse_u64(&page_info["FROM_NUMBER"])
                .ok_or("e-Stat response omitted RESULT_INF.FROM_NUMBER")?;
            let to = parse_u64(&page_info["TO_NUMBER"])
                .ok_or("e-Stat response omitted RESULT_INF.TO_NUMBER")?;
            if from != u64::from(start_position) || to != u64::from(start_position) + page_count - 1
            {
                return Err(format!("Unexpected e-Stat page boundary at {start_position}").into());
            }
        } else if page_total > 0 {
            return Err(format!("e-Stat returned an empty page at {start_position}").into());
        }
        let page_path = raw_dir.join(format!("get_stats_data_{start_position:010}.json"));
        redact_app_id(&mut parsed);
        fs::write(page_path, serde_json::to_vec_pretty(&parsed)?)?;
        println!("Fetched records beginning at {start_position}: {page_count}");

        if page_count == 0 || u64::from(start_position) + page_count > page_total {
            break;
        }
        start_position = start_position
            .checked_add(page_count as u32)
            .ok_or("e-Stat page position overflow")?;
    }

    println!("Acquisition pages saved; normalizing captured data");
    transform()
}

fn current_utc_timestamp() -> Result<String, Box<dyn Error>> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    Ok(format_utc_timestamp(timestamp))
}

fn parse_u64(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|text| text.parse::<u64>().ok()))
}

/// Convert the captured metadata and all raw pages into the browser-facing
/// options index and one time series per observed four-dimension combination.
pub fn transform() -> Result<(), Box<dyn Error>> {
    let metadata: Value = serde_json::from_slice(&fs::read(
        PathBuf::from(OUTPUT_DIR_META).join("get_meta_info.json"),
    )?)?;
    let meta_root = &metadata["GET_META_INFO"]["METADATA_INF"];
    let table = &meta_root["TABLE_INF"];
    let class_objs = meta_root["CLASS_INF"]["CLASS_OBJ"]
        .as_array()
        .ok_or("Metadata CLASS_INF.CLASS_OBJ was not an array")?;

    let age = dimension_options(class_objs, "cat01")?;
    let employment_type = dimension_options(class_objs, "cat03")?;
    let education = dimension_options(class_objs, "cat04")?;
    validate_education_categories(&education)?;
    let sex = dimension_options(class_objs, "cat05")?;
    let lookups = [
        (&age, "age"),
        (&employment_type, "employment_type"),
        (&education, "education"),
        (&sex, "sex"),
    ];
    let label_maps: Vec<HashMap<String, String>> = lookups
        .iter()
        .map(|(options, _)| {
            options
                .iter()
                .map(|o| (o.code.clone(), o.name.clone()))
                .collect()
        })
        .collect();

    let mut groups: BTreeMap<(String, String, String, String), Vec<YearValue>> = BTreeMap::new();
    let mut notes: HashMap<String, String> = HashMap::new();
    let mut unit = String::new();
    let mut expected_total = None;
    let mut seen_records = BTreeSet::new();

    let mut raw_paths: Vec<PathBuf> = fs::read_dir(OUTPUT_DIR_RAW)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    raw_paths.sort();
    if raw_paths.is_empty() {
        return Err("No acquired e-Stat pages are available to transform".into());
    }
    for path in raw_paths {
        let page: Value = serde_json::from_slice(&fs::read(&path)?)?;
        let statistical_data = &page["GET_STATS_DATA"]["STATISTICAL_DATA"];
        if let Some(page_total) = parse_u64(&statistical_data["RESULT_INF"]["TOTAL_NUMBER"]) {
            if expected_total.is_some_and(|expected| expected != page_total as usize) {
                return Err("Raw pages disagree on TOTAL_NUMBER".into());
            }
            expected_total = usize::try_from(page_total).ok();
        }
        for note in as_items(&statistical_data["DATA_INF"]["NOTE"]) {
            if let (Some(char), Some(description)) = (attr(note, "@char"), attr(note, "$")) {
                notes.insert(char.to_owned(), description.to_owned());
            }
        }
        for item in as_items(&statistical_data["DATA_INF"]["VALUE"]) {
            if attr(item, "@tab") != Some("06")
                || attr(item, "@area") != Some("00000")
                || attr(item, "@cat02") != Some("02")
            {
                continue;
            }
            let record = normalize_record(item, &notes)?;
            let value_key = (
                record.codes.0.clone(),
                record.codes.1.clone(),
                record.codes.2.clone(),
                record.codes.3.clone(),
                record.time_code.clone(),
            );
            if !seen_records.insert(value_key) {
                return Err(format!("Duplicate source record in {}", path.display()).into());
            }
            let record_unit = record.unit;
            if unit.is_empty() && !record_unit.is_empty() {
                unit = record_unit.clone();
            } else if !record_unit.is_empty() && unit != record_unit {
                return Err("The selected table contains multiple units".into());
            }
            groups
                .entry(record.codes)
                .or_default()
                .push(record.year_value);
        }
    }
    if let Some(expected) = expected_total {
        if seen_records.len() != expected {
            return Err(format!(
                "Fetched {} source rows but e-Stat reports {expected}",
                seen_records.len()
            )
            .into());
        }
    }

    let transformed_target = PathBuf::from(OUTPUT_DIR_TRANSFORMED);
    if transformed_target.exists() {
        fs::remove_dir_all(&transformed_target)?;
    }
    fs::create_dir_all(&transformed_target)?;
    let options_target = PathBuf::from(OUTPUT_DIR_META).join("options.json");
    if options_target.exists() {
        fs::remove_file(&options_target)?;
    }
    let mut series = Vec::with_capacity(groups.len());
    for ((age_code, employment_type_code, education_code, sex_code), mut values) in groups {
        values.sort_by(|left, right| left.year.cmp(&right.year));
        let path = format!(
            "age-{age_code}_employment-{employment_type_code}_education-{education_code}_sex-{sex_code}.json"
        );
        let document = SeriesDocument {
            age_name: lookup_label(&label_maps[0], &age_code)?,
            employment_type_name: lookup_label(&label_maps[1], &employment_type_code)?,
            education_name: lookup_label(&label_maps[2], &education_code)?,
            sex_name: lookup_label(&label_maps[3], &sex_code)?,
            age_code: age_code.clone(),
            employment_type_code: employment_type_code.clone(),
            education_code: education_code.clone(),
            sex_code: sex_code.clone(),
            unit: unit.clone(),
            time_value_pairs: values,
        };
        fs::write(
            transformed_target.join(&path),
            serde_json::to_vec_pretty(&document)?,
        )?;
        series.push(SeriesReference {
            age_code,
            employment_type_code,
            education_code,
            sex_code,
            path,
        });
    }

    let title = table["TITLE"]["$"]
        .as_str()
        .unwrap_or("年齢階級，教育，雇用形態別雇用者数")
        .to_owned();
    let statistics_name = table["STAT_NAME"]["$"].as_str().unwrap_or("労働力調査");
    let year_names = class_objs
        .iter()
        .find(|item| attr(item, "@id") == Some("time"));
    let period = year_names
        .and_then(|item| item.get("CLASS"))
        .map(as_items)
        .and_then(|items| {
            let first = items.first().and_then(|item| attr(item, "@name"))?;
            let last = items.last().and_then(|item| attr(item, "@name"))?;
            Some(format!("{}～{}", first.trim_end_matches('年'), last))
        })
        .unwrap_or_else(|| "2002年～".to_owned());
    let fetched_at = fs::read(PathBuf::from(OUTPUT_DIR_META).join("acquisition.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .and_then(|v| v["fetched_at"].as_str().map(str::to_owned))
        .or_else(|| {
            fs::read(PathBuf::from(OUTPUT_DIR_META).join("options.json"))
                .ok()
                .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                .and_then(|v| v["metadata"]["fetched_at"].as_str().map(str::to_owned))
        })
        .unwrap_or(current_utc_timestamp()?);
    let options = OptionsDocument {
        metadata: OptionsMetadata {
            stats_data_id: STATS_DATA_ID.to_owned(),
            title,
            statistics_name: statistics_name.to_owned(),
            unit: if unit.is_empty() {
                "万人".to_owned()
            } else {
                unit
            },
            period,
            fetched_at,
        },
        dimensions: DimensionOptions {
            age,
            employment_type,
            education,
            sex,
        },
        education_display_hierarchy: education_display_hierarchy(),
        education_comparison_sets: education_comparison_sets(),
        series,
    };
    for item in &options.series {
        if !transformed_target.join(&item.path).is_file() {
            return Err(format!("Series index points to a missing file: {}", item.path).into());
        }
    }
    let options_bytes = serde_json::to_vec_pretty(&options)?;
    fs::write(&options_target, options_bytes)?;
    Ok(())
}

fn format_utc_timestamp(timestamp: u64) -> String {
    let days = (timestamp / 86_400) as i64;
    let seconds = timestamp % 86_400;
    // Gregorian civil date from days since 1970-01-01.
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
    let hour = seconds / 3_600;
    let minute = seconds % 3_600 / 60;
    let second = seconds % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    const META_SAMPLE: &str = include_str!(
        "../../../../inspections/workers_by_age_employment_type_education_sex/samples/meta_info.json"
    );
    const DATA_SAMPLE: &str = include_str!(
        "../../../../inspections/workers_by_age_employment_type_education_sex/samples/stats_data_start_1_limit_3.json"
    );

    fn sample_notes_and_values() -> (HashMap<String, String>, Vec<Value>) {
        let data: Value = serde_json::from_str(DATA_SAMPLE).unwrap();
        let statistical_data = &data["GET_STATS_DATA"]["STATISTICAL_DATA"]["DATA_INF"];
        let notes = as_items(&statistical_data["NOTE"])
            .into_iter()
            .map(|note| {
                (
                    attr(note, "@char").unwrap().to_owned(),
                    attr(note, "$").unwrap().to_owned(),
                )
            })
            .collect();
        let values = as_items(&statistical_data["VALUE"])
            .into_iter()
            .cloned()
            .collect();
        (notes, values)
    }

    #[test]
    fn captured_metadata_produces_classification_labels() {
        let metadata: Value = serde_json::from_str(META_SAMPLE).unwrap();
        let class_objs = metadata["GET_META_INFO"]["METADATA_INF"]["CLASS_INF"]["CLASS_OBJ"]
            .as_array()
            .unwrap();
        let age = dimension_options(class_objs, "cat01").unwrap();
        let employment = dimension_options(class_objs, "cat03").unwrap();
        let education = dimension_options(class_objs, "cat04").unwrap();
        let sex = dimension_options(class_objs, "cat05").unwrap();

        assert_eq!((age[0].code.as_str(), age[0].name.as_str()), ("00", "総数"));
        assert_eq!(
            employment
                .iter()
                .find(|option| option.code == "03")
                .unwrap()
                .name,
            "正規の職員・従業員"
        );
        assert_eq!(
            education
                .iter()
                .find(|option| option.code == "08")
                .unwrap()
                .name,
            "大学・大学院"
        );
        assert_eq!(
            sex.iter().find(|option| option.code == "2").unwrap().name,
            "女"
        );
    }

    #[test]
    fn captured_values_keep_codes_year_value_and_unit() {
        let (notes, values) = sample_notes_and_values();
        let record = normalize_record(&values[0], &notes).unwrap();

        assert_eq!(
            record.codes,
            ("00".into(), "00".into(), "00".into(), "0".into())
        );
        assert_eq!(record.time_code, "2002000000");
        assert_eq!(record.unit, "万人");
        assert_eq!(record.year_value.year, "2002");
        assert_eq!(record.year_value.value, Some(5337.0));
        assert_eq!(record.year_value.reason, None);
    }

    #[test]
    fn below_unit_note_is_preserved_as_missing_not_zero() {
        let (notes, mut values) = sample_notes_and_values();
        values[0]["$"] = Value::String("0".to_owned());
        let record = normalize_record(&values[0], &notes).unwrap();

        assert_eq!(record.year_value.value, None);
        assert_eq!(
            record.year_value.reason.as_deref(),
            notes.get("0").map(String::as_str)
        );
    }

    #[test]
    fn fetched_at_uses_iso_utc_format() {
        assert_eq!(format_utc_timestamp(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_utc_timestamp(1_767_225_600), "2026-01-01T00:00:00Z");
    }
}

fn dimension_options(
    class_objs: &[Value],
    id: &str,
) -> Result<Vec<DimensionOption>, Box<dyn Error>> {
    let dimension = class_objs
        .iter()
        .find(|item| attr(item, "@id") == Some(id))
        .ok_or_else(|| format!("Missing dimension {id} in e-Stat metadata"))?;
    let options = as_items(&dimension["CLASS"]);
    options
        .iter()
        .enumerate()
        .map(|(order, item)| {
            Ok(DimensionOption {
                code: required_attr(item, "@code")?.to_owned(),
                name: required_attr(item, "@name")?.to_owned(),
                order,
                api_level: attr(item, "@level").map(str::to_owned),
                api_parent_code: attr(item, "@parentCode").map(str::to_owned),
            })
        })
        .collect()
}

fn validate_education_categories(categories: &[DimensionOption]) -> Result<(), Box<dyn Error>> {
    let expected = [
        ("00", "総数", "1", None),
        ("01", "うち在学中", "2", None),
        ("02", "小学・中学・高校", "3", None),
        ("03", "短大・高専", "3", Some("02")),
        ("04", "大学・大学院", "3", Some("02")),
        ("05", "うち卒業", "2", None),
        ("06", "小学・中学・高校・旧中", "3", Some("05")),
        ("07", "短大・高専", "3", Some("05")),
        ("08", "大学・大学院", "3", Some("05")),
        ("09", "大学", "3", Some("05")),
        ("10", "大学院", "3", Some("05")),
    ];
    if categories.len() != expected.len() {
        return Err(format!("教育区分の数が変わりました: {}", categories.len()).into());
    }
    for (index, (category, (code, name, level, parent))) in
        categories.iter().zip(expected).enumerate()
    {
        if category.order != index
            || category.code != code
            || category.name != name
            || category.api_level.as_deref() != Some(level)
            || category.api_parent_code.as_deref() != parent
        {
            return Err(format!(
                "想定外の労働力調査教育区分: {} ({})",
                category.code, category.name
            )
            .into());
        }
    }
    Ok(())
}

fn education_display_hierarchy() -> Vec<DisplayNode> {
    let leaf = |code: &str| DisplayNode {
        code: code.to_owned(),
        children: Vec::new(),
    };
    vec![DisplayNode {
        code: "00".into(),
        children: vec![
            DisplayNode {
                code: "01".into(),
                children: vec![leaf("02"), leaf("03"), leaf("04")],
            },
            DisplayNode {
                code: "05".into(),
                children: vec![leaf("06"), leaf("07"), leaf("08"), leaf("09"), leaf("10")],
            },
        ],
    }]
}

fn education_comparison_sets() -> Vec<ComparisonSet> {
    vec![
        ComparisonSet {
            id: "education-status".into(),
            label: "在学中・卒業".into(),
            category_codes: vec!["01".into(), "05".into()],
        },
        ComparisonSet {
            id: "enrolled-categories".into(),
            label: "在学中の教育区分".into(),
            category_codes: vec!["02".into(), "03".into(), "04".into()],
        },
        ComparisonSet {
            id: "graduated-categories".into(),
            label: "卒業者の教育区分".into(),
            category_codes: vec![
                "06".into(),
                "07".into(),
                "08".into(),
                "09".into(),
                "10".into(),
            ],
        },
    ]
}

fn normalize_record(
    item: &Value,
    notes: &HashMap<String, String>,
) -> Result<NormalizedRecord, Box<dyn Error>> {
    let time_code = required_attr(item, "@time")?;
    if !time_code.is_ascii()
        || time_code.len() != 10
        || !time_code[4..].bytes().all(|byte| byte == b'0')
    {
        return Err(format!("Unexpected non-annual time code {time_code}").into());
    }
    let raw_value = required_attr(item, "$")?;
    // e-Stat NOTE characters take precedence over numeric parsing: in this
    // table `0` marks a value below the display unit, not zero.
    let note_reason = notes.get(raw_value).cloned();
    let numeric = if note_reason.is_some() {
        None
    } else {
        raw_value.parse::<f64>().ok()
    };
    let reason = if numeric.is_some() {
        None
    } else {
        Some(note_reason.unwrap_or_else(|| raw_value.to_owned()))
    };
    Ok(NormalizedRecord {
        codes: (
            required_attr(item, "@cat01")?.to_owned(),
            required_attr(item, "@cat03")?.to_owned(),
            required_attr(item, "@cat04")?.to_owned(),
            required_attr(item, "@cat05")?.to_owned(),
        ),
        time_code: time_code.to_owned(),
        unit: attr(item, "@unit").unwrap_or("").to_owned(),
        year_value: YearValue {
            year: time_code[..4].to_owned(),
            value: numeric,
            reason,
        },
    })
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
    attr(value, key).ok_or_else(|| format!("e-Stat record is missing {key}").into())
}

fn lookup_label(labels: &HashMap<String, String>, code: &str) -> Result<String, Box<dyn Error>> {
    labels
        .get(code)
        .cloned()
        .ok_or_else(|| format!("Unknown classification code {code}").into())
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
        Value::Array(items) => {
            for child in items {
                redact_app_id(child);
            }
        }
        _ => {}
    }
}
