//! Build the small runtime catalogue from successfully generated product files.
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Serialize)]
struct Product {
    key: String,
    label: String,
}

#[derive(Serialize)]
struct ProductIndex {
    schema_version: u8,
    products: Vec<Product>,
}

fn fail(message: impl Into<String>) -> Box<dyn Error> {
    message.into().into()
}

pub fn generate(output_dir: &Path) -> Result<PathBuf> {
    let mut products = BTreeMap::new();
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_file()
            || path.extension().and_then(|ext| ext.to_str()) != Some("json")
            || path.file_name().and_then(|name| name.to_str()) == Some("products.json")
        {
            continue;
        }
        let key = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| fail("product filename is not valid UTF-8"))?;
        let data: Value = serde_json::from_slice(&fs::read(&path)?)?;
        let label = data
            .get("commodity_label")
            .and_then(Value::as_str)
            .or_else(|| (key == "daikon").then_some("だいこん"))
            .filter(|label| !label.trim().is_empty())
            .ok_or_else(|| fail(format!("{key}: product label missing")))?;
        let years = data
            .get("years")
            .and_then(Value::as_array)
            .ok_or_else(|| fail(format!("{key}: yearly data missing")))?;
        let year_numbers: Vec<_> = years
            .iter()
            .filter_map(|year| year.get("year").and_then(Value::as_u64))
            .collect();
        if data.get("schema_version").and_then(Value::as_u64) != Some(1)
            || data.get("commodity").and_then(Value::as_str) != Some(key)
            || !matches!(
                data.get("status").and_then(Value::as_str),
                Some("valid" | "incomplete")
            )
            || year_numbers != [2021, 2022, 2023]
        {
            return Err(fail(format!("{key}: generated product data is invalid")));
        }
        if products.insert(key.to_owned(), label.to_owned()).is_some() {
            return Err(fail(format!("duplicate product key: {key}")));
        }
    }
    if products.is_empty() {
        return Err(fail("no generated product files found"));
    }
    let index = ProductIndex {
        schema_version: 1,
        products: products
            .into_iter()
            .map(|(key, label)| Product { key, label })
            .collect(),
    };
    let output = output_dir.join("products.json");
    fs::write(&output, serde_json::to_vec_pretty(&index)?)?;
    Ok(output)
}
