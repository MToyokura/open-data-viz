//! Initial e-Stat inspection capture for the 2025-base CPI table.

use reqwest::blocking::Client;
use serde_json::{Value, json};
use std::{error::Error, fs, path::Path, time::Duration};

const TABLE: &str = "0004052037";
const BASE: &str = "https://api.e-stat.go.jp/rest/3.0/app/json";
const ROOT: &str = "inspections/consumer_price_index";

fn fetch(
    client: &Client,
    endpoint: &str,
    params: &[(&str, &str)],
    app_id: &str,
) -> Result<Value, Box<dyn Error>> {
    let mut query = params.to_vec();
    query.push(("appId", app_id));
    Ok(client
        .get(format!("{BASE}/{endpoint}"))
        .query(&query)
        .send()?
        .error_for_status()?
        .json()?)
}

fn scrub(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object.iter_mut() {
                if key.eq_ignore_ascii_case("appid") || key.eq_ignore_ascii_case("app_id") {
                    *value = Value::String("[REDACTED]".into());
                } else {
                    scrub(value);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(scrub),
        _ => {}
    }
}

fn save(name: &str, mut value: Value) -> Result<String, Box<dyn Error>> {
    scrub(&mut value);
    let path = format!("{ROOT}/samples/{name}");
    fs::write(&path, serde_json::to_vec_pretty(&value)?)?;
    Ok(path)
}

pub fn run(app_id: &str) -> Result<(), Box<dyn Error>> {
    if app_id.is_empty() {
        return Err("application ID is required".into());
    }
    fs::create_dir_all(Path::new(ROOT).join("samples"))?;
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(60))
        .build()?;
    let metadata_params = [
        ("statsDataId", TABLE),
        ("lang", "J"),
        ("explanationGetFlg", "Y"),
    ];
    let metadata = fetch(&client, "getMetaInfo", &metadata_params, app_id)?;
    let metadata_sample = save("meta_info.json", metadata)?;

    let mut samples = Vec::new();
    for start in ["1", "4"] {
        let response = fetch(
            &client,
            "getStatsData",
            &[
                ("statsDataId", TABLE),
                ("startPosition", start),
                ("limit", "3"),
                ("metaGetFlg", "N"),
            ],
            app_id,
        )?;
        let name = format!("stats_data_start_{start}_limit_3.json");
        samples.push(json!({"startPosition": start, "limit": 3, "sample": save(&name, response)?}));
    }
    let volume_sample = fetch(
        &client,
        "getStatsData",
        &[
            ("statsDataId", TABLE),
            ("startPosition", "1"),
            ("limit", "1000"),
            ("metaGetFlg", "N"),
        ],
        app_id,
    )?;
    let volume_sample_path = save("stats_data_start_1_limit_1000.json", volume_sample)?;
    let index_measure_count_sample = fetch(
        &client,
        "getStatsData",
        &[
            ("statsDataId", TABLE),
            ("cdTab", "1"),
            ("startPosition", "1"),
            ("limit", "3"),
            ("metaGetFlg", "N"),
        ],
        app_id,
    )?;
    let index_measure_count_path = save(
        "index_measure_start_1_limit_3.json",
        index_measure_count_sample,
    )?;
    let monthly_measure_count_sample = fetch(
        &client,
        "getStatsData",
        &[
            ("statsDataId", TABLE),
            ("cdTab", "1"),
            ("lvTime", "4"),
            ("startPosition", "1"),
            ("limit", "3"),
            ("metaGetFlg", "N"),
        ],
        app_id,
    )?;
    let monthly_measure_count_path = save(
        "index_monthly_measure_start_1_limit_3.json",
        monthly_measure_count_sample,
    )?;
    let series = fetch(
        &client,
        "getStatsData",
        &[
            ("statsDataId", TABLE),
            ("cdTab", "1"),
            ("cdCat01", "0001"),
            ("cdArea", "00000"),
            ("startPosition", "1"),
            ("limit", "1000"),
            ("metaGetFlg", "N"),
        ],
        app_id,
    )?;
    let series_sample = save("index_national_overall_series.json", series)?;
    let mut representative_series = Vec::new();
    for item in ["0002", "1001"] {
        for (area, area_name) in [("00000", "national"), ("13100", "tokyo")] {
            let response = fetch(
                &client,
                "getStatsData",
                &[
                    ("statsDataId", TABLE),
                    ("lvTime", "4"),
                    ("cdTab", "1"),
                    ("cdCat01", item),
                    ("cdArea", area),
                    ("startPosition", "1"),
                    ("limit", "1000"),
                    ("metaGetFlg", "N"),
                ],
                app_id,
            )?;
            let file_name = format!("index_item_{item}_area_{area}_series.json");
            representative_series.push(json!({
                "itemCode": item,
                "areaCode": area,
                "areaLabel": area_name,
                "params": {"lvTime": "4", "cdTab": "1", "cdCat01": item, "cdArea": area, "startPosition": "1", "limit": "1000"},
                "sample": save(&file_name, response)?
            }));
        }
    }
    let manifest = json!({
        "tableId": TABLE,
        "capturedAtUtc": std::process::Command::new("date").args(["-u", "+%Y-%m-%dT%H:%M:%SZ"]).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned()),
        "endpoints": {
            "getMetaInfo": {"url": format!("{BASE}/getMetaInfo"), "params": {"statsDataId": TABLE, "lang": "J", "explanationGetFlg": "Y"}, "sample": metadata_sample},
            "getStatsData": {
                "url": format!("{BASE}/getStatsData"),
                "fixedParams": {"statsDataId": TABLE, "metaGetFlg": "N"},
                "pages": samples,
                "volumeSample": {"params": {"startPosition": "1", "limit": "1000"}, "sample": volume_sample_path},
                "indexMeasureCountSample": {"params": {"cdTab": "1", "startPosition": "1", "limit": "3"}, "sample": index_measure_count_path},
                "monthlyIndexCountSample": {"params": {"cdTab": "1", "lvTime": "4", "startPosition": "1", "limit": "3"}, "sample": monthly_measure_count_path},
                "indexSeriesSample": {"params": {"cdTab": "1", "cdCat01": "0001", "cdArea": "00000", "startPosition": "1", "limit": "1000"}, "sample": series_sample},
                "representativeSeries": representative_series
            }
        },
        "credentialHandling": "ESTAT_APP_ID is read from the environment; saved API responses have appId values redacted."
    });
    fs::write(
        Path::new(ROOT).join("capture_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!("Captured scrubbed metadata and small API samples for table {TABLE}.");
    Ok(())
}

pub fn run_inspection(args: &[String]) -> bool {
    if args.first().map(String::as_str) != Some("consumer-price-index-inspect") {
        return false;
    }
    let app_id = match crate::modules::estat_api::app_id::retrieve_estat_app_id() {
        Ok(app_id) => app_id,
        Err(_) => {
            eprintln!("ESTAT_APP_ID not found");
            std::process::exit(1);
        }
    };
    if let Err(error) = run(&app_id) {
        let detail = error.to_string().replace(&app_id, "[REDACTED]");
        eprintln!("CPI inspection failed: {detail}");
        std::process::exit(1);
    }
    true
}
