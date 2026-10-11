//! Build the small runtime catalogue only from complete schema-2 product splits.
use super::aggregation::ensure_split_from_legacy;
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const REGIONS: [&str; 9] = [
    "1001", "1002", "1004", "1006", "1007", "1008", "1009", "1010", "1011",
];

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

fn string<'a>(data: &'a Value, key: &str) -> Result<&'a str> {
    data.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| fail(format!("missing or invalid {key}")))
}

fn count(data: &Value, key: &str) -> Result<u64> {
    data.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| fail(format!("missing or invalid {key}")))
}

// here
fn validate_split(output_dir: &Path, key: &str, data: &Value) -> Result<()> {
    let years = data
        .get("years")
        .and_then(Value::as_array)
        .ok_or_else(|| fail(format!("{key}: yearly data missing")))?;
    if count(data, "schema_version")? != 2
        || string(data, "commodity")? != key
        || !matches!(string(data, "status")?, "valid" | "incomplete")
        || years.len() != 3
        || !data.get("mapping_warnings").is_some_and(Value::is_array)
    {
        return Err(fail(format!("{key}: invalid summary contract")));
    }

    let mut by_prefecture: BTreeMap<String, BTreeMap<String, &Value>> = BTreeMap::new();
    let mut incomplete = false;
    for (year_index, year) in years.iter().enumerate() {
        let year_number = 2021 + year_index;
        if count(year, "year")? != year_number as u64
            || string(year, "table_id")?.is_empty()
            || string(year, "source_unit")? != "t"
        {
            return Err(fail(format!("{key}: invalid yearly metadata")));
        }
        let rows = year
            .get("months")
            .and_then(Value::as_array)
            .ok_or_else(|| fail("summary months missing"))?;
        if rows.len() != 12 * 47 {
            return Err(fail(format!("{key}: expected 564 summary rows per year")));
        }
        for row in rows {
            let code = string(row, "consignor_prefecture_code")?;
            let month = string(row, "month")?;
            if !matches!(code.parse::<u32>(), Ok(1002..=1048))
                || !((1..=12)
                    .map(|m| format!("{year_number}-{m:02}"))
                    .any(|m| m == month))
                || string(row, "consignor_prefecture_name")?.is_empty()
                || string(row, "source_unit")? != "t"
                || !matches!(string(row, "status")?, "valid" | "incomplete")
            {
                return Err(fail(format!("{key}: invalid summary coordinate")));
            }
            let cells = count(row, "reported_cells")?
                + count(row, "below_display_unit_cells")?
                + count(row, "no_fact_cells")?
                + count(row, "unknown_or_uncollected_cells")?;
            let expected_status = if count(row, "reported_cells")? == 9 {
                "valid"
            } else {
                incomplete = true;
                "incomplete"
            };
            if cells != 9
                || string(row, "status")? != expected_status
                || string(row, "reported_value_sum")?.parse::<u128>().is_err()
                || by_prefecture
                    .entry(code.to_owned())
                    .or_default()
                    .insert(month.to_owned(), row)
                    .is_some()
            {
                return Err(fail(format!("{key}: duplicate or invalid summary row")));
            }
        }
    }
    let expected_overall_status = if incomplete { "incomplete" } else { "valid" };
    if string(data, "status")? != expected_overall_status {
        return Err(fail(format!("{key}: overall summary status mismatch")));
    }
    if by_prefecture.len() != 47 {
        return Err(fail(format!("{key}: missing summary prefectures")));
    }

    let detail_dir = output_dir.join(key).join("details");
    let files: BTreeSet<_> = fs::read_dir(&detail_dir)?
        .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
        .collect::<std::result::Result<_, _>>()?;
    let expected_files: BTreeSet<_> = (1002..=1048).map(|code| format!("{code}.json")).collect();
    if files != expected_files {
        return Err(fail(format!(
            "{key}: incomplete or unexpected detail files"
        )));
    }

    for (code, summaries) in by_prefecture {
        if summaries.len() != 36 {
            return Err(fail(format!("{key}/{code}: expected 36 summary months")));
        }
        let detail: Value =
            serde_json::from_slice(&fs::read(detail_dir.join(format!("{code}.json")))?)?;
        if count(&detail, "schema_version")? != 2
            || string(&detail, "commodity")? != key
            || string(&detail, "consignor_prefecture_code")? != code
        {
            return Err(fail(format!("{key}/{code}: detail identity mismatch")));
        }
        let months = detail
            .get("months")
            .and_then(Value::as_array)
            .ok_or_else(|| fail("detail months missing"))?;
        if months.len() != 36 {
            return Err(fail(format!("{key}/{code}: incomplete detail months")));
        }
        let mut seen = BTreeSet::new();
        for month in months {
            let period = string(month, "month")?;
            if !seen.insert(period) || !summaries.contains_key(period) {
                return Err(fail(format!(
                    "{key}/{code}: duplicate or missing detail period"
                )));
            }
            let contributions = month
                .get("contributions")
                .and_then(Value::as_array)
                .ok_or_else(|| fail("detail contributions missing"))?;
            if contributions.len() != 9 {
                return Err(fail(format!(
                    "{key}/{code}/{period}: expected nine contributions"
                )));
            }
            let mut regions = BTreeSet::new();
            let mut sum = 0u128;
            let mut counts = [0u64; 4];
            for cell in contributions {
                let region = string(cell, "consumer_region_code")?;
                if !REGIONS.contains(&region)
                    || !regions.insert(region)
                    || string(cell, "consumer_region_name")?.is_empty()
                {
                    return Err(fail(format!("{key}/{code}/{period}: invalid region")));
                }
                let raw = string(cell, "raw_value")?;
                match (string(cell, "value_kind")?, raw) {
                    ("reported", value) => {
                        if !value.bytes().all(|b| b.is_ascii_digit()) || value.starts_with('0') {
                            return Err(fail("invalid reported contribution"));
                        }
                        let amount = value.parse::<u128>()?;
                        sum = sum
                            .checked_add(amount)
                            .ok_or("contribution total overflow")?;
                        counts[0] += 1;
                    }
                    ("below_display_unit", "0") => counts[1] += 1,
                    ("no_fact", "-") => counts[2] += 1,
                    ("unknown_or_uncollected", "…") => counts[3] += 1,
                    _ => return Err(fail("invalid contribution symbol")),
                }
            }
            let summary = summaries[period];
            if string(summary, "reported_value_sum")? != sum.to_string()
                || count(summary, "reported_cells")? != counts[0]
                || count(summary, "below_display_unit_cells")? != counts[1]
                || count(summary, "no_fact_cells")? != counts[2]
                || count(summary, "unknown_or_uncollected_cells")? != counts[3]
            {
                return Err(fail(format!(
                    "{key}/{code}/{period}: summary/detail mismatch"
                )));
            }
        }
    }

    // Audit retains the original source metadata hashes and row coordinates.
    let audit: Value = serde_json::from_slice(&fs::read(output_dir.join(key).join("audit.json"))?)?;
    if count(&audit, "schema_version")? != 1
        || string(&audit, "commodity")? != key
        || audit
            .get("years")
            .and_then(Value::as_array)
            .map_or(true, |years| years.len() != 3)
    {
        return Err(fail(format!("{key}: missing or invalid audit artifact")));
    }
    for (summary_year, audit_year) in years.iter().zip(
        audit
            .get("years")
            .and_then(Value::as_array)
            .ok_or("audit years missing")?,
    ) {
        if count(summary_year, "year")? != count(audit_year, "year")?
            || string(summary_year, "table_id")? != string(audit_year, "table_id")?
            || ![
                "metadata_sha256",
                "temporal_mapping_sha256",
                "normalized_input_sha256",
            ]
            .iter()
            .all(|field| {
                audit_year
                    .get(*field)
                    .and_then(Value::as_str)
                    .is_some_and(|value| {
                        value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
                    })
            })
            || audit_year
                .get("months")
                .and_then(Value::as_array)
                .map_or(true, |months| months.len() != 564)
        {
            return Err(fail(format!("{key}: audit provenance is incomplete")));
        }
    }
    Ok(())
}

pub fn generate(output_dir: &Path) -> Result<PathBuf> {
    // Existing v1 files may be reused by the upstream generation pipeline.
    // Materialize missing sidecars from them before validating the v2 index.
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.ends_with(".json") || name.ends_with(".summary.json") || name == "products.json" {
            continue;
        }
        let key = &name[..name.len() - ".json".len()];
        if !output_dir.join(format!("{key}.summary.json")).exists() {
            ensure_split_from_legacy(&entry.path())?;
        }
    }
    let mut products = BTreeMap::new();
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_file() {
            continue;
        }
        // Versioned summaries are discovered independently of the legacy
        // <key>.json files, which may coexist during the deployment window.
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| fail("product filename is not valid UTF-8"))?;
        let Some(key) = filename.strip_suffix(".summary.json") else {
            continue;
        };
        if key.is_empty()
            || !key
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
        {
            return Err(fail(format!("invalid product filename: {key}")));
        }
        let data: Value = serde_json::from_slice(&fs::read(&path)?)?;
        let label = string(&data, "commodity_label")?.trim();
        if label.is_empty() {
            return Err(fail(format!("{key}: product label missing")));
        }
        validate_split(output_dir, key, &data)?;
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
    // Never replace the index when any product's summary/detail split is invalid.
    let temp = output_dir.join(format!("products.json.tmp-{}", std::process::id()));
    let result = (|| -> Result<()> {
        fs::write(&temp, serde_json::to_vec_pretty(&index)?)?;
        fs::rename(&temp, &output)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result?;
    Ok(output)
}
