//! M5-A: offline, fail-closed normalization of the 2024 daikon raw evidence.
//! This module creates NON-PUBLISHABLE inspection artifacts only.
use super::acquisition::{self, SourceReport, SourceRow};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const TABLE: &str = "0004044496";
const EXPECTED_ROWS: usize = 14_976;
fn fail(message: impl Into<String>) -> Box<dyn Error> {
    message.into().into()
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// An externally reviewed, explicit source-code contract. Code lists must be complete,
/// including excluded/summary codes; no source code is inferred from ordinal position.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Codebook {
    pub format_version: u32,
    pub table_id: String,
    pub metadata_sha256: String,
    pub axes: BTreeMap<String, AxisMapping>,
    /// Must be EXACTLY the source axes used in observations (e.g. @cat01, @cat02, @cat03, @time).
    pub origin_axis: String,
    pub market_axis: String,
    pub time_axis: String,
    pub measure_axis: String,
    pub quantity_measure_code: String,
    pub quantity_unit: String,
    /// 13 explicit entries: 12 source month codes -> 1..=12, aggregate -> null.
    pub months: BTreeMap<String, Option<u8>>,
    /// Every origin code classified explicitly. `origin` is consignor, not production prefecture.
    pub origins: BTreeMap<String, OriginMapping>,
    /// Every market code classified explicitly; subordinate regions must never be summed with parents.
    pub markets: BTreeMap<String, MarketMapping>,
    /// Evidence for the mapping, recorded for review; not an approval flag.
    pub evidence: Vec<String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AxisMapping {
    pub codes: Vec<String>,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum OriginMapping {
    ConsignorPrefecture { prefecture_code: String },
    Summary,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MarketMapping {
    Major { name: String },
    Subset { name: String, parent_code: String },
    Summary { name: String },
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Quantity {
    Reported { tonnes_lexical: String },
    BelowDisplayUnit,
    NoFact,
}
#[derive(Debug, Serialize)]
pub struct NormalizedRow {
    pub source_table_id: String,
    pub source_position: usize,
    pub source_attributes: BTreeMap<String, String>,
    pub month: u8,
    pub consignor_prefecture_code: String,
    pub major_consumer_region_code: String,
    pub quantity: Quantity,
    pub source_unit: String,
}
#[derive(Debug, Serialize)]
pub struct ValidationReport {
    pub status: String,
    pub publishable: bool,
    pub table_id: String,
    pub metadata_sha256: String,
    pub codebook_sha256: String,
    pub input_count: usize,
    pub normalized_count: usize,
    pub skipped_aggregate_time: usize,
    pub skipped_non_quantity: usize,
    pub skipped_summary_origins: usize,
    pub skipped_subset_or_summary_markets: usize,
    pub reported_count: usize,
    pub below_display_unit_count: usize,
    pub no_fact_count: usize,
    pub outstanding_gates: Vec<String>,
}
#[derive(Debug, Serialize)]
pub struct NormalizedEvidence {
    pub validation: ValidationReport,
    pub rows: Vec<NormalizedRow>,
}

/// Normalized 2021–2023 rows retain source block codes, period evidence, and lexical values.
/// `status` distinguishes fully reported input from input with non-numeric source states.
#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalRow {
    pub source_table_id: String,
    pub source_position: usize,
    pub source_attributes: BTreeMap<String, String>,
    /// Original `cat03`: encodes both the temporal block and region position.
    pub source_cat03_code: String,
    pub period: String,
    pub annual_total: bool,
    pub consignor_category_code: String,
    /// Region classification mapped to 1001..1012 by position within each 12-code block.
    pub consumer_region_category_code: String,
    pub raw_value: String,
    pub source_unit: Option<String>,
    pub value_kind: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalEvidence {
    pub status: String,
    pub table_id: String,
    pub metadata_sha256: String,
    pub temporal_mapping_sha256: String,
    pub input_count: usize,
    pub monthly_count: usize,
    pub annual_summary_count: usize,
    pub missing_unit_count: usize,
    pub rows: Vec<HistoricalRow>,
}

fn complete_keys<T>(
    expected: &BTreeSet<String>,
    mapped: &BTreeMap<String, T>,
    label: &str,
) -> Result<()> {
    if expected != &mapped.keys().cloned().collect() {
        return Err(fail(format!("incomplete or extraneous {label} mappings")));
    }
    Ok(())
}
fn validate_book(book: &Codebook, source: &SourceReport) -> Result<()> {
    if book.format_version != 1
        || book.table_id != TABLE
        || source.table_id != TABLE
        || book.metadata_sha256 != source.source_metadata_sha256
        || source.gate != "RAW_ONLY_NOT_PUBLISHABLE"
        || source.count != EXPECTED_ROWS
        || source.rows.len() != EXPECTED_ROWS
        || source.pages != 15
        || book.quantity_unit != "t"
        || book.evidence.is_empty()
    {
        return Err(fail(
            "2024 source identity, provenance, count, unit or codebook contract mismatch",
        ));
    }
    let dimensions = [
        &book.origin_axis,
        &book.market_axis,
        &book.time_axis,
        &book.measure_axis,
    ];
    if dimensions.iter().any(|a| !a.starts_with('@'))
        || dimensions.iter().collect::<BTreeSet<_>>().len() != 4
        || book.axes.len() != 4
        || dimensions.iter().any(|a| !book.axes.contains_key(*a))
    {
        return Err(fail("missing or duplicated required dimension"));
    }
    for (axis, mapping) in &book.axes {
        let codes: BTreeSet<_> = mapping.codes.iter().collect();
        if mapping.codes.is_empty()
            || codes.len() != mapping.codes.len()
            || mapping.codes.iter().any(|c| c.is_empty())
        {
            return Err(fail(format!("empty or duplicate codes on {axis}")));
        }
    }
    let keys = |axis: &str| -> BTreeSet<String> { book.axes[axis].codes.iter().cloned().collect() };
    complete_keys(&keys(&book.origin_axis), &book.origins, "origins")?;
    complete_keys(&keys(&book.market_axis), &book.markets, "markets")?;
    complete_keys(&keys(&book.time_axis), &book.months, "months")?;
    if !keys(&book.measure_axis).contains(&book.quantity_measure_code) {
        return Err(fail("quantity code is not on measure axis"));
    }
    if book.months.len() != 13
        || book
            .months
            .values()
            .filter_map(|m| *m)
            .collect::<BTreeSet<_>>()
            != (1..=12).collect()
        || book.months.values().filter(|m| m.is_none()).count() != 1
    {
        return Err(fail(
            "months must uniquely identify January through December",
        ));
    }
    let mut prefectures = BTreeSet::new();
    for origin in book.origins.values() {
        if let OriginMapping::ConsignorPrefecture { prefecture_code } = origin {
            if prefecture_code.is_empty() || !prefectures.insert(prefecture_code) {
                return Err(fail("duplicate or empty prefecture mapping"));
            }
        }
    }
    if prefectures.len() != 47 {
        return Err(fail("expected exactly 47 consignor prefectures"));
    }
    let mut major = 0;
    let mut subsets = 0;
    for (code, market) in &book.markets {
        match market {
            MarketMapping::Major { name } => {
                major += 1;
                if name.is_empty() {
                    return Err(fail("empty market name"));
                }
            }
            MarketMapping::Subset { name, parent_code } => {
                subsets += 1;
                if name.is_empty()
                    || code == parent_code
                    || !matches!(
                        book.markets.get(parent_code),
                        Some(MarketMapping::Major { .. })
                    )
                {
                    return Err(fail("subset must reference major parent"));
                }
            }
            MarketMapping::Summary { name } => {
                if name.is_empty() {
                    return Err(fail("empty summary name"));
                }
            }
        }
    }
    if major != 9 || subsets != 3 {
        return Err(fail("expected nine major regions and three subsets"));
    }
    let grid: usize = book.axes.values().map(|a| a.codes.len()).product();
    if grid != EXPECTED_ROWS {
        return Err(fail("source-coordinate grid size mismatch"));
    }
    Ok(())
}
fn classify_value(s: &str) -> Result<Quantity> {
    if s == "-" {
        return Ok(Quantity::NoFact);
    }
    if s == "0" {
        return Ok(Quantity::BelowDisplayUnit);
    }
    // Strict non-negative decimal syntax, avoiding floating-point transformations.
    let valid = !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_digit() || b == b'.' || b == b',')
        && s.matches('.').count() <= 1
        && !s.starts_with('.')
        && !s.ends_with('.')
        && s.split('.').all(|part| !part.is_empty())
        && {
            let groups: Vec<_> = s.split('.').next().unwrap().split(',').collect();
            groups.len() == 1 && groups[0].bytes().all(|b| b.is_ascii_digit())
                || groups.len() > 1
                    && (1..=3).contains(&groups[0].len())
                    && groups[0].bytes().all(|b| b.is_ascii_digit())
                    && groups[1..]
                        .iter()
                        .all(|g| g.len() == 3 && g.bytes().all(|b| b.is_ascii_digit()))
        };
    if !valid
        || s.replace(',', "")
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite() && *v > 0.0)
            .is_none()
    {
        return Err(fail("unknown, negative or malformed source quantity"));
    }
    Ok(Quantity::Reported {
        tonnes_lexical: s.to_owned(),
    })
}
fn coordinate<'a>(row: &'a SourceRow, axis: &str) -> Result<&'a str> {
    row.attributes
        .get(axis)
        .map(String::as_str)
        .ok_or_else(|| fail(format!("missing axis {axis} at {}", row.position)))
}

/// Apply only mappings supported by the independently matched workbook manifest.
/// The manifest itself remains pending reviewer approval and output cannot publish.
pub fn normalize_2021_2023(
    source: SourceReport,
    mapping_path: &Path,
) -> Result<HistoricalEvidence> {
    use serde_json::Value;
    let raw_mapping = fs::read(mapping_path)?;
    let mapping: Value = serde_json::from_slice(&raw_mapping)?;
    let year = match source.table_id.as_str() {
        "0002008540" => "2021",
        "0002120386" => "2022",
        "0002114330" => "2023",
        _ => return Err(fail("unsupported historical table")),
    };
    let year_entry = mapping
        .get("years")
        .and_then(|v| v.get(year))
        .ok_or("missing temporal mapping year")?;
    if year_entry.get("table_id").and_then(Value::as_str) != Some(&source.table_id)
        || year_entry.get("metadata_sha256").and_then(Value::as_str)
            != Some(&source.source_metadata_sha256)
        || source.gate != "RAW_ONLY_NOT_PUBLISHABLE"
        || source.count != 14_976
        || source.rows.len() != 14_976
    {
        return Err(fail(
            "historical source identity, metadata binding, or row count mismatch",
        ));
    }
    let periods = year_entry
        .get("periods")
        .and_then(Value::as_array)
        .ok_or("missing period evidence")?;
    if periods.len() != 13 {
        return Err(fail("expected annual summary and 12 monthly mappings"));
    }
    let mut block_period = BTreeMap::new();
    let mut annual_blocks = BTreeSet::new();
    for period in periods {
        let block = period
            .get("block")
            .and_then(Value::as_u64)
            .ok_or("invalid block number")? as usize;
        let label = period
            .get("period")
            .and_then(Value::as_str)
            .ok_or("invalid period label")?;
        let code_first = period
            .get("cat03_first_code")
            .and_then(Value::as_str)
            .ok_or("missing first code")?;
        let code_last = period
            .get("cat03_last_code")
            .and_then(Value::as_str)
            .ok_or("missing last code")?;
        if period.get("matching_cells").and_then(Value::as_u64) != Some(1152)
            || period.get("compared_cells").and_then(Value::as_u64) != Some(1152)
            || period
                .get("spreadsheet_sha256")
                .and_then(Value::as_str)
                .map_or(true, |s| s.len() != 64)
            || period
                .get("stat_inf_id")
                .and_then(Value::as_str)
                .map_or(true, str::is_empty)
        {
            return Err(fail("period lacks exact official workbook-match evidence"));
        }
        let expected_first = 989 + block * 12;
        let expected_last = expected_first + 11;
        if code_first != expected_first.to_string()
            || code_last != expected_last.to_string()
            || block < 1
            || block > 13
            || block_period.insert(block, label.to_owned()).is_some()
        {
            return Err(fail("period/block code mapping invalid or duplicated"));
        }
        if label == "total" {
            annual_blocks.insert(block);
        } else if label != format!("{year}-{:02}", block - 1) || !(2..=13).contains(&block) {
            return Err(fail("monthly period mapping invalid"));
        }
    }
    if block_period.len() != 13 || annual_blocks.len() != 1 || !annual_blocks.contains(&1) {
        return Err(fail(
            "mapping must contain block 1 annual summary and 12 months",
        ));
    }
    let expected_meta = year_entry
        .get("metadata_sha256")
        .and_then(Value::as_str)
        .unwrap();
    let mut seen = BTreeSet::new();
    let mut rows = Vec::with_capacity(source.rows.len());
    let mut monthly_count = 0;
    let mut annual_summary_count = 0;
    let mut missing_unit_count = 0;
    for row in source.rows {
        let origin = coordinate(&row, "@cat01")?.to_owned();
        let measure = coordinate(&row, "@cat02")?;
        let source_cat03_code = coordinate(&row, "@cat03")?.to_owned();
        let category = source_cat03_code
            .parse::<usize>()
            .map_err(|_| fail("invalid cat03 code"))?;
        if !(1001..=1048).contains(
            &origin
                .parse::<usize>()
                .map_err(|_| fail("invalid cat01 code"))?,
        ) || !(1001..=1002).contains(
            &measure
                .parse::<usize>()
                .map_err(|_| fail("invalid cat02 code"))?,
        ) || !(1001..=1156).contains(&category)
            || row.reporting_month.is_some()
            || row.annual_total.is_some()
        {
            return Err(fail(
                "historical observation has unexpected codes or inferred semantics",
            ));
        }
        let block = (category - 1001) / 12 + 1;
        // Verified workbooks show that each temporal block repeats the same 12 region
        // positions. Map position 1..=12 to the region codes 1001..=1012, while retaining
        // the complete temporal-plus-region source code in `source_cat03_code`.
        let consumer_region_code = 1001 + (category - 1001) % 12;
        let period = block_period
            .get(&block)
            .ok_or("unmapped cat03 code")?
            .clone();
        let key = (origin, measure.to_owned(), category);
        if !seen.insert(key.clone()) {
            return Err(fail("duplicate historical observation coordinate"));
        }
        let annual = period == "total";
        if annual {
            annual_summary_count += 1;
        } else {
            monthly_count += 1;
        }
        if row.unit.is_none() {
            missing_unit_count += 1;
        }
        let value_kind = match row.raw_value.as_str() {
            "-" => "no_fact",
            "…" => "unknown_or_uncollected",
            "0" => "below_display_unit",
            _ => "reported",
        }
        .to_owned();
        rows.push(HistoricalRow {
            source_table_id: row.table_id,
            source_position: row.position,
            source_attributes: row.attributes,
            source_cat03_code,
            period,
            annual_total: annual,
            consignor_category_code: key.0,
            consumer_region_category_code: consumer_region_code.to_string(),
            raw_value: row.raw_value,
            source_unit: row.unit,
            value_kind,
        });
    }
    if seen.len() != 14_976
        || monthly_count != 12 * 1152
        || annual_summary_count != 1152
        || expected_meta != source.source_metadata_sha256
    {
        return Err(fail("historical grid or period coverage mismatch"));
    }
    let status = if rows.iter().all(|row| row.value_kind == "reported") {
        "valid"
    } else {
        "incomplete"
    };
    Ok(HistoricalEvidence {
        status: status.into(),
        table_id: source.table_id,
        metadata_sha256: source.source_metadata_sha256,
        temporal_mapping_sha256: hash(&raw_mapping),
        input_count: 14_976,
        monthly_count,
        annual_summary_count,
        missing_unit_count,
        rows,
    })
}

pub fn run_2021_2023(
    acquisition_run: &Path,
    metadata_dir: &Path,
    mapping_file: &Path,
    output: &Path,
) -> Result<PathBuf> {
    let source = acquisition::replay(acquisition_run, metadata_dir)?;
    let normalized = normalize_2021_2023(source, mapping_file)?;
    let path = output.join("m5b_inspection_only.json");
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)?;
    f.write_all(&serde_json::to_vec_pretty(&normalized)?)?;
    f.sync_all()?;
    Ok(path)
}
/// Takes ONLY replay-verified raw rows. No publication/approval possible from this function.
pub fn normalize(
    source: SourceReport,
    book: &Codebook,
    codebook_sha256: String,
) -> Result<NormalizedEvidence> {
    validate_book(book, &source)?;
    let mut seen = BTreeSet::new();
    let mut positions = BTreeSet::new();
    let mut observed: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut rows = Vec::new();
    let (mut aggregate_time, mut non_quantity, mut summary_origins, mut other_markets) =
        (0, 0, 0, 0);
    let (mut reported, mut below, mut no_fact) = (0, 0, 0);
    let required: BTreeSet<String> = book.axes.keys().cloned().collect();
    for row in &source.rows {
        if row.table_id != TABLE
            || row.position == 0
            || row.position > EXPECTED_ROWS
            || !positions.insert(row.position)
            || row.reporting_month.is_some()
            || row.annual_total.is_some()
            || row.attributes.keys().cloned().collect::<BTreeSet<_>>() != required
        {
            return Err(fail(format!(
                "unexpected raw observation coordinate at {}",
                row.position
            )));
        }
        for (axis, value) in &row.attributes {
            if !book.axes[axis].codes.contains(value) {
                return Err(fail(format!("unknown {axis} code at {}", row.position)));
            }
            observed
                .entry(axis.clone())
                .or_default()
                .insert(value.clone());
        }
        let coordinates: Vec<_> = row.attributes.iter().collect();
        if !seen.insert(
            coordinates
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<Vec<_>>(),
        ) {
            return Err(fail("duplicate classification coordinate"));
        }
        if row.unit.as_deref() != Some(book.quantity_unit.as_str())
            && coordinate(row, &book.measure_axis)? == book.quantity_measure_code
        {
            return Err(fail("quantity unit mismatch"));
        }
        if coordinate(row, &book.measure_axis)? != book.quantity_measure_code {
            non_quantity += 1;
            continue;
        }
        let month = match book.months[coordinate(row, &book.time_axis)?] {
            Some(month) => month,
            None => {
                aggregate_time += 1;
                continue;
            }
        };
        let prefecture = match &book.origins[coordinate(row, &book.origin_axis)?] {
            OriginMapping::ConsignorPrefecture { prefecture_code } => prefecture_code,
            OriginMapping::Summary => {
                summary_origins += 1;
                continue;
            }
        };
        let market_code = coordinate(row, &book.market_axis)?;
        if !matches!(&book.markets[market_code], MarketMapping::Major { .. }) {
            other_markets += 1;
            continue;
        }
        let quantity = classify_value(&row.raw_value)?;
        match quantity {
            Quantity::Reported { .. } => reported += 1,
            Quantity::BelowDisplayUnit => below += 1,
            Quantity::NoFact => no_fact += 1,
        }
        rows.push(NormalizedRow {
            source_table_id: TABLE.into(),
            source_position: row.position,
            source_attributes: row.attributes.clone(),
            month,
            consignor_prefecture_code: prefecture.clone(),
            major_consumer_region_code: market_code.into(),
            quantity,
            source_unit: book.quantity_unit.clone(),
        });
    }
    if seen.len() != EXPECTED_ROWS
        || observed
            .iter()
            .any(|(axis, values)| *values != book.axes[axis].codes.iter().cloned().collect())
    {
        return Err(fail("incomplete observed coordinate coverage"));
    }
    // Validate each 2024 month × consignor prefecture × major consumer region exists.
    let actual: BTreeSet<_> = rows
        .iter()
        .map(|r| {
            (
                r.month,
                r.consignor_prefecture_code.as_str(),
                r.major_consumer_region_code.as_str(),
            )
        })
        .collect();
    if rows.len() != 12 * 47 * 9 || actual.len() != rows.len() {
        return Err(fail(
            "incomplete or duplicate monthly quantity coordinate coverage",
        ));
    }
    Ok(NormalizedEvidence {
        validation: ValidationReport {
            status: "M5A_INSPECTION_ONLY".into(),
            publishable: false,
            table_id: TABLE.into(),
            metadata_sha256: source.source_metadata_sha256,
            codebook_sha256,
            input_count: source.count,
            normalized_count: rows.len(),
            skipped_aggregate_time: aggregate_time,
            skipped_non_quantity: non_quantity,
            skipped_summary_origins: summary_origins,
            skipped_subset_or_summary_markets: other_markets,
            reported_count: reported,
            below_display_unit_count: below,
            no_fact_count: no_fact,
            outstanding_gates: vec![
                "Independent source-code mapping review".into(),
                "Official spreadsheet reconciliation".into(),
                "M6 independent aggregation verification".into(),
                "M7 publication approval".into(),
            ],
        },
        rows,
    })
}

/// Convert the project's inspected 2024 codebook into the strict M5-A contract.
/// This is a structural conversion, NOT independent semantic approval.
fn parse_inspection_codebook(raw: &[u8]) -> Result<Codebook> {
    use serde_json::{Value, json};
    let doc: Value = serde_json::from_slice(raw)?;
    if doc.get("status").and_then(Value::as_str) != Some("INSPECTION_ONLY_REVIEW_PENDING") {
        return Err(fail(
            "unexpected inspection codebook status; review required",
        ));
    }
    if doc.get("table_id").and_then(Value::as_str) != Some(TABLE) {
        return Err(fail("inspection codebook table mismatch"));
    }
    let metadata_hash = doc
        .get("metadata_sha256")
        .and_then(Value::as_str)
        .ok_or("missing metadata hash")?;
    let evidence = doc
        .get("source_documents")
        .and_then(Value::as_array)
        .ok_or("missing source documents")?;
    if evidence.is_empty()
        || evidence
            .iter()
            .any(|v| v.as_str().map_or(true, str::is_empty))
    {
        return Err(fail("missing source-document citation"));
    }
    let mut axes = serde_json::Map::new();
    let mut months = serde_json::Map::new();
    let mut origins = serde_json::Map::new();
    let mut markets = serde_json::Map::new();
    let mut quantity_code: Option<String> = None;
    let mut quantity_unit: Option<String> = None;
    for (section, axis) in [
        ("time", "@time"),
        ("origin", "@cat01"),
        ("measure", "@cat02"),
        ("consumer_region", "@cat03"),
    ] {
        let entries = doc
            .get(section)
            .and_then(Value::as_array)
            .ok_or_else(|| fail(format!("missing axis {section}")))?;
        let mut codes = Vec::new();
        for entry in entries {
            let code = entry
                .get("code")
                .and_then(Value::as_str)
                .ok_or("missing code")?;
            let role = entry
                .get("role")
                .and_then(Value::as_str)
                .ok_or("missing role")?;
            let label = entry
                .get("label")
                .and_then(Value::as_str)
                .ok_or("missing label")?;
            if code.is_empty() || label.is_empty() {
                return Err(fail("empty code or label"));
            }
            codes.push(code.to_owned());
            match section {
                "time" => {
                    let period = entry
                        .get("reporting_month")
                        .ok_or("missing reporting month")?;
                    if role == "summary_excluded" && period.is_null() {
                        months.insert(code.to_owned(), Value::Null);
                    } else if role == "monthly" {
                        let value = period.as_str().ok_or("invalid reporting month")?;
                        let n = value
                            .strip_prefix("2024-")
                            .and_then(|s| s.parse::<u8>().ok())
                            .filter(|n| (1..=12).contains(n))
                            .ok_or("invalid 2024 month")?;
                        if value != format!("2024-{n:02}") {
                            return Err(fail("noncanonical reporting month"));
                        }
                        months.insert(code.to_owned(), json!(n));
                    } else {
                        return Err(fail("unexpected time role"));
                    }
                }
                "origin" => {
                    let value = match role {
                        "total_excluded" => json!({"kind":"summary"}),
                        "prefecture" => {
                            json!({"kind":"consignor_prefecture", "prefecture_code": code})
                        }
                        _ => return Err(fail("unexpected origin role")),
                    };
                    origins.insert(code.to_owned(), value);
                }
                "measure" => {
                    if role == "quantity" {
                        if quantity_code.replace(code.to_owned()).is_some() {
                            return Err(fail("duplicate quantity measure"));
                        }
                        quantity_unit = Some(
                            entry
                                .get("unit")
                                .and_then(Value::as_str)
                                .ok_or("missing quantity unit")?
                                .to_owned(),
                        );
                    } else if role != "price_excluded" {
                        return Err(fail("unexpected measure role"));
                    }
                }
                "consumer_region" => {
                    let parent = entry.get("parent_code").ok_or("missing region parent")?;
                    let value = match role {
                        "candidate_disjoint_contributor" if parent.is_null() => {
                            json!({"kind":"major", "name":label})
                        }
                        "overlapping_subset_excluded" => json!({"kind":"subset", "name":label,
                            "parent_code":parent.as_str().ok_or("missing subset parent")?}),
                        _ => return Err(fail("unexpected region role or parent")),
                    };
                    markets.insert(code.to_owned(), value);
                }
                _ => unreachable!(),
            }
        }
        axes.insert(axis.into(), json!({"codes":codes}));
    }
    let contract = json!({
        "format_version": 1, "table_id": TABLE, "metadata_sha256": metadata_hash,
        "axes": axes, "origin_axis":"@cat01", "measure_axis":"@cat02", "market_axis":"@cat03",
        "time_axis":"@time", "quantity_measure_code":quantity_code.ok_or("no quantity measure")?,
        "quantity_unit":quantity_unit.ok_or("no quantity unit")?,
        "months":months, "origins":origins, "markets":markets, "evidence":evidence,
    });
    Ok(serde_json::from_value(contract)?)
}

/// Writes one new file, only after successful offline replay and complete validation.
/// Caller chooses an ignored inspection directory; existing files are never overwritten.
pub fn run(
    acquisition_run: &Path,
    metadata_dir: &Path,
    codebook_file: &Path,
    output: &Path,
) -> Result<PathBuf> {
    let source = acquisition::replay(acquisition_run, metadata_dir)?;
    let raw = fs::read(codebook_file)?;
    let codebook = parse_inspection_codebook(&raw)?;
    let normalized = normalize(source, &codebook, hash(&raw))?;
    let path = output.join("m5a_inspection_only.json");
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)?;
    f.write_all(&serde_json::to_vec_pretty(&normalized)?)?;
    f.sync_all()?;
    Ok(path)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lexical_values_fail_closed() {
        assert_eq!(classify_value("- ").is_err(), true);
        assert_eq!(classify_value("-").unwrap(), Quantity::NoFact);
        assert_eq!(classify_value("0").unwrap(), Quantity::BelowDisplayUnit);
        assert_eq!(
            classify_value("1,234.5").unwrap(),
            Quantity::Reported {
                tonnes_lexical: "1,234.5".into()
            }
        );
        for s in ["", "-1", "NaN", "0.0", "--", "1,23", "1e3", "1..2", "+1"] {
            assert!(classify_value(s).is_err(), "{s}");
        }
    }
    #[test]
    fn refuses_wrong_table() {
        let book: Codebook = serde_json::from_str(r#"{"format_version":1,"table_id":"bad","metadata_sha256":"x","axes":{},"origin_axis":"@cat01","market_axis":"@cat03","time_axis":"@time","measure_axis":"@cat02","quantity_measure_code":"1","quantity_unit":"t","months":{},"origins":{},"markets":{},"evidence":["source"]}"#).unwrap();
        let source = SourceReport {
            table_id: "bad".into(),
            source_metadata_sha256: "x".into(),
            count: 0,
            pages: 0,
            gate: "RAW_ONLY_NOT_PUBLISHABLE".into(),
            rows: vec![],
        };
        assert!(normalize(source, &book, "hash".into()).is_err());
    }
}
