//! Aggregation for product-specific, evidenced 2021–2023 tables.
//! Non-numeric source states remain visible as incomplete counts in the output.
use super::{
    metadata::{ClassValue, Inventory},
    normalization::HistoricalEvidence,
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const EXPECTED_ROWS: usize = 14_976;
const MAJOR_REGIONS: [&str; 9] = [
    "1001", "1002", "1004", "1006", "1007", "1008", "1009", "1010", "1011",
];
const SUBREGIONS: [&str; 3] = ["1003", "1005", "1012"];
const YEARS: [&str; 3] = ["2021", "2022", "2023"];

fn fail(message: impl Into<String>) -> Box<dyn Error> {
    message.into().into()
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Debug, Serialize)]
struct Contribution {
    consumer_region_code: String,
    consumer_region_name: String,
    source_position: usize,
    source_cat03_code: String,
    value_kind: String,
    raw_value: String,
}

#[derive(Debug, Serialize)]
struct PrefectureMonth {
    month: String,
    consignor_prefecture_code: String,
    consignor_prefecture_name: String,
    reported_value_sum: String,
    source_unit: String,
    status: String,
    reported_cells: usize,
    below_display_unit_cells: usize,
    no_fact_cells: usize,
    unknown_or_uncollected_cells: usize,
    contributions: Vec<Contribution>,
}

#[derive(Debug, Serialize)]
struct YearData {
    year: u16,
    table_id: String,
    metadata_sha256: String,
    temporal_mapping_sha256: String,
    normalized_input_sha256: String,
    source_unit: String,
    months: Vec<PrefectureMonth>,
}

#[derive(Debug, Serialize)]
struct AggregationCandidate {
    schema_version: u8,
    status: String,
    aggregation: String,
    coverage_warning: String,
    commodity: String,
    commodity_label: String,
    mapping_warnings: Vec<String>,
    years: Vec<YearData>,
}

#[derive(Debug)]
struct PeriodBlock {
    first: u32,
    last: u32,
}

fn class_values<'a>(
    inventory: &'a Inventory,
    axis_id: &str,
) -> Result<BTreeMap<&'a str, &'a ClassValue>> {
    let mut axes = inventory.axes.iter().filter(|axis| axis.id == axis_id);
    let axis = axes
        .next()
        .ok_or_else(|| fail(format!("{axis_id} metadata axis missing")))?;
    if axes.next().is_some() {
        return Err(fail(format!("duplicate {axis_id} metadata axis")));
    }
    let values: BTreeMap<_, _> = axis
        .values
        .iter()
        .map(|value| (value.code.as_str(), value))
        .collect();
    if values.len() != axis.values.len() {
        return Err(fail(format!("duplicate classification code in {axis_id}")));
    }
    Ok(values)
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| fail(format!("missing or invalid {key}")))
}

fn verify_mapping_blocks(
    mapping: &Value,
    year: &str,
    inventory: &Inventory,
) -> Result<BTreeMap<String, PeriodBlock>> {
    let year_mapping = mapping
        .get("years")
        .and_then(|years| years.get(year))
        .ok_or_else(|| fail(format!("temporal mapping missing {year}")))?;
    let periods = year_mapping
        .get("periods")
        .and_then(Value::as_array)
        .ok_or_else(|| fail(format!("{year}: temporal periods missing")))?;
    if periods.len() != 13 {
        return Err(fail(format!(
            "{year}: expected annual total plus twelve months"
        )));
    }
    let cat03 = class_values(inventory, "cat03")?;
    let mut blocks = BTreeMap::new();
    let mut total_found = false;
    let mut seen_blocks = BTreeSet::new();
    for period in periods {
        let name = required_string(period, "period")?;
        let block = period
            .get("block")
            .and_then(Value::as_u64)
            .ok_or_else(|| fail(format!("{year} {name}: source block is missing")))?;
        if !(1..=13).contains(&block) || !seen_blocks.insert(block) {
            return Err(fail(format!(
                "{year} {name}: source block is invalid or duplicated"
            )));
        }
        let exact_candidates: BTreeSet<_> = period
            .get("exact_match_candidates")
            .and_then(Value::as_array)
            .ok_or_else(|| fail(format!("{year} {name}: exact match evidence missing")))?
            .iter()
            .filter_map(Value::as_u64)
            .collect();
        if !exact_candidates.contains(&block)
            || !matches!(
                required_string(period, "block_identity")?,
                "unique" | "equivalent_candidates"
            )
        {
            return Err(fail(format!(
                "{year} {name}: source block is not supported by exact workbook matches"
            )));
        }
        let first = required_string(period, "cat03_first_code")?
            .parse::<u32>()
            .map_err(|_| fail("invalid first cat03 code"))?;
        let last = required_string(period, "cat03_last_code")?
            .parse::<u32>()
            .map_err(|_| fail("invalid last cat03 code"))?;
        let matched = period.get("matching_cells").and_then(Value::as_u64);
        let compared = period.get("compared_cells").and_then(Value::as_u64);
        if matched != Some(1152) || compared != Some(1152) || last != first + 11 {
            return Err(fail(format!(
                "{year} {name}: official workbook match is not complete"
            )));
        }
        let expected_first = 1001 + (block as u32 - 1) * 12;
        if first != expected_first {
            return Err(fail(format!(
                "{year} {name}: source code range differs from mapped block"
            )));
        }
        if name == "total" {
            if total_found {
                return Err(fail(format!("{year}: invalid annual summary block")));
            }
            total_found = true;
            continue;
        }
        let month: u32 = name
            .strip_prefix(&format!("{year}-"))
            .ok_or_else(|| fail(format!("invalid period label {name}")))?
            .parse()
            .map_err(|_| fail(format!("invalid period label {name}")))?;
        if !(1..=12).contains(&month) || last != first + 11 {
            return Err(fail(format!("{year} {name}: period/code block mismatch")));
        }
        for offset in 0..12 {
            let region_code = (1001 + offset).to_string();
            let source_code = (first + offset).to_string();
            let first_block = cat03
                .get(region_code.as_str())
                .ok_or("first region block incomplete")?;
            let current_block = cat03
                .get(source_code.as_str())
                .ok_or("monthly region block incomplete")?;
            if first_block.label != current_block.label || first_block.level != current_block.level
            {
                return Err(fail(format!(
                    "{year} {name}: repeated region metadata differs"
                )));
            }
        }
        if blocks
            .insert(name.to_owned(), PeriodBlock { first, last })
            .is_some()
        {
            return Err(fail(format!("{year}: duplicate period {name}")));
        }
    }
    if !total_found || blocks.len() != 12 || seen_blocks != (1..=13).collect() {
        return Err(fail(format!("{year}: incomplete annual/monthly mapping")));
    }
    Ok(blocks)
}

fn load_matching_inventory(
    metadata_dir: &Path,
    table_id: &str,
    metadata_sha: &str,
) -> Result<Inventory> {
    for entry in fs::read_dir(metadata_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir()
            || !entry.file_name().to_string_lossy().starts_with("metadata-")
        {
            continue;
        }
        let inventory_path = entry.path().join("inventory.json");
        if !inventory_path.is_file() {
            continue;
        }
        let inventory: Inventory = serde_json::from_slice(&fs::read(inventory_path)?)?;
        if inventory.stats_data_id == table_id && inventory.source_sha256 == metadata_sha {
            return Ok(inventory);
        }
    }
    Err(fail(format!(
        "matching metadata inventory not found for {table_id}"
    )))
}

fn verify_inventory(
    inventory: &Inventory,
    year: &str,
) -> Result<(BTreeMap<String, String>, BTreeMap<String, String>)> {
    let origins = class_values(inventory, "cat01")?;
    let measures = class_values(inventory, "cat02")?;
    let regions = class_values(inventory, "cat03")?;
    if origins.len() != 48 || measures.len() != 2 || regions.len() != 156 {
        return Err(fail(format!("{year}: unexpected metadata axis sizes")));
    }
    if origins.get("1001").is_none() || origins.get("1001").unwrap().label != "計" {
        return Err(fail(format!("{year}: origin summary code missing")));
    }
    let mut names = BTreeMap::new();
    for code in 1002..=1048 {
        let value = origins
            .get(code.to_string().as_str())
            .ok_or_else(|| fail(format!("{year}: origin code {code} missing")))?;
        names.insert(value.code.clone(), value.label.clone());
    }
    for code in MAJOR_REGIONS {
        let value = regions
            .get(code)
            .ok_or_else(|| fail(format!("{year}: consumer region {code} missing")))?;
        if value.level.as_deref() != Some("1") {
            return Err(fail(format!("{year}: region {code} is not level 1")));
        }
    }
    for code in SUBREGIONS {
        let value = regions
            .get(code)
            .ok_or_else(|| fail(format!("{year}: subregion {code} missing")))?;
        if value.level.as_deref() != Some("2") {
            return Err(fail(format!(
                "{year}: region {code} is not a subordinate region"
            )));
        }
    }
    let quantity = measures
        .get("1001")
        .ok_or_else(|| fail(format!("{year}: quantity code missing")))?;
    let price = measures
        .get("1002")
        .ok_or_else(|| fail(format!("{year}: price code missing")))?;
    let expected_quantity_unit = if year == "2021" { None } else { Some("t") };
    let expected_price_unit = if year == "2021" { None } else { Some("円/kg") };
    if quantity.unit.as_deref() != expected_quantity_unit
        || price.unit.as_deref() != expected_price_unit
    {
        return Err(fail(format!(
            "{year}: source measure units differ from the recorded contract"
        )));
    }
    let region_names = MAJOR_REGIONS
        .iter()
        .map(|code| ((*code).to_owned(), regions[*code].label.clone()))
        .collect();
    Ok((names, region_names))
}

fn add_numeric(total: &mut u128, lexical: &str) -> Result<()> {
    if lexical.is_empty()
        || !lexical.bytes().all(|byte| byte.is_ascii_digit())
        || lexical.starts_with('0')
    {
        return Err(fail(format!(
            "invalid positive integer source value {lexical}"
        )));
    }
    let value = lexical
        .parse::<u128>()
        .map_err(|_| fail("source value exceeds supported integer range"))?;
    if value == 0 {
        return Err(fail("reported quantity must be positive"));
    }
    *total = total
        .checked_add(value)
        .ok_or_else(|| fail("aggregate value overflow"))?;
    Ok(())
}

fn aggregate_year(
    year: &str,
    expected_table: &str,
    input_path: &Path,
    metadata_dir: &Path,
    mapping: &Value,
    mapping_sha: &str,
) -> Result<YearData> {
    let input_bytes = fs::read(input_path)?;
    let evidence: HistoricalEvidence = serde_json::from_slice(&input_bytes)?;
    if !matches!(evidence.status.as_str(), "valid" | "incomplete")
        || evidence.table_id != expected_table
        || evidence.input_count != EXPECTED_ROWS
        || evidence.rows.len() != EXPECTED_ROWS
    {
        return Err(fail(format!("{year}: unexpected or incomplete M5-B input")));
    }
    let year_mapping = mapping
        .get("years")
        .and_then(|years| years.get(year))
        .ok_or("year mapping missing")?;
    if required_string(year_mapping, "table_id")? != expected_table
        || required_string(year_mapping, "metadata_sha256")? != evidence.metadata_sha256
        || evidence.temporal_mapping_sha256 != mapping_sha
    {
        return Err(fail(format!(
            "{year}: mapping/table/metadata hash mismatch"
        )));
    }
    let inventory =
        load_matching_inventory(metadata_dir, expected_table, &evidence.metadata_sha256)?;
    let (origin_names, region_names) = verify_inventory(&inventory, year)?;
    let blocks = verify_mapping_blocks(mapping, year, &inventory)?;
    let origin_classes = class_values(&inventory, "cat01")?;
    let measure_classes = class_values(&inventory, "cat02")?;
    let mut seen_positions = BTreeSet::new();
    let mut seen_rows = BTreeSet::new();
    let mut seen_coordinates = BTreeSet::new();
    let mut buckets: BTreeMap<(String, String), BTreeMap<String, Contribution>> = BTreeMap::new();
    for row in &evidence.rows {
        if row.source_table_id != expected_table
            || row.source_position == 0
            || row.source_position > EXPECTED_ROWS
            || !seen_positions.insert(row.source_position)
        {
            return Err(fail(format!(
                "{year}: source row identity/position invalid"
            )));
        }
        let origin = row.source_attributes.get("@cat01").ok_or("cat01 missing")?;
        let measure = row.source_attributes.get("@cat02").ok_or("cat02 missing")?;
        let source_region = row.source_attributes.get("@cat03").ok_or("cat03 missing")?;
        if row.source_cat03_code != *source_region
            || !origin_classes.contains_key(origin.as_str())
            || !measure_classes.contains_key(measure.as_str())
        {
            return Err(fail(format!("{year}: source row codes mismatch")));
        }
        let expected_unit = measure_classes[measure.as_str()].unit.clone();
        if row.source_unit != expected_unit {
            return Err(fail(format!("{year}: row unit differs from metadata")));
        }
        if !seen_rows.insert((
            row.period.clone(),
            origin.clone(),
            measure.clone(),
            source_region.clone(),
        )) {
            return Err(fail(format!(
                "{year}: duplicate normalized source coordinate"
            )));
        }
        if row.annual_total {
            if row.period != "total"
                || !(1001..=1012).contains(
                    &source_region
                        .parse::<u32>()
                        .map_err(|_| fail("invalid source cat03"))?,
                )
            {
                return Err(fail(format!("{year}: annual summary identity mismatch")));
            }
            continue;
        }
        let block = blocks
            .get(&row.period)
            .ok_or_else(|| fail(format!("{year}: unmapped period {}", row.period)))?;
        let source_region_number = source_region
            .parse::<u32>()
            .map_err(|_| fail("invalid source cat03 code"))?;
        if source_region_number < block.first || source_region_number > block.last {
            return Err(fail(format!(
                "{year}: source region does not match mapped period"
            )));
        }
        let expected_region = (1001 + ((source_region_number - 1001) % 12)).to_string();
        if row.consumer_region_category_code != expected_region {
            return Err(fail(format!("{year}: normalized region code mismatch")));
        }
        let origin_code = row.consignor_category_code.as_str();
        if origin_code != origin || measure != "1001" {
            continue;
        }
        if origin_code == "1001" {
            continue;
        }
        if !origin_names.contains_key(origin_code) {
            return Err(fail(format!(
                "{year}: unmapped consignor category {origin_code}"
            )));
        }
        if !seen_coordinates.insert((
            row.period.clone(),
            origin_code.to_owned(),
            source_region.clone(),
        )) {
            return Err(fail(format!("{year}: duplicate quantity coordinate")));
        }
        if !MAJOR_REGIONS.contains(&expected_region.as_str()) {
            continue;
        }
        let value_kind = match (row.value_kind.as_str(), row.raw_value.as_str()) {
            ("reported", raw) => {
                let mut parsed = 0;
                add_numeric(&mut parsed, raw)?;
                "reported"
            }
            ("below_display_unit", "0") | ("reported_zero_symbol_unspecified", "0") => {
                "below_display_unit"
            }
            ("no_fact", "-") => "no_fact",
            ("unknown_or_uncollected", "…") => "unknown_or_uncollected",
            _ => {
                return Err(fail(format!(
                    "{year}: unsupported quantity symbol/value-kind pair"
                )));
            }
        };
        let region_index = (source_region_number - block.first) as usize;
        let region_code = (1001 + region_index as u32).to_string();
        let contribution = Contribution {
            consumer_region_code: region_code.clone(),
            consumer_region_name: region_names[&region_code].clone(),
            source_position: row.source_position,
            source_cat03_code: source_region.clone(),
            value_kind: value_kind.to_owned(),
            raw_value: row.raw_value.clone(),
        };
        let key = (row.period.clone(), origin_code.to_owned());
        if buckets
            .entry(key)
            .or_default()
            .insert(region_code, contribution)
            .is_some()
        {
            return Err(fail(format!("{year}: duplicate major-region contribution")));
        }
    }
    if seen_positions.len() != EXPECTED_ROWS
        || seen_rows.len() != EXPECTED_ROWS
        || seen_coordinates.len() != 12 * 47 * 12
    {
        return Err(fail(format!(
            "{year}: source coordinate grid is incomplete"
        )));
    }
    // The 2021 API metadata omits units. All 39 official annual/month workbooks label
    // quantity values as tonnes, so use that evidenced display unit for every year.
    let source_unit = "t".to_owned();
    let mut months = Vec::with_capacity(12 * 47);
    for month in 1..=12 {
        let period = format!("{year}-{month:02}");
        for origin_code in 1002..=1048 {
            let origin_code = origin_code.to_string();
            let mut cells: Vec<_> = buckets
                .remove(&(period.clone(), origin_code.clone()))
                .ok_or_else(|| fail(format!("{year} {period}: origin bucket missing")))?
                .into_values()
                .collect();
            cells.sort_by(|a, b| a.consumer_region_code.cmp(&b.consumer_region_code));
            if cells.len() != 9
                || cells
                    .iter()
                    .map(|cell| cell.consumer_region_code.as_str())
                    .collect::<BTreeSet<_>>()
                    != MAJOR_REGIONS.into_iter().collect::<BTreeSet<_>>()
            {
                return Err(fail(format!(
                    "{year} {period} origin {origin_code}: expected nine major-region cells"
                )));
            }
            let mut sum = 0u128;
            let mut reported = 0;
            let mut below_display_unit = 0;
            let mut no_fact = 0;
            let mut unknown_or_uncollected = 0;
            for cell in &cells {
                match cell.value_kind.as_str() {
                    "reported" => {
                        add_numeric(&mut sum, &cell.raw_value)?;
                        reported += 1;
                    }
                    "below_display_unit" => below_display_unit += 1,
                    "no_fact" => no_fact += 1,
                    "unknown_or_uncollected" => unknown_or_uncollected += 1,
                    _ => return Err(fail("unknown contribution value kind")),
                }
            }
            months.push(PrefectureMonth {
                month: period.clone(),
                consignor_prefecture_code: origin_code.clone(),
                consignor_prefecture_name: origin_names[&origin_code].clone(),
                reported_value_sum: sum.to_string(),
                source_unit: source_unit.clone(),
                status: if reported == 9
                    && below_display_unit == 0
                    && no_fact == 0
                    && unknown_or_uncollected == 0
                {
                    "valid"
                } else {
                    "incomplete"
                }
                .into(),
                reported_cells: reported,
                below_display_unit_cells: below_display_unit,
                no_fact_cells: no_fact,
                unknown_or_uncollected_cells: unknown_or_uncollected,
                contributions: cells,
            });
        }
    }
    if !buckets.is_empty() {
        return Err(fail(format!(
            "{year}: unexpected leftover aggregation buckets"
        )));
    }
    Ok(YearData {
        year: year.parse()?,
        table_id: expected_table.to_owned(),
        metadata_sha256: evidence.metadata_sha256,
        temporal_mapping_sha256: evidence.temporal_mapping_sha256,
        normalized_input_sha256: hash(&input_bytes),
        source_unit,
        months,
    })
}

pub fn run_2021_2023(
    input_2021: &Path,
    input_2022: &Path,
    input_2023: &Path,
    metadata_dir: &Path,
    mapping_path: &Path,
    output_path: &Path,
) -> Result<PathBuf> {
    run_2021_2023_product(
        input_2021,
        input_2022,
        input_2023,
        metadata_dir,
        mapping_path,
        output_path,
        "daikon",
        "だいこん",
    )
}

/// Aggregate a product whose three table IDs and workbook matches are explicitly
/// recorded in the temporal mapping. Product identity is supplied by the caller;
/// table IDs are always read from and checked against that mapping.
pub fn run_2021_2023_product(
    input_2021: &Path,
    input_2022: &Path,
    input_2023: &Path,
    metadata_dir: &Path,
    mapping_path: &Path,
    output_path: &Path,
    commodity: &str,
    commodity_label: &str,
) -> Result<PathBuf> {
    if commodity.is_empty()
        || !commodity.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
        || commodity_label.trim().is_empty()
    {
        return Err(fail("invalid product key or label"));
    }
    let mapping_bytes = fs::read(mapping_path)?;
    let mapping: Value = serde_json::from_slice(&mapping_bytes)?;
    if !matches!(
        mapping.get("status").and_then(Value::as_str),
        Some("VALUE_MATCH_VERIFIED" | "VALUE_MATCH_VERIFIED_WITH_EQUIVALENT_BLOCKS")
    ) {
        return Err(fail(
            "temporal mapping is not verified exact-match evidence",
        ));
    }
    let mapping_sha = hash(&mapping_bytes);
    let mut mapping_warnings = Vec::new();
    for year in YEARS {
        let ambiguous_periods: Vec<_> = mapping
            .get("years")
            .and_then(|entries| entries.get(year))
            .and_then(|entry| entry.get("periods"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|period| {
                period.get("block_identity").and_then(Value::as_str)
                    == Some("equivalent_candidates")
            })
            .filter_map(|period| period.get("period").and_then(Value::as_str))
            .collect();
        if !ambiguous_periods.is_empty() {
            mapping_warnings.push(format!(
                "{} {}: {} period(s) have equivalent exact-match cat03 blocks; the displayed quantities are identical across those candidate blocks.",
                commodity_label.trim(),
                year,
                ambiguous_periods.len()
            ));
        }
    }
    let input_paths = [input_2021, input_2022, input_2023];
    let mut years = Vec::with_capacity(YEARS.len());
    for (year, input_path) in YEARS.iter().zip(input_paths) {
        let table_id = mapping
            .get("years")
            .and_then(|entries| entries.get(*year))
            .and_then(|entry| entry.get("table_id"))
            .and_then(Value::as_str)
            .filter(|table_id| !table_id.is_empty())
            .ok_or_else(|| fail(format!("temporal mapping lacks a table ID for {year}")))?;
        years.push(aggregate_year(
            year,
            table_id,
            input_path,
            metadata_dir,
            &mapping,
            &mapping_sha,
        )?);
    }
    let candidate = AggregationCandidate {
        schema_version: 1,
        status: if years.iter().flat_map(|year| &year.months).all(|month| month.status == "valid") {
            "valid"
        } else {
            "incomplete"
        }
        .into(),
        aggregation: "sum numeric reported quantity values across nine top-level major consumer regions; exclude below-display-unit, no-fact, and unknown/uncollected values".into(),
        coverage_warning: "Sums omit below-display-unit, no-fact, and unknown/uncollected cells. They represent wholesale flows into selected major consumer regions, not total production or a national quantity. Check year-specific survey scope before comparing years.".into(),
        commodity: commodity.into(),
        commodity_label: commodity_label.trim().into(),
        mapping_warnings,
        years,
    };
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output_path)?;
    file.write_all(&serde_json::to_vec(&candidate)?)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(output_path.to_path_buf())
}
