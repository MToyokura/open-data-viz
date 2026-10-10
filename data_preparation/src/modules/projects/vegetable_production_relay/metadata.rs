//! M3: immutable getMetaInfo evidence, offline replay, and unresolved classification inventory.
//! An observed axis name is not a verified semantic mapping.
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    error::Error,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const ENDPOINT: &str = "https://api.e-stat.go.jp/rest/3.0/app/json/getMetaInfo";
const MAX_BYTES: usize = 32 * 1024 * 1024;
const MAX_ATTEMPTS: usize = 3;
type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn err(message: impl Into<String>) -> Box<dyn Error> {
    message.into().into()
}
fn required<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| err(format!("missing or invalid {key}")))
}
fn atom(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}
fn entries(v: &Value) -> Result<Vec<&Value>> {
    match v {
        Value::Array(a) => Ok(a.iter().collect()),
        Value::Object(_) => Ok(vec![v]),
        _ => Err(err("classification entries must be object or array")),
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Capture {
    pub endpoint: String,
    pub parameters: Vec<(String, String)>,
    pub requested_unix_seconds: u64,
    pub duration_ms: u128,
    pub http_status: u16,
    pub byte_count: usize,
    pub sha256: String,
    pub attempts: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassValue {
    pub code: String,
    pub label: String,
    pub level: Option<String>,
    pub unit: Option<String>,
    pub parent_code: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Axis {
    pub id: String,
    pub label: String,
    pub values: Vec<ClassValue>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub stats_data_id: String,
    pub source_sha256: String,
    pub axes: Vec<Axis>,
    pub status: String,
    pub unresolved: Vec<String>,
}
/// Inventory all observed IDs and labels. Do not assign geographical, time, unit,
/// measure, or market meaning without separate source documentation.
pub fn parse(bytes: &[u8], expected_table: &str) -> Result<Inventory> {
    let doc: Value = serde_json::from_slice(bytes)?;
    let root = doc.get("GET_META_INFO").ok_or("GET_META_INFO missing")?;
    let status = atom(&root["RESULT"]["STATUS"]).ok_or("RESULT.STATUS missing")?;
    if status != "0" {
        return Err(err("e-Stat metadata application status is not success"));
    }
    if let Some(returned) = root["PARAMETER"]["STATS_DATA_ID"].as_str() {
        if returned != expected_table {
            return Err(err("metadata parameter table ID mismatch"));
        }
    }
    let meta = root.get("METADATA_INF").ok_or("METADATA_INF missing")?;
    let table = required(&meta["TABLE_INF"], "@id")?;
    if table != expected_table {
        return Err(err("metadata table ID mismatch"));
    }
    let object = meta
        .get("CLASS_INF")
        .and_then(|v| v.get("CLASS_OBJ"))
        .ok_or("CLASS_OBJ missing")?;
    let mut axes = Vec::new();
    let mut ids = HashSet::new();
    for group in entries(object)? {
        let id = required(group, "@id")?.to_owned();
        if !ids.insert(id.clone()) {
            return Err(err("duplicate metadata axis ID"));
        }
        let label = required(group, "@name")?.to_owned();
        let source = group.get("CLASS").ok_or("axis CLASS missing")?;
        let mut values = Vec::new();
        let mut codes = HashSet::new();
        for row in entries(source)? {
            let code = required(row, "@code")?.to_owned();
            if !codes.insert(code.clone()) {
                return Err(err(format!("duplicate code in axis {id}")));
            }
            let label = required(row, "@name")?.to_owned();
            values.push(ClassValue {
                code,
                label,
                level: row.get("@level").and_then(atom),
                unit: row.get("@unit").and_then(atom),
                parent_code: row.get("@parentCode").and_then(atom),
            });
        }
        if values.is_empty() {
            return Err(err("empty classification axis"));
        }
        axes.push(Axis { id, label, values });
    }
    if axes.is_empty() {
        return Err(err("no metadata axes"));
    }
    let unresolved = vec![
        "Dimension roles (origin, destination market, measure, time) need independent corroboration".into(),
        "Prefecture membership and market inclusion/exclusion are not verified".into(),
        "Units, annual totals, month coding and hierarchy require source evidence".into(),
    ];
    Ok(Inventory {
        stats_data_id: expected_table.into(),
        source_sha256: sha(bytes),
        axes,
        status: "pending_semantic_verification".into(),
        unresolved,
    })
}
fn save_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut f = OpenOptions::new().write(true).create_new(true).open(path)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    Ok(())
}
fn verify(dir: &Path) -> Result<(Vec<u8>, Capture, String)> {
    let metadata: Capture = serde_json::from_slice(&fs::read(dir.join("capture.json"))?)?;
    if metadata.endpoint != ENDPOINT
        || metadata.http_status != 200
        || metadata.attempts == 0
        || metadata.attempts > MAX_ATTEMPTS
    {
        return Err(err("invalid capture endpoint/status/attempts"));
    }
    if metadata.parameters.len() != 1 || metadata.parameters[0].0 != "statsDataId" {
        return Err(err("unsafe or unexpected capture parameters"));
    }
    let table = metadata.parameters[0].1.clone();
    if table.is_empty() || !table.bytes().all(|c| c.is_ascii_alphanumeric()) {
        return Err(err("invalid table ID"));
    }
    let bytes = fs::read(dir.join("response.bin"))?;
    if bytes.len() > MAX_BYTES
        || bytes.len() != metadata.byte_count
        || sha(&bytes) != metadata.sha256
    {
        return Err(err("metadata capture checksum/length mismatch"));
    }
    Ok((bytes, metadata, table))
}
fn report(inv: &Inventory) -> String {
    let mut s = format!(
        "# getMetaInfo classification inventory\n\nTable: `{}`\n\nRaw SHA-256: `{}`\n\n**Gate: BLOCKED — semantic verification pending.**\n\n",
        inv.stats_data_id, inv.source_sha256
    );
    for axis in &inv.axes {
        s.push_str(&format!("## {} — {} ({} codes)\n\n| Code | Source label | Level | Unit | Parent |\n|---|---|---|---|---|\n", axis.id, axis.label.replace('|', "\\|"), axis.values.len()));
        for value in &axis.values {
            let escaped = |s: &str| s.replace('|', "\\|").replace('\n', " ");
            s.push_str(&format!(
                "| `{}` | {} | {} | {} | {} |\n",
                escaped(&value.code),
                escaped(&value.label),
                value.level.as_deref().unwrap_or(""),
                value.unit.as_deref().unwrap_or(""),
                value.parent_code.as_deref().unwrap_or("")
            ));
        }
        s.push('\n');
    }
    s.push_str("## Unresolved interpretations\n\n");
    for item in &inv.unresolved {
        s.push_str(&format!("- {item}\n"));
    }
    s
}
/// Replay uses only saved bytes and capture metadata; never requires an API key.
pub fn replay(dir: &Path) -> Result<Inventory> {
    let (bytes, _, table) = verify(dir)?;
    parse(&bytes, &table)
}
pub fn write_report(dir: &Path) -> Result<()> {
    let inv = replay(dir)?;
    save_new(
        &dir.join("inventory.json"),
        &serde_json::to_vec_pretty(&inv)?,
    )?;
    save_new(&dir.join("inventory.md"), report(&inv).as_bytes())?;
    println!(
        "Metadata inventory saved to {} (semantics remain unverified)",
        dir.display()
    );
    Ok(())
}
/// Online inspection captures exact final response bytes, including HTTP failures.
pub fn inspect(app_id: &str, table: &str) -> Result<PathBuf> {
    if app_id.is_empty() {
        return Err(err("ESTAT_APP_ID is empty"));
    }
    if table.is_empty() || !table.bytes().all(|c| c.is_ascii_alphanumeric()) {
        return Err(err("table ID must be alphanumeric"));
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let dir = Path::new("inspections/vegetable_production_relay/runs").join(format!(
        "metadata-{}-{}-{}",
        now.as_secs(),
        now.subsec_nanos(),
        std::process::id()
    ));
    fs::create_dir(&dir)?;
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()?;
    let mut last_error = None;
    for attempt in 1..=MAX_ATTEMPTS {
        let requested = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let start = Instant::now();
        match client
            .get(ENDPOINT)
            .query(&[("appId", app_id), ("statsDataId", table)])
            .send()
        {
            Ok(response) => {
                let status = response.status().as_u16();
                let mut bytes = Vec::new();
                response
                    .take((MAX_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)?;
                if bytes.len() > MAX_BYTES {
                    return Err(err("metadata response too large"));
                }
                if (status == 429 || status >= 500) && attempt < MAX_ATTEMPTS {
                    thread::sleep(Duration::from_millis(250 * attempt as u64));
                    continue;
                }
                let metadata = Capture {
                    endpoint: ENDPOINT.into(),
                    parameters: vec![("statsDataId".into(), table.into())],
                    requested_unix_seconds: requested,
                    duration_ms: start.elapsed().as_millis(),
                    http_status: status,
                    byte_count: bytes.len(),
                    sha256: sha(&bytes),
                    attempts: attempt,
                };
                save_new(&dir.join("response.bin"), &bytes)?;
                save_new(
                    &dir.join("capture.json"),
                    &serde_json::to_vec_pretty(&metadata)?,
                )?;
                if status != 200 {
                    return Err(err(format!("HTTP {status} captured in {}", dir.display())));
                }
                write_report(&dir)?;
                return Ok(dir);
            }
            Err(_) => {
                last_error = Some("network failure (URL withheld)");
                if attempt < MAX_ATTEMPTS {
                    thread::sleep(Duration::from_millis(250 * attempt as u64));
                }
            }
        }
    }
    Err(err(last_error.unwrap_or("inspection failed")))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({"GET_META_INFO":{"RESULT":{"STATUS":0}, "PARAMETER":{"STATS_DATA_ID":"T1"}, "METADATA_INF":{"TABLE_INF":{"@id":"T1"},"CLASS_INF":{"CLASS_OBJ":[{"@id":"cat01","@name":"分類","CLASS":[{"@code":"001","@name":"A","@level":"1","@unit":"t"},{"@code":"002","@name":"B"}]},{"@id":"time","@name":"時間","CLASS":{"@code":"202001","@name":"2020年1月"}}]}}}})).unwrap()
    }
    #[test]
    fn retains_ids_and_units_without_interpretation() {
        let i = parse(&fixture(), "T1").unwrap();
        assert_eq!(i.axes[0].values[0].code, "001");
        assert_eq!(i.axes[0].values[0].unit.as_deref(), Some("t"));
        assert_eq!(i.axes[1].values.len(), 1);
        assert_eq!(i.status, "pending_semantic_verification");
    }
    #[test]
    fn rejects_duplicates_wrong_tables_and_errors() {
        let mut v: Value = serde_json::from_slice(&fixture()).unwrap();
        assert!(parse(&fixture(), "T2").is_err());
        v["GET_META_INFO"]["METADATA_INF"]["CLASS_INF"]["CLASS_OBJ"][0]["CLASS"][1]["@code"] =
            Value::String("001".into());
        assert!(parse(&serde_json::to_vec(&v).unwrap(), "T1").is_err());
        v["GET_META_INFO"]["RESULT"]["STATUS"] = Value::from(1);
        assert!(parse(&serde_json::to_vec(&v).unwrap(), "T1").is_err());
    }
    #[test]
    fn checksum_and_credentials_are_checked() {
        let path = std::env::temp_dir().join(format!(
            "meta-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        let bytes = fixture();
        fs::write(path.join("response.bin"), &bytes).unwrap();
        let mut m = Capture {
            endpoint: ENDPOINT.into(),
            parameters: vec![("statsDataId".into(), "T1".into())],
            requested_unix_seconds: 0,
            duration_ms: 0,
            http_status: 200,
            byte_count: bytes.len(),
            sha256: sha(&bytes),
            attempts: 1,
        };
        fs::write(path.join("capture.json"), serde_json::to_vec(&m).unwrap()).unwrap();
        assert!(replay(&path).is_ok());
        m.parameters.push(("appId".into(), "secret".into()));
        fs::write(path.join("capture.json"), serde_json::to_vec(&m).unwrap()).unwrap();
        assert!(replay(&path).is_err());
        fs::remove_dir_all(path).unwrap();
    }
}
