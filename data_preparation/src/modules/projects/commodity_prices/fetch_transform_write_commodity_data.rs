use std::{error::Error, path::PathBuf};

use crate::modules::{
    estat_api::get_stats_data::{GetStatsDataParams, get_stats_data},
    io::file_writer::{self, WriteArgs},
    projects::commodity_prices::{OUTPUT_DIR_RAW, OUTPUT_DIR_TRANSFORMED},
};

use super::{
    COMMODITY_STATS_DATA_ID,
    get_commodity_meta_info::{AreaEntry, Cat02Entry},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Root {
    #[serde(rename = "GET_STATS_DATA")]
    pub get_stats_data_root: GetStatsDataInner,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStatsDataInner {
    #[serde(rename = "PARAMETER")]
    pub parameter: Value,
    #[serde(rename = "RESULT")]
    pub result: Value,
    #[serde(rename = "STATISTICAL_DATA")]
    pub statistical_data: StatisticalData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticalData {
    #[serde(rename = "CLASS_INF")]
    pub class_inf: ClassInf,
    #[serde(rename = "DATA_INF")]
    pub data_inf: Option<DataInf>,
    #[serde(rename = "RESULT_INF")]
    pub result_inf: Value,
    #[serde(rename = "TABLE_INF")]
    pub table_inf: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassInf {
    #[serde(rename = "CLASS_OBJ")]
    pub class_obj: Vec<ClassObj>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassObj {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "CLASS")]
    pub class: Option<ClassField>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClassField {
    Single(ClassDetails),
    Multiple(Vec<ClassDetails>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClassDetails {
    #[serde(rename = "@code")]
    pub code: Option<String>,
    #[serde(rename = "@level")]
    pub level: Option<String>,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@parentCode")]
    pub parent_code: Option<String>,
    #[serde(rename = "@unit")]
    pub unit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataInf {
    #[serde(rename = "NOTE")]
    pub note: Vec<Note>,
    #[serde(rename = "VALUE")]
    pub value: Vec<ValueEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    #[serde(rename = "$")]
    pub content: String,
    #[serde(rename = "@char")]
    pub char: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValueEntry {
    #[serde(rename = "$")]
    pub value: String,
    #[serde(rename = "@area")]
    pub area: String,
    #[serde(rename = "@cat01")]
    pub cat01: String,
    #[serde(rename = "@cat02")]
    pub cat02: String,
    #[serde(rename = "@tab")]
    pub tab: String,
    #[serde(rename = "@time")]
    pub time: String,
    #[serde(rename = "@unit")]
    pub unit: String,
}

pub struct GetCommodityStatsDataParams<'a> {
    pub app_id: &'a str,
    pub cat02_entry: &'a Cat02Entry,
    pub area_lookup: &'a [AreaEntry],
}

// A vector of time and value pairs
#[derive(Debug, Serialize, Deserialize)]
pub struct ValueContent {
    pub area: String,
    pub time: String,
    pub value: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TransformedData {
    pub cat02_code: String,
    pub cat02_name: String,
    pub time_value_pairs: Vec<ValueContent>,
}

fn transform_data(
    root: &Root,
    cat02_code: &str,
    cat02_name: &str,
) -> Result<String, Box<dyn Error>> {
    let mut time_value_pairs = Vec::new();

    // Check if data_inf is Some, and get a reference to the inner DataInf struct
    if let Some(data_inf) = &root.get_stats_data_root.statistical_data.data_inf {
        // Now data_inf is a reference to DataInf, so we can safely access .value
        for value_entry in &data_inf.value {
            if value_entry.cat02 == cat02_code {
                time_value_pairs.push(ValueContent {
                    area: value_entry.area.clone(),
                    time: value_entry.time.clone(),
                    value: value_entry.value.clone(),
                });
            }
        }
    }
    // If data_inf was None, time_value_pairs remains empty.

    let transformed_data = TransformedData {
        cat02_code: cat02_code.to_string(),
        cat02_name: cat02_name.to_string(),
        time_value_pairs,
    };

    // Serialize the transformed data to JSON
    let transformed_data_json = serde_json::to_string(&transformed_data)?;

    Ok(transformed_data_json)
}

fn fetch_commodity_stats_data(
    params: &GetCommodityStatsDataParams,
) -> Result<String, Box<dyn Error>> {
    // Check if the file already exists
    let file_name = format!(
        "get_commodity_prices_stats_data_{}.json",
        &params.cat02_entry.cat02_code
    );
    let file_path = PathBuf::from(OUTPUT_DIR_RAW).join(&file_name);
    if file_path.exists() {
        println!("File {} already exists. Skipping fetch.", file_name);
        let json_string = std::fs::read_to_string(&file_path)?;

        return Ok(json_string);
    }

    // Wait for 2 seconds before the next request
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Fetch the data
    let stats_data = get_stats_data(GetStatsDataParams {
        app_id: params.app_id,
        stats_data_id: Some(COMMODITY_STATS_DATA_ID),
        cd_cat02: Some(&params.cat02_entry.cat02_code),
        ..Default::default()
    })?;

    Ok(stats_data)
}

/// Fetches, transforms, and writes a single commodity prices data to files.
pub fn fetch_transform_write_commodity_data(
    params: GetCommodityStatsDataParams,
) -> Result<(), Box<dyn Error>> {
    let stats_data = fetch_commodity_stats_data(&params)?;

    // Write the raw JSON string to a file
    let file_name = format!(
        "get_commodity_prices_stats_data_{}.json",
        &params.cat02_entry.cat02_code
    );
    let args = WriteArgs {
        file_name: &file_name,
        directory: PathBuf::from(OUTPUT_DIR_RAW),
        extension: "json",
        content: &stats_data,
    };
    file_writer::write_to_directory(args)?;
    println!("  Successfully written raw JSON to file: {}", file_name);

    // Deserialize the JSON string into the Root struct
    let root: Root = match serde_json::from_str(&stats_data) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            return Err(Box::new(e));
        }
    };

    #[derive(Serialize, Deserialize)]
    struct TimeValuePairs {
        time: String,
        value: String,
    }
    #[derive(Serialize, Deserialize)]

    struct AreaCat02Data {
        cat02_code: String,
        cat02_name: String,
        area_code: String,
        area_name: String,
        time_value_pairs: Vec<TimeValuePairs>,
    }

    #[derive(Serialize, Deserialize)]
    struct AreaDataContainer {
        area_data: Vec<AreaCat02Data>,
    }

    // Create container for storing area data
    let mut area_data_container = AreaDataContainer {
        area_data: Vec::new(),
    };

    // Iterate over the class_inf and data_inf to populate area_data
    let values = &root.get_stats_data_root.statistical_data.data_inf;
    if let Some(data_inf) = values {
        for value_entry in &data_inf.value {
            // Find the area name from the area_lookup
            let area_name = params
                .area_lookup
                .iter()
                .find(|entry| entry.area_code == value_entry.area)
                .map(|entry| entry.area_name.clone())
                .unwrap_or_else(|| "Unknown Area".to_string());

            // Create a new TimeValuePairs instance
            let time_value_pair = TimeValuePairs {
                time: value_entry.time.clone(),
                value: value_entry.value.clone(),
            };

            // Check if the area data already exists
            if let Some(area_data) = area_data_container
                .area_data
                .iter_mut()
                .find(|data| data.area_code == value_entry.area)
            {
                // If it exists, push the new time_value_pair to it
                area_data.time_value_pairs.push(time_value_pair);
            } else {
                // If it doesn't exist, create a new AreaData instance
                let new_area_data = AreaCat02Data {
                    area_code: value_entry.area.clone(),
                    area_name,
                    cat02_code: value_entry.cat02.clone(),
                    cat02_name: params.cat02_entry.cat02_name.clone(),
                    time_value_pairs: vec![time_value_pair],
                };
                area_data_container.area_data.push(new_area_data);
            }
        }
    }

    // Parse each area name data to JSON and write to file
    for area_data in &area_data_container.area_data {
        let transformed_commodity_prices_file_name = format!(
            "commodity_prices_{}_{}",
            &area_data.cat02_code, &area_data.area_code
        );
        let area_json_string = serde_json::to_string(&area_data)?;

        // Write the transformed JSON string to a file
        let args = WriteArgs {
            file_name: &transformed_commodity_prices_file_name,
            directory: PathBuf::from(OUTPUT_DIR_TRANSFORMED),
            extension: "json",
            content: &area_json_string,
        };

        // If file already exists, skip writing
        let file_path = PathBuf::from(OUTPUT_DIR_TRANSFORMED)
            .join(&transformed_commodity_prices_file_name)
            .with_extension("json");
        if file_path.exists() {
            println!(
                "File {} already exists. Skipping write.",
                transformed_commodity_prices_file_name
            );
            continue;
        }

        file_writer::write_to_directory(args)?;
        println!(
            "  Successfully written transformed JSON to file: {}.json",
            transformed_commodity_prices_file_name
        );
    }

    Ok(())
}
