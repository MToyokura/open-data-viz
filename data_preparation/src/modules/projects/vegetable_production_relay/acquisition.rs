//! M4 raw observation acquisition. Source references are never interpreted as calendar,
//! geography or comparable measures. In particular 2021-23 monthly publication is blocked.
use super::metadata;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const ENDPOINT: &str = "https://api.e-stat.go.jp/rest/3.0/app/json/getStatsData";
const LIMIT: usize = 1000;
const MAX_PAGES: usize = 10000;
const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_ATTEMPTS: usize = 3;
fn fail(s: impl Into<String>) -> Box<dyn Error> {
    s.into().into()
}
fn sha(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn save(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut f = OpenOptions::new().write(true).create_new(true).open(path)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    Ok(())
}
fn atom(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}
fn number(v: &Value) -> Result<usize> {
    let s = atom(v).ok_or_else(|| fail("missing pagination number"))?;
    Ok(s.parse::<usize>()?)
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Page {
    start: usize,
    count: usize,
    total: usize,
    http_status: u16,
    attempts: usize,
    duration_ms: u128,
    byte_count: usize,
    sha256: String,
    file: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Ledger {
    format_version: u32,
    endpoint: String,
    table_id: String,
    metadata_sha256: String,
    page_limit: usize,
    pages: Vec<Page>,
    total: usize,
    semantic_gate: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceRow {
    pub table_id: String,
    pub position: usize,
    pub attributes: BTreeMap<String, String>,
    pub raw_value: String,
    pub unit: Option<String>,
    pub reporting_month: Option<String>,
    pub annual_total: Option<bool>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SourceReport {
    pub table_id: String,
    pub source_metadata_sha256: String,
    pub count: usize,
    pub pages: usize,
    pub gate: String,
    pub rows: Vec<SourceRow>,
}
fn parse_page(bytes: &[u8], table: &str, start: usize) -> Result<(usize, Vec<SourceRow>)> {
    let doc: Value = serde_json::from_slice(bytes)?;
    let root = doc.get("GET_STATS_DATA").ok_or("GET_STATS_DATA missing")?;
    if atom(&root["RESULT"]["STATUS"]).as_deref() != Some("0") {
        return Err(fail("API application status not success"));
    }
    if let Some(id) = root["PARAMETER"]["STATS_DATA_ID"].as_str() {
        if id != table {
            return Err(fail("response table ID mismatch"));
        }
    }
    let data = root
        .get("STATISTICAL_DATA")
        .ok_or("STATISTICAL_DATA missing")?;
    let result = data.get("RESULT_INF").ok_or("RESULT_INF missing")?;
    let total = number(&result["TOTAL_NUMBER"])?;
    let from = number(&result["FROM_NUMBER"])?;
    let to = number(&result["TO_NUMBER"])?;
    if from != start || to < from || to > total || to - from + 1 > LIMIT {
        return Err(fail("invalid page interval"));
    }
    let values = data
        .get("DATA_INF")
        .and_then(|d| d.get("VALUE"))
        .ok_or("DATA_INF.VALUE missing")?;
    let entries: Vec<&Value> = match values {
        Value::Array(a) => a.iter().collect(),
        Value::Object(_) => vec![values],
        _ => return Err(fail("invalid VALUE shape")),
    };
    if entries.len() != to - from + 1 {
        return Err(fail("API interval/row count mismatch"));
    }
    let mut rows = Vec::with_capacity(entries.len());
    for (i, entry) in entries.into_iter().enumerate() {
        let obj = entry.as_object().ok_or("VALUE item not object")?;
        let mut attributes = BTreeMap::new();
        for (key, value) in obj {
            if key == "$" || key == "@unit" {
                continue;
            }
            if !key.starts_with('@') {
                return Err(fail("unexpected observation field"));
            }
            attributes.insert(
                key.clone(),
                atom(value).ok_or("nonlexical classification attribute")?,
            );
        }
        if attributes.is_empty() {
            return Err(fail("observation has no classification references"));
        }
        let raw_value = atom(obj.get("$").ok_or("observation value missing")?)
            .ok_or("unsupported value shape")?;
        rows.push(SourceRow {
            table_id: table.into(),
            position: from + i,
            attributes,
            raw_value,
            unit: obj
                .get("@unit")
                .map(|v| atom(v).ok_or("invalid unit"))
                .transpose()?,
            reporting_month: None,
            annual_total: None,
        });
    }
    Ok((total, rows))
}
/// Replay is strictly offline; no report is written by replay.
pub fn replay(run: &Path, metadata_dir: &Path) -> Result<SourceReport> {
    let inv = metadata::replay(metadata_dir)?;
    let ledger: Ledger = serde_json::from_slice(&fs::read(run.join("ledger.json"))?)?;
    if ledger.format_version != 1
        || ledger.endpoint != ENDPOINT
        || ledger.table_id != inv.stats_data_id
        || ledger.metadata_sha256 != inv.source_sha256
        || ledger.page_limit != LIMIT
        || ledger.semantic_gate != "RAW_ONLY"
        || ledger.pages.is_empty()
        || ledger.pages.len() > MAX_PAGES
    {
        return Err(fail("invalid ledger or metadata binding"));
    }
    let mut expected = 1;
    let mut rows = Vec::new();
    let axes: BTreeMap<_, _> = inv
        .axes
        .iter()
        .map(|a| {
            (
                a.id.as_str(),
                a.values
                    .iter()
                    .map(|v| v.code.as_str())
                    .collect::<HashSet<_>>(),
            )
        })
        .collect();
    for (i, page) in ledger.pages.iter().enumerate() {
        if page.start != expected
            || page.count == 0
            || page.count > LIMIT
            || page.total != ledger.total
            || page.http_status != 200
            || page.attempts == 0
            || page.attempts > MAX_ATTEMPTS
            || page.file != format!("page-{i:05}.bin")
        {
            return Err(fail("page gap, duplicate or invalid ledger"));
        }
        let bytes = fs::read(run.join(&page.file))?;
        if bytes.len() > MAX_BYTES || bytes.len() != page.byte_count || sha(&bytes) != page.sha256 {
            return Err(fail("page checksum/length mismatch"));
        }
        let (total, part) = parse_page(&bytes, &ledger.table_id, expected)?;
        if total != ledger.total || part.len() != page.count {
            return Err(fail("page count/total mismatch"));
        }
        for r in &part {
            for (axis, codes) in &axes {
                if let Some(code) = r.attributes.get(&format!("@{axis}")) {
                    if !codes.contains(code.as_str()) {
                        return Err(fail("observation references unknown metadata code"));
                    }
                }
            }
            if r.reporting_month.is_some() || r.annual_total.is_some() {
                return Err(fail("unreviewed semantic mapping"));
            }
        }
        expected += part.len();
        rows.extend(part);
    }
    if expected - 1 != ledger.total {
        return Err(fail("incomplete captured table"));
    }
    Ok(SourceReport {
        table_id: ledger.table_id,
        source_metadata_sha256: inv.source_sha256,
        count: rows.len(),
        pages: ledger.pages.len(),
        gate: "RAW_ONLY_NOT_PUBLISHABLE".into(),
        rows,
    })
}
/// All pages, including their exact bytes, are retained in an ignored inspection directory.
/// Ledger is written LAST, so interrupted captures cannot replay as complete.
pub fn acquire(app_id: &str, metadata_dir: &Path) -> Result<PathBuf> {
    if app_id.is_empty() {
        return Err(fail("ESTAT_APP_ID missing"));
    }
    let inv = metadata::replay(metadata_dir)?;
    if !inv.stats_data_id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(fail("invalid table ID"));
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let run = Path::new("inspections/vegetable_production_relay/runs").join(format!(
        "observations-{}-{}-{}",
        now.as_secs(),
        now.subsec_nanos(),
        std::process::id()
    ));
    fs::create_dir(&run)?;
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(60))
        .build()?;
    let mut ledger = Ledger {
        format_version: 1,
        endpoint: ENDPOINT.into(),
        table_id: inv.stats_data_id.clone(),
        metadata_sha256: inv.source_sha256,
        page_limit: LIMIT,
        pages: Vec::new(),
        total: 0,
        semantic_gate: "RAW_ONLY".into(),
    };
    let mut next = 1;
    loop {
        if ledger.pages.len() >= MAX_PAGES {
            return Err(fail("page cap exceeded"));
        }
        let mut final_response = None;
        for attempt in 1..=MAX_ATTEMPTS {
            let started = Instant::now();
            let reply = client
                .get(ENDPOINT)
                .query(&[
                    ("appId", app_id),
                    ("statsDataId", &ledger.table_id),
                    ("startPosition", &next.to_string()),
                    ("limit", &LIMIT.to_string()),
                ])
                .send();
            if let Ok(response) = reply {
                let status = response.status().as_u16();
                let mut bytes = Vec::new();
                response
                    .take((MAX_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)?;
                if bytes.len() > MAX_BYTES {
                    return Err(fail("page size cap exceeded"));
                }
                if (status == 429 || status >= 500) && attempt < MAX_ATTEMPTS {
                    thread::sleep(Duration::from_millis(250 * attempt as u64));
                    continue;
                }
                final_response = Some((status, bytes, attempt, started.elapsed().as_millis()));
                break;
            }
            if attempt < MAX_ATTEMPTS {
                thread::sleep(Duration::from_millis(250 * attempt as u64));
            }
        }
        let (status, bytes, attempts, elapsed) =
            final_response.ok_or("network failure; credentials withheld")?;
        let file = format!("page-{:05}.bin", ledger.pages.len());
        save(&run.join(&file), &bytes)?;
        if status != 200 {
            return Err(fail(format!(
                "HTTP {status}; raw response retained in {}",
                run.display()
            )));
        }
        let (total, rows) = parse_page(&bytes, &ledger.table_id, next)?;
        if total == 0 || (ledger.total != 0 && total != ledger.total) {
            return Err(fail("empty or changed result total"));
        }
        ledger.total = total;
        ledger.pages.push(Page {
            start: next,
            count: rows.len(),
            total,
            http_status: status,
            attempts,
            duration_ms: elapsed,
            byte_count: bytes.len(),
            sha256: sha(&bytes),
            file,
        });
        next += rows.len();
        if next > total {
            break;
        }
    }
    save(
        &run.join("ledger.json"),
        &serde_json::to_vec_pretty(&ledger)?,
    )?;
    let report = replay(&run, metadata_dir)?;
    save(
        &run.join("source_observations.json"),
        &serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(run)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(from: usize, to: usize, total: usize, values: &str) -> Vec<u8> {
        format!(r#"{{"GET_STATS_DATA":{{"RESULT":{{"STATUS":0}},"PARAMETER":{{"STATS_DATA_ID":"0002008540"}},"STATISTICAL_DATA":{{"RESULT_INF":{{"TOTAL_NUMBER":{total},"FROM_NUMBER":{from},"TO_NUMBER":{to}}},"DATA_INF":{{"VALUE":{values}}}}}}}}}"#).into_bytes()
    }
    #[test]
    fn preserves_lexical_markers_and_unknown_month() {
        let bytes = fixture(
            1,
            2,
            2,
            r#"[{"@cat03":"1001","@cat01":"1","@unit":"t","$":"-"},{"@cat03":"1002","$":"0"}]"#,
        );
        let (_, rows) = parse_page(&bytes, "0002008540", 1).unwrap();
        assert_eq!(rows[0].raw_value, "-");
        assert_eq!(rows[1].raw_value, "0");
        assert_eq!(rows[0].unit.as_deref(), Some("t"));
        assert_eq!(rows[0].reporting_month, None);
        assert_eq!(rows[0].annual_total, None);
    }
    #[test]
    fn rejects_pagination_gaps_and_bad_application_status() {
        let bytes = fixture(2, 2, 2, r#"{"@cat03":"1001","$":"3"}"#);
        assert!(parse_page(&bytes, "0002008540", 1).is_err());
        let bad = br#"{"GET_STATS_DATA":{"RESULT":{"STATUS":1}}}"#;
        assert!(parse_page(bad, "0002008540", 1).is_err());
    }
    #[test]
    fn rejects_row_count_disagreement() {
        let bytes = fixture(1, 2, 2, r#"{"@cat03":"1001","$":"-"}"#);
        assert!(parse_page(&bytes, "0002008540", 1).is_err());
    }
}
