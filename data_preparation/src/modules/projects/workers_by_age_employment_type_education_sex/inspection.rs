//! Reproducible e-Stat API inspection captures for table 0003006608.

use reqwest::blocking::Client;
use serde_json::{Map, Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    error::Error,
    fs,
    path::Path,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const TABLE: &str = "0003006608";
const BASE: &str = "https://api.e-stat.go.jp/rest/3.0/app/json";
const ROOT: &str = "inspections/workers_by_age_employment_type_education_sex";

fn timestamp() -> Result<String, Box<dyn Error>> {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let days = (seconds / 86_400) as i64;
    let remainder = seconds % 86_400;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}+00:00",
        remainder / 3600,
        (remainder % 3600) / 60,
        remainder % 60
    ))
}

fn scrub(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object.iter_mut() {
                if key.eq_ignore_ascii_case("appid") || key.eq_ignore_ascii_case("app_id") {
                    *value = Value::String("[REDACTED]".to_owned());
                } else {
                    scrub(value);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(scrub),
        _ => {}
    }
}

fn fetch(
    client: &Client,
    endpoint: &str,
    params: &BTreeMap<String, String>,
    app_id: &str,
) -> Result<Value, Box<dyn Error>> {
    let url = format!("{BASE}/{endpoint}");
    let mut query: Vec<(&str, &str)> = params
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    query.push(("appId", app_id));
    Ok(client
        .get(url)
        .query(&query)
        .send()?
        .error_for_status()?
        .json()?)
}

fn save(name: &str, mut payload: Value) -> Result<String, Box<dyn Error>> {
    scrub(&mut payload);
    let relative = format!("samples/{name}");
    fs::write(
        Path::new(ROOT).join(&relative),
        serde_json::to_vec_pretty(&payload)?,
    )?;
    Ok(relative)
}

fn parameters(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

fn stats(client: &Client, app_id: &str, params: &[(&str, &str)]) -> Result<Value, Box<dyn Error>> {
    fetch(client, "getStatsData", &parameters(params), app_id)
}

fn as_items(value: &Value) -> Vec<Value> {
    match value {
        Value::Array(items) => items.clone(),
        Value::Null => Vec::new(),
        Value::String(s) if s.is_empty() => Vec::new(),
        Value::Object(_) => vec![value.clone()],
        _ => Vec::new(),
    }
}

fn number(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str()?.parse().ok())
        .or_else(|| value.get("$").and_then(number))
}

/// Check that a full scan summary covers every record once, with continuous page boundaries.
pub fn validate_scan_payload(scan: &Value, expected_total: u64) -> Result<(), Box<dyn Error>> {
    if number(&scan["recordCount"]) != Some(expected_total)
        || scan["tableId"].as_str() != Some(TABLE)
    {
        return Err("full-scan summary does not match the requested table/count".into());
    }
    let pages = scan["pages"]
        .as_array()
        .ok_or("full-scan summary has no page boundaries")?;
    if pages.is_empty() {
        return Err("full-scan summary has no page boundaries".into());
    }
    let mut next = 1_u64;
    for page in pages {
        let start = number(&page["startPosition"]).ok_or("missing scan start position")?;
        let from = number(&page["fromNumber"]).ok_or("missing scan FROM_NUMBER")?;
        let to = number(&page["toNumber"]).ok_or("missing scan TO_NUMBER")?;
        if start != next || from != start || to < from {
            return Err("full-scan summary has inconsistent page boundaries".into());
        }
        next = to + 1;
    }
    if next - 1 != expected_total {
        return Err("full-scan page boundaries do not cover the requested record count".into());
    }
    Ok(())
}

pub fn run(app_id: &str, limit: u32, scan_all: bool) -> Result<(), Box<dyn Error>> {
    if app_id.is_empty() || limit == 0 {
        return Err("application ID and a positive page limit are required".into());
    }
    let samples = Path::new(ROOT).join("samples");
    fs::create_dir_all(&samples)?;
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(60))
        .build()?;
    let captured = timestamp()?;
    let meta_params = parameters(&[
        ("statsDataId", TABLE),
        ("lang", "J"),
        ("explanationGetFlg", "Y"),
    ]);
    let metadata = fetch(&client, "getMetaInfo", &meta_params, app_id)?;
    let meta_path = save("meta_info.json", metadata.clone())?;
    let mut pages = Vec::new();
    for start in [1, limit + 1] {
        let start_text = start.to_string();
        let limit_text = limit.to_string();
        let response = stats(
            &client,
            app_id,
            &[
                ("statsDataId", TABLE),
                ("startPosition", &start_text),
                ("limit", &limit_text),
                ("metaGetFlg", "N"),
                ("explanationGetFlg", "Y"),
            ],
        )?;
        pages.push(json!({"startPosition": start, "limit": limit, "sample": save(&format!("stats_data_start_{start}_limit_{limit}.json"), response)?}));
    }

    let single = [
        ("statsDataId", TABLE),
        ("startPosition", "1"),
        ("limit", "3"),
        ("metaGetFlg", "N"),
        ("cdTab", "06"),
        ("cdCat01", "00"),
        ("cdCat02", "02"),
        ("cdCat03", "00"),
        ("cdCat04", "00"),
        ("cdCat05", "0"),
        ("cdArea", "00000"),
        ("cdTime", "2002000000"),
    ];
    let single_path = save(
        "stats_data_single_filtered.json",
        stats(&client, app_id, &single)?,
    )?;
    let empty = [
        ("statsDataId", TABLE),
        ("startPosition", "1"),
        ("limit", "3"),
        ("metaGetFlg", "N"),
        ("cdTab", "06"),
        ("cdCat01", "20"),
        ("cdCat02", "02"),
        ("cdCat03", "03"),
        ("cdCat04", "10"),
        ("cdCat05", "2"),
        ("cdArea", "00000"),
        ("cdTime", "2002000000"),
    ];
    let empty_path = save(
        "stats_data_no_data_filtered.json",
        stats(&client, app_id, &empty)?,
    )?;
    let special = [
        ("statsDataId", TABLE),
        ("startPosition", "1"),
        ("limit", "1"),
        ("metaGetFlg", "N"),
        ("cdTab", "06"),
        ("cdCat01", "00"),
        ("cdCat02", "02"),
        ("cdCat03", "00"),
        ("cdCat04", "00"),
        ("cdCat05", "0"),
        ("cdArea", "00000"),
        ("cdTime", "2011000000"),
    ];
    let special_path = save(
        "stats_data_special_value.json",
        stats(&client, app_id, &special)?,
    )?;
    let zero = [
        ("statsDataId", TABLE),
        ("startPosition", "1"),
        ("limit", "1"),
        ("metaGetFlg", "N"),
        ("cdTab", "06"),
        ("cdCat01", "00"),
        ("cdCat02", "02"),
        ("cdCat03", "03"),
        ("cdCat04", "02"),
        ("cdCat05", "0"),
        ("cdArea", "00000"),
        ("cdTime", "2003000000"),
    ];
    let zero_path = save("stats_data_zero_value.json", stats(&client, app_id, &zero)?)?;
    let endpoint = |sample: &str, params: &[(&str, &str)]| json!({"params": params.iter().filter(|(key, _)| *key != "statsDataId").map(|(key, value)| ((*key).to_owned(), (*value).to_owned())).collect::<BTreeMap<_, _>>(), "sample": sample});
    let mut manifest = json!({
        "capturedAtUtc": captured, "tableId": TABLE,
        "endpoints": {
            "getMetaInfo": {"url": format!("{BASE}/getMetaInfo"), "params": {"statsDataId": TABLE, "lang": "J", "explanationGetFlg": "Y"}, "sample": meta_path},
            "getStatsData": {"url": format!("{BASE}/getStatsData"), "pages": pages, "fixedParams": {"statsDataId": TABLE, "metaGetFlg": "N"},
                "singleFiltered": endpoint(&single_path, &single), "noDataFiltered": endpoint(&empty_path, &empty), "specialValue": endpoint(&special_path, &special), "zeroValue": endpoint(&zero_path, &zero)}
        },
        "credentialHandling": "ESTAT_APP_ID is read from the environment; appId values in all saved JSON are replaced by [REDACTED]."
    });
    if scan_all {
        let mut total =
            number(&metadata["GET_META_INFO"]["METADATA_INF"]["TABLE_INF"]["OVERALL_TOTAL_NUMBER"])
                .ok_or("metadata did not include the overall record count")?;
        let expected_total = total;
        let scan_limit = 10_000_u64;
        let mut counts = BTreeMap::<String, u64>::new();
        let mut examples = Map::new();
        let mut annotation_keys = BTreeSet::new();
        let mut seen = HashSet::new();
        let mut scan_pages = Vec::new();
        let mut education_examples = Vec::new();
        let mut start = 1_u64;
        while start <= total {
            if start > 1 {
                thread::sleep(Duration::from_secs(1));
            }
            let start_text = start.to_string();
            let limit_text = scan_limit.to_string();
            let payload = stats(
                &client,
                app_id,
                &[
                    ("statsDataId", TABLE),
                    ("startPosition", &start_text),
                    ("limit", &limit_text),
                    ("metaGetFlg", "N"),
                ],
            )?;
            let statistical = &payload["GET_STATS_DATA"]["STATISTICAL_DATA"];
            let values = as_items(&statistical["DATA_INF"]["VALUE"]);
            if values.is_empty() {
                return Err(format!("empty full-scan page at {start}").into());
            }
            let result = &statistical["RESULT_INF"];
            let returned_total =
                number(&result["TOTAL_NUMBER"]).ok_or("scan response omitted TOTAL_NUMBER")?;
            let from = number(&result["FROM_NUMBER"]).ok_or("scan response omitted FROM_NUMBER")?;
            let to = number(&result["TO_NUMBER"]).ok_or("scan response omitted TO_NUMBER")?;
            if returned_total != total || from != start || to != start + values.len() as u64 - 1 {
                return Err(format!("inconsistent full-scan page at {start}").into());
            }
            scan_pages.push(json!({"startPosition": start, "limit": scan_limit, "fromNumber": from, "toNumber": to}));
            for value in values {
                let key = [
                    "tab", "cat01", "cat02", "cat03", "cat04", "cat05", "area", "time",
                ]
                .map(|axis| value[format!("@{axis}")].clone());
                if !seen.insert(serde_json::to_string(&key)?) {
                    return Err(format!("duplicate source record at scan position {start}").into());
                }
                if let Some(token) = value["$"]
                    .as_str()
                    .filter(|s| ["0", "*", "***", "…"].contains(s))
                {
                    *counts.entry(token.to_owned()).or_default() += 1;
                    examples.entry(token.to_owned()).or_insert(value.clone());
                }
                if let Some(object) = value.as_object() {
                    annotation_keys.extend(
                        object
                            .keys()
                            .filter(|key| key.to_lowercase().contains("annotation"))
                            .cloned(),
                    );
                }
                if value["@tab"] == "06"
                    && value["@area"] == "00000"
                    && value["@cat01"] == "00"
                    && value["@cat02"] == "02"
                    && value["@cat03"] == "00"
                    && value["@cat05"] == "0"
                    && ["2024000000", "2025000000"]
                        .contains(&value["@time"].as_str().unwrap_or_default())
                {
                    education_examples.push(value.clone());
                }
            }
            start += to - from + 1;
            total = returned_total;
        }
        if start - 1 != expected_total {
            return Err(format!("scan returned {} of {expected_total} records", start - 1).into());
        }
        let scan = json!({"capturedAtUtc": captured, "tableId": TABLE, "recordCount": expected_total, "pageLimit": scan_limit, "pages": scan_pages, "specialValueCounts": counts, "specialValueSamples": examples, "annotationKeysObserved": annotation_keys});
        validate_scan_payload(&scan, expected_total)?;
        save("special_value_scan.json", scan)?;
        manifest["fullValueScan"] = json!({"sample": "samples/special_value_scan.json", "recordCount": expected_total, "pageLimit": scan_limit, "pageCount": (expected_total + scan_limit - 1) / scan_limit});
        let education_examples_path = save(
            "education_value_examples.json",
            json!({
                "sample_note": "Current e-Stat VALUE records for 2024 and 2025, holding age, employment status/type, and sex at their total categories; missing category rows are not imputed.",
                "conditions": {"tab": "06", "area": "00000", "cat01": "00", "cat02": "02", "cat03": "00", "cat05": "0"},
                "education_codes": ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10"],
                "VALUE": education_examples
            }),
        )?;
        manifest["educationValueExamples"] = json!({
            "sample": education_examples_path,
            "recordCount": education_examples.len(),
            "years": ["2024", "2025"]
        });
    }
    fs::write(
        Path::new(ROOT).join("capture_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!("Captured scrubbed responses for table {TABLE} at {captured} (credentials omitted).");
    if scan_all {
        println!("Full-value scan and education examples saved under samples/.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_in_scan_matches_manifest_and_covers_every_record() {
        let manifest: Value = serde_json::from_str(include_str!("../../../../inspections/workers_by_age_employment_type_education_sex/capture_manifest.json")).unwrap();
        let scan: Value = serde_json::from_str(include_str!("../../../../inspections/workers_by_age_employment_type_education_sex/samples/special_value_scan.json")).unwrap();
        let reference = &manifest["fullValueScan"];
        let count = number(&reference["recordCount"]).unwrap();
        assert_eq!(scan["recordCount"], reference["recordCount"]);
        assert_eq!(scan["pageLimit"], reference["pageLimit"]);
        assert_eq!(
            scan["pages"].as_array().unwrap().len() as u64,
            number(&reference["pageCount"]).unwrap()
        );
        validate_scan_payload(&scan, count).unwrap();
    }

    #[test]
    fn scan_validator_rejects_page_gaps() {
        let scan = json!({"tableId": TABLE, "recordCount": 4, "pages": [{"startPosition": 1, "fromNumber": 1, "toNumber": 2}, {"startPosition": 4, "fromNumber": 4, "toNumber": 4}]});
        assert!(validate_scan_payload(&scan, 4).is_err());
    }
}
