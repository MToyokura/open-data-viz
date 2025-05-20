use std::{error::Error, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::modules::{
    estat_api::get_meta_info::{GetMetaInfoParams, get_meta_info},
    io::file_writer::{self, WriteArgs},
    projects::commodity_prices::{COMMODITY_STATS_DATA_ID, OUTPUT_DIR_META},
};

#[derive(Deserialize, Debug)]
pub struct Root {
    #[serde(rename = "GET_META_INFO")]
    pub get_meta_info_root: GetMetaInfo,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct GetMetaInfo {
    #[serde(rename = "METADATA_INF")]
    pub metadata_inf: MetadataInf,
    #[serde(rename = "PARAMETER")]
    pub parameter: Value,
    #[serde(rename = "RESULT")]
    pub result: Value,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct MetadataInf {
    #[serde(rename = "CLASS_INF")]
    pub class_inf: ClassInf,
    #[serde(rename = "TABLE_INF")]
    pub table_inf: Value,
}

#[derive(Deserialize, Debug)]
pub struct ClassInf {
    #[serde(rename = "CLASS_OBJ")]
    pub class_obj: Vec<ClassObj>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ClassField {
    Single(ClassDetails),
    Multiple(Vec<ClassDetails>),
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct ClassObj {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "CLASS")]
    pub class: ClassField,
    #[serde(rename = "EXPLANATION", default)]
    pub explanation: Vec<Explanation>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct ClassDetails {
    #[serde(rename = "@code")]
    pub code: String,
    #[serde(rename = "@level", default)]
    pub level: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@unit", default)]
    pub unit: Option<String>,
    #[serde(rename = "@parentCode", default)]
    pub parent_code: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Explanation {
    #[serde(rename = "$")]
    pub value: String,
    #[serde(rename = "@id")]
    pub id: String,
}

#[derive(Debug)]
pub struct Combination {
    pub cat02_code: String,
    pub area_code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AreaEntry {
    pub area_code: String,
    pub area_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cat02Entry {
    pub cat02_code: String,
    pub cat02_name: String,
}

#[derive(Debug)]
pub struct CommodityMetaData {
    pub cat02_lookup: Vec<Cat02Entry>,
    pub area_lookup: Vec<AreaEntry>,
}

/// Fetches and saves the meta info data. Returns the raw JSON string.
pub fn fetch_write_commodity_meta_info(app_id: &str) -> Result<String, Box<dyn Error>> {
    let meta_info_file_name = "get_commodity_meta_info.json";
    let meta_info_file_path = PathBuf::from(OUTPUT_DIR_META).join(meta_info_file_name);

    // If the file already exists, skip the fetch and return the existing file
    if meta_info_file_path.exists() {
        println!(
            "Meta info file already exists: {}",
            meta_info_file_path.display()
        );
        let json_string = std::fs::read_to_string(&meta_info_file_path)?;
        return Ok(json_string);
    }

    println!("Fetching meta info for commodity prices...");

    let get_meta_info_result = get_meta_info(GetMetaInfoParams {
        app_id: &app_id,
        stats_data_id: COMMODITY_STATS_DATA_ID,
        ..Default::default()
    })?;

    file_writer::write_to_directory(WriteArgs {
        file_name: meta_info_file_name,
        directory: PathBuf::from(OUTPUT_DIR_META),
        extension: "json",
        content: &get_meta_info_result,
    })?;

    println!("Successfully fetched and saved meta info JSON.");

    Ok(get_meta_info_result)
}

/// Sorts and writes the sorted cat02 lookup to a file
fn write_return_sorted_cat02_lookup(
    cat02_details: &Vec<ClassDetails>,
) -> Result<Vec<Cat02Entry>, Box<dyn Error>> {
    let mut cat02_lookup: Vec<Cat02Entry> = Vec::new();
    for detail in cat02_details {
        cat02_lookup.push(Cat02Entry {
            cat02_code: detail.code.clone(),
            cat02_name: detail.name.clone(),
        });
    }

    let mut sorted_cat02_entries: Vec<Cat02Entry> = cat02_lookup.clone();
    sorted_cat02_entries.sort_by(|a, b| a.cat02_code.cmp(&b.cat02_code));

    file_writer::write_to_directory(WriteArgs {
        file_name: "cat02_lookup.json",
        directory: PathBuf::from(OUTPUT_DIR_META),
        extension: "json",
        content: &serde_json::to_string_pretty(&sorted_cat02_entries)?,
    })?;

    Ok(sorted_cat02_entries)
}

/// Sorts and writes the sorted area lookup to a file
fn write_return_sorted_area_lookup(
    area_details: &Vec<ClassDetails>,
) -> Result<Vec<AreaEntry>, Box<dyn Error>> {
    let mut area_lookup: Vec<AreaEntry> = Vec::new();
    for detail in area_details {
        area_lookup.push(AreaEntry {
            area_code: detail.code.clone(),
            area_name: detail.name.clone(),
        });
    }

    let mut sorted_area_entries: Vec<AreaEntry> = area_lookup.clone();
    sorted_area_entries.sort_by(|a, b| a.area_code.cmp(&b.area_code));

    file_writer::write_to_directory(WriteArgs {
        file_name: "area_lookup.json",
        directory: PathBuf::from(OUTPUT_DIR_META),
        extension: "json",
        content: &serde_json::to_string_pretty(&sorted_area_entries)?,
    })?;

    Ok(sorted_area_entries)
}

/// Processes the raw JSON string to create the `CommodityMetaData`.
pub fn generate_commodity_meta_data(
    json_string: &str,
) -> Result<CommodityMetaData, Box<dyn Error>> {
    println!("Parsing JSON and generating metadata for commodity prices...");

    let root: Root = serde_json::from_str(json_string)?;

    let mut cat02_details: Vec<ClassDetails> = Vec::new();
    let mut area_details: Vec<ClassDetails> = Vec::new();

    for class_obj in &root.get_meta_info_root.metadata_inf.class_inf.class_obj {
        match class_obj.id.as_str() {
            "cat02" => match &class_obj.class {
                ClassField::Single(details) => {
                    cat02_details.push(details.clone());
                }
                ClassField::Multiple(details_vec) => {
                    cat02_details.extend(details_vec.iter().cloned());
                }
            },
            "area" => match &class_obj.class {
                ClassField::Single(details) => {
                    area_details.push(details.clone());
                }
                ClassField::Multiple(details_vec) => {
                    area_details.extend(details_vec.iter().cloned());
                }
            },
            _ => {}
        }
    }

    println!("Successfully parsed JSON.");
    println!("  Length of cat02_details: {}", cat02_details.len());
    println!("  Length of area_details: {}", area_details.len());

    let sorted_cat02_lookup = write_return_sorted_cat02_lookup(&cat02_details)?;
    println!("Sorted cat02_lookup written to file.");

    let sorted_area_lookup = write_return_sorted_area_lookup(&area_details)?;
    println!("Sorted area_lookup written to file.");

    Ok(CommodityMetaData {
        cat02_lookup: sorted_cat02_lookup,
        area_lookup: sorted_area_lookup,
    })
}

pub fn fetch_write_transform_commodity_metadata(
    app_id: &str,
) -> Result<CommodityMetaData, Box<dyn Error>> {
    let json_string = fetch_write_commodity_meta_info(app_id)?;
    let commodity_meta_data = generate_commodity_meta_data(&json_string)?;
    println!("Successfully generated commodity meta data.");
    Ok(commodity_meta_data)
}
