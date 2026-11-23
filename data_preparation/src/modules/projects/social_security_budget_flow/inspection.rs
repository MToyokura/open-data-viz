use std::{
    error::Error,
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;

use crate::modules::estat_api::{
    EstatClient,
    get_meta_info::{GetMetaInfoParamsBuilder, MandatoryParams as MetaParams},
    get_stats_data::{GetStatsDataParamsBuilder, MandatoryParams as DataParams},
};

use super::{EXPENDITURE_TABLE_ID, INCOME_TABLE_ID};

const OUTPUT: &str = "data_preparation/inspections/social_security_budget_flow";

pub fn run_inspection(app_id: &str) -> Result<(), Box<dyn Error>> {
    let client = EstatClient::new()?;
    fs::create_dir_all(OUTPUT)?;
    let captured_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let mut manifest = serde_json::json!({
        "captured_at_unix": captured_at,
        "method": "existing estat_api getMetaInfo + getStatsData; sample limit 100",
        "tables": []
    });
    for table_id in [INCOME_TABLE_ID, EXPENDITURE_TABLE_ID] {
        let meta = client
            .get_meta_info(
                GetMetaInfoParamsBuilder::new(MetaParams {
                    app_id,
                    stats_data_id: table_id,
                })
                .lang("J")
                .build(),
            )
            .map_err(|error| format!("metadata request for table {table_id} failed: {error}"))?;
        let mut meta_json: Value = serde_json::from_str(&meta)?;
        redact_app_id(&mut meta_json);
        fs::write(
            Path::new(OUTPUT).join(format!("{table_id}_metadata.json")),
            serde_json::to_vec_pretty(&meta_json)?,
        )?;

        let data = client
            .get_stats_data(
                GetStatsDataParamsBuilder::new(DataParams { app_id })
                    .stats_data_id(table_id)
                    .start_position(1)
                    .limit(100)
                    .meta_get_flg("N")
                    .build(),
            )
            .map_err(|error| format!("sample request for table {table_id} failed: {error}"))?;
        let mut data_json: Value = serde_json::from_str(&data)?;
        redact_app_id(&mut data_json);
        let statistical_data = &data_json["GET_STATS_DATA"]["STATISTICAL_DATA"];
        let result = &statistical_data["RESULT_INF"];
        let total = result["TOTAL_NUMBER"].to_string();
        let sample_count = match &statistical_data["DATA_INF"]["VALUE"] {
            Value::Array(items) => items.len(),
            Value::Object(_) => 1,
            _ => 0,
        };
        fs::write(
            Path::new(OUTPUT).join(format!("{table_id}_sample.json")),
            serde_json::to_vec_pretty(&data_json)?,
        )?;
        manifest["tables"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "stats_data_id": table_id,
                "reported_total_records": total,
                "sample_records": sample_count,
                "metadata_file": format!("{table_id}_metadata.json"),
                "sample_file": format!("{table_id}_sample.json")
            }));
    }
    fs::write(
        Path::new(OUTPUT).join("inspection_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!("Saved e-Stat inspection samples to {OUTPUT}");
    Ok(())
}

fn redact_app_id(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, item) in map.iter_mut() {
                if key.eq_ignore_ascii_case("appid") {
                    *item = Value::String("REDACTED".into());
                } else {
                    redact_app_id(item);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_app_id(item);
            }
        }
        _ => {}
    }
}
