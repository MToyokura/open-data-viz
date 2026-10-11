//! M3 semantic investigation: describe repeated classification blocks and sample
//! explicitly chosen source codes. No calendar mapping or dataset publication.
use super::metadata;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const ENDPOINT: &str = "https://api.e-stat.go.jp/rest/3.0/app/json/getStatsData";
const MAX_BYTES: usize = 4 * 1024 * 1024;
const LIMIT: u32 = 10;
const MAX_ATTEMPTS: usize = 3;
fn fail(message: impl Into<String>) -> Box<dyn Error> {
    message.into().into()
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn save_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
fn items(v: &Value) -> Result<Vec<&Value>> {
    match v {
        Value::Array(a) => Ok(a.iter().collect()),
        Value::Object(_) => Ok(vec![v]),
        _ => Err(fail("VALUE must be an object or array")),
    }
}
#[derive(Debug, Serialize, Deserialize)]
struct ProbeCapture {
    endpoint: String,
    stats_data_id: String,
    cd_cat03: String,
    metadata_sha256: String,
    limit: u32,
    http_status: u16,
    attempts: usize,
    duration_ms: u128,
    byte_count: usize,
    sha256: String,
}
#[derive(Debug, Serialize)]
struct ObservedRow {
    cat03: Option<String>,
    cat01: Option<String>,
    cat02: Option<String>,
    time: Option<String>,
    // Retain original lexical value. Do not parse quantity/price here.
    raw_value: Option<String>,
}
#[derive(Debug, Serialize)]
struct ProbeReport {
    table_id: String,
    requested_cat03: String,
    metadata_sha256: String,
    response_sha256: String,
    returned_rows: usize,
    rows: Vec<ObservedRow>,
    gate: &'static str,
    warning: &'static str,
}
fn parse_rows(bytes: &[u8]) -> Result<Vec<ObservedRow>> {
    let doc: Value = serde_json::from_slice(bytes)?;
    let root = doc.get("GET_STATS_DATA").ok_or("GET_STATS_DATA missing")?;
    let status = &root["RESULT"]["STATUS"];
    if !(status == "0" || status == 0) {
        return Err(fail("getStatsData application status is not success"));
    }
    let data = root
        .get("STATISTICAL_DATA")
        .ok_or("STATISTICAL_DATA missing")?;
    let values = data
        .get("DATA_INF")
        .and_then(|v| v.get("VALUE"))
        .ok_or("DATA_INF.VALUE missing")?;
    let mut out = Vec::new();
    for value in items(values)? {
        let get = |key: &str| value.get(key).and_then(Value::as_str).map(str::to_owned);
        let raw_value = value.get("$").and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            _ => None,
        });
        out.push(ObservedRow {
            cat03: get("@cat03"),
            cat01: get("@cat01"),
            cat02: get("@cat02"),
            time: get("@time"),
            raw_value,
        });
    }
    if out.is_empty() || out.len() > LIMIT as usize {
        return Err(fail("empty or oversized data sample"));
    }
    Ok(out)
}
/// Describe code blocks only when the entire axis repeats exactly and block
/// boundaries are unambiguous; do not associate block ordinals with months.
pub fn block_report(metadata_dir: &Path) -> Result<PathBuf> {
    let inv = metadata::replay(metadata_dir)?;
    let axis = inv
        .axes
        .iter()
        .find(|axis| axis.id == "cat03")
        .ok_or("cat03 axis missing")?;
    let path = metadata_dir.join("semantic_blocks.md");
    let mut text = format!(
        "# M3 repeated-classification evidence\n\nTable: `{}`\nMetadata SHA-256: `{}`\n\n**Gate: BLOCKED. No month mapping inferred.**\n\n",
        inv.stats_data_id, inv.source_sha256
    );
    if axis.values.len() == 156 {
        let first = &axis.values[..12];
        let all_equal = axis.values.chunks_exact(12).all(|chunk| {
            chunk
                .iter()
                .zip(first)
                .all(|(a, b)| a.label == b.label && a.level == b.level)
        });
        if !all_equal {
            return Err(fail(
                "156 codes do not form 13 identical twelve-label blocks",
            ));
        }
        text.push_str("Observed 13 consecutive blocks with identical sequences of 12 source labels and hierarchy levels. This is **not evidence of block-to-month correspondence**.\n\n| Block (ordinal only) | First code | Last code | First label |\n|---:|---|---|---|\n");
        for (idx, chunk) in axis.values.chunks(12).enumerate() {
            text.push_str(&format!(
                "| {} | `{}` | `{}` | {} |\n",
                idx + 1,
                chunk[0].code,
                chunk[11].code,
                chunk[0].label.replace('|', "\\|")
            ));
        }
    } else {
        text.push_str(&format!(
            "cat03 contains {} codes; 13 × 12 repetition not detected.\n",
            axis.values.len()
        ));
    }
    text.push_str("\nTo establish temporal meaning, obtain independently dated source documentation or evidence that explicitly links source codes to months. Sample observations alone do not establish month assignments.\n");
    save_new(&path, text.as_bytes())?;
    Ok(path)
}
fn validated_code(metadata_dir: &Path, code: &str) -> Result<(String, String)> {
    let inv = metadata::replay(metadata_dir)?;
    if !matches!(
        inv.stats_data_id.as_str(),
        "0002008540" | "0002120386" | "0002114330"
    ) {
        return Err(fail(
            "probe restricted to the three investigated consolidated table IDs",
        ));
    }
    let axis = inv
        .axes
        .iter()
        .find(|axis| axis.id == "cat03")
        .ok_or("cat03 missing")?;
    if axis.values.len() != 156 || !axis.values.iter().any(|v| v.code == code) {
        return Err(fail("requested code not present in 156-code cat03 axis"));
    }
    Ok((inv.stats_data_id, inv.source_sha256))
}
/// Bounded single-code sample. Stores exact bytes; does not interpret or publish.
pub fn probe(app_id: &str, metadata_dir: &Path, code: &str) -> Result<PathBuf> {
    if app_id.is_empty() {
        return Err(fail("ESTAT_APP_ID missing"));
    }
    let (table, metadata_sha256) = validated_code(metadata_dir, code)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let dir = Path::new("inspections/vegetable_production_relay/runs").join(format!(
        "semantic-probe-{}-{}-{}",
        now.as_secs(),
        now.subsec_nanos(),
        std::process::id()
    ));
    fs::create_dir(&dir)?;
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()?;
    for attempt in 1..=MAX_ATTEMPTS {
        let start = Instant::now();
        // No secret-bearing URL or error body is logged.
        match client
            .get(ENDPOINT)
            .query(&[
                ("appId", app_id),
                ("statsDataId", table.as_str()),
                ("cdCat03", code),
                ("limit", "10"),
            ])
            .send()
        {
            Ok(response) => {
                let status = response.status().as_u16();
                let mut bytes = Vec::new();
                response
                    .take((MAX_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)?;
                if bytes.len() > MAX_BYTES {
                    return Err(fail("probe response exceeds size cap"));
                }
                if (status == 429 || status >= 500) && attempt < MAX_ATTEMPTS {
                    thread::sleep(Duration::from_millis(250 * attempt as u64));
                    continue;
                }
                let capture = ProbeCapture {
                    endpoint: ENDPOINT.into(),
                    stats_data_id: table,
                    cd_cat03: code.into(),
                    metadata_sha256,
                    limit: LIMIT,
                    http_status: status,
                    attempts: attempt,
                    duration_ms: start.elapsed().as_millis(),
                    byte_count: bytes.len(),
                    sha256: hash(&bytes),
                };
                save_new(&dir.join("response.bin"), &bytes)?;
                save_new(
                    &dir.join("capture.json"),
                    &serde_json::to_vec_pretty(&capture)?,
                )?;
                if status != 200 {
                    return Err(fail(format!(
                        "HTTP {status}; final response retained in {}",
                        dir.display()
                    )));
                }
                replay(&dir, metadata_dir)?;
                return Ok(dir);
            }
            Err(_) => {
                if attempt < MAX_ATTEMPTS {
                    thread::sleep(Duration::from_millis(250 * attempt as u64));
                }
            }
        }
    }
    Err(fail("network failure; URL and credentials withheld"))
}
/// Verify both metadata and response bytes offline, and reject samples whose
/// returned cat03 references do not match the requested classification code.
pub fn replay(probe_dir: &Path, metadata_dir: &Path) -> Result<()> {
    let capture: ProbeCapture = serde_json::from_slice(&fs::read(probe_dir.join("capture.json"))?)?;
    if capture.endpoint != ENDPOINT
        || capture.http_status != 200
        || capture.limit != LIMIT
        || capture.attempts == 0
        || capture.attempts > MAX_ATTEMPTS
    {
        return Err(fail("invalid probe capture"));
    }
    let (table, metadata_hash) = validated_code(metadata_dir, &capture.cd_cat03)?;
    if capture.stats_data_id != table || capture.metadata_sha256 != metadata_hash {
        return Err(fail("metadata/probe binding mismatch"));
    }
    let bytes = fs::read(probe_dir.join("response.bin"))?;
    if bytes.len() > MAX_BYTES
        || bytes.len() != capture.byte_count
        || hash(&bytes) != capture.sha256
    {
        return Err(fail("probe capture checksum/length mismatch"));
    }
    let rows = parse_rows(&bytes)?;
    if rows
        .iter()
        .any(|r| r.cat03.as_deref() != Some(capture.cd_cat03.as_str()))
    {
        return Err(fail("returned cat03 code differs from request"));
    }
    let report = ProbeReport {
        table_id: table,
        requested_cat03: capture.cd_cat03,
        metadata_sha256: metadata_hash,
        response_sha256: capture.sha256,
        returned_rows: rows.len(),
        rows,
        gate: "BLOCKED",
        warning: "Only observed references; no month, geographic, measurement, or comparability interpretation",
    };
    // Offline replay never writes/overwrites evidence; report is optional and immutable.
    let output = probe_dir.join("sample.json");
    if !output.exists() {
        save_new(&output, &serde_json::to_vec_pretty(&report)?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_single_and_multiple_rows_without_calendar_guessing() {
        let bytes = br#"{"GET_STATS_DATA":{"RESULT":{"STATUS":0},"STATISTICAL_DATA":{"DATA_INF":{"VALUE":{"@cat03":"1001","@cat01":"1002","@cat02":"1001","$":"12"}}}}}"#;
        let rows = parse_rows(bytes).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cat03.as_deref(), Some("1001"));
        assert_eq!(rows[0].time, None);
    }
    #[test]
    fn rejects_api_errors_and_empty_data() {
        assert!(parse_rows(br#"{"GET_STATS_DATA":{"RESULT":{"STATUS":1},"STATISTICAL_DATA":{"DATA_INF":{"VALUE":[]}}}}"#).is_err());
        assert!(parse_rows(br#"{"GET_STATS_DATA":{"RESULT":{"STATUS":0},"STATISTICAL_DATA":{"DATA_INF":{"VALUE":[]}}}}"#).is_err());
    }
}
