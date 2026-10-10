//! M2-A: capture and validate a complete getStatsList catalogue, without selecting tables.
use reqwest::{StatusCode, blocking::Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    error::Error,
    fs,
    io::Read,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const ENDPOINT: &str = "https://api.e-stat.go.jp/rest/3.0/app/json/getStatsList";
const ROOT: &str = "inspections/vegetable_production_relay/runs";
const STATS_CODE: &str = "00500226";
const PAGE_SIZE: usize = 100;
const MAX_PAGES: usize = 10000;
const MAX_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
struct PageMetadata {
    endpoint: String,
    parameters: Vec<(String, String)>,
    requested_unix_seconds: u64,
    duration_ms: u128,
    http_status: u16,
    byte_count: usize,
    sha256: String,
    attempts: usize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableCandidate {
    pub id: String,
    pub title: String,
    pub statistics_name: Option<String>,
    pub survey_date: Option<String>,
    pub open_date: Option<String>,
    pub updated_date: Option<String>,
    pub table_sub_category1: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Catalogue {
    pub stats_code: String,
    pub declared_total: usize,
    pub candidates: Vec<TableCandidate>,
    pub pages: usize,
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn fail(msg: impl Into<String>) -> Box<dyn Error> {
    msg.into().into()
}
fn integer(v: &Value, key: &str) -> Result<usize, Box<dyn Error>> {
    let x = &v[key];
    if let Some(n) = x.as_u64() {
        return Ok(usize::try_from(n)?);
    }
    if let Some(s) = x.as_str() {
        return Ok(s.parse()?);
    }
    Err(fail(format!("missing/invalid integer: {key}")))
}
fn optional_string(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_owned)
}
fn parse_page(
    bytes: &[u8],
    requested_start: usize,
    expected_total: Option<usize>,
) -> Result<(usize, Vec<TableCandidate>, Option<usize>), Box<dyn Error>> {
    let doc: Value = serde_json::from_slice(bytes)?;
    let root = doc.get("GET_STATS_LIST").ok_or("GET_STATS_LIST absent")?;
    if integer(&root["RESULT"], "STATUS")? != 0 {
        return Err(fail("e-Stat application status is not success"));
    }
    if root["PARAMETER"]["STATS_CODE"].as_str() != Some(STATS_CODE) {
        return Err(fail("response statistics code mismatch"));
    }
    let data = &root["DATALIST_INF"];
    let total = integer(data, "NUMBER")?;
    if let Some(prior) = expected_total {
        if prior != total {
            return Err(fail("catalogue total changed mid-run"));
        }
    }
    let info = &data["RESULT_INF"];
    let from = integer(info, "FROM_NUMBER")?;
    let to = integer(info, "TO_NUMBER")?;
    if from != requested_start || to < from || to > total {
        return Err(fail("invalid pagination range"));
    }
    let rows = data["TABLE_INF"]
        .as_array()
        .ok_or("TABLE_INF must be an array")?;
    if rows.len() != to - from + 1 {
        return Err(fail("pagination count mismatch"));
    }
    let mut candidates = Vec::with_capacity(rows.len());
    for row in rows {
        let id = row["@id"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or("missing table ID")?
            .to_owned();
        let title = row["TITLE"]["$"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or("missing table title")?
            .to_owned();
        if row["STAT_NAME"]["@code"].as_str() != Some(STATS_CODE) {
            return Err(fail("table statistics code mismatch"));
        }
        candidates.push(TableCandidate {
            id,
            title,
            statistics_name: optional_string(row, "STATISTICS_NAME"),
            survey_date: optional_string(row, "SURVEY_DATE"),
            open_date: optional_string(row, "OPEN_DATE"),
            updated_date: optional_string(row, "UPDATED_DATE"),
            table_sub_category1: optional_string(&row["TITLE_SPEC"], "TABLE_SUB_CATEGORY1"),
        });
    }
    let next = match info.get("NEXT_KEY") {
        None | Some(Value::Null) => None,
        Some(_) => Some(integer(info, "NEXT_KEY")?),
    };
    if to == total {
        if next.is_some() {
            return Err(fail("continuation after catalogue end"));
        }
    } else if next != Some(to + 1) {
        return Err(fail("missing or discontinuous NEXT_KEY"));
    }
    Ok((total, candidates, next))
}

fn assemble<'a>(pages: impl IntoIterator<Item = &'a [u8]>) -> Result<Catalogue, Box<dyn Error>> {
    let mut total = None;
    let mut next = Some(1);
    let mut all = Vec::new();
    let mut seen = HashSet::new();
    let mut count = 0;
    for bytes in pages {
        if count >= MAX_PAGES {
            return Err(fail("page safety limit exceeded"));
        }
        let start = next.ok_or("extra page after completion")?;
        let (n, rows, continuation) = parse_page(bytes, start, total)?;
        total = Some(n);
        for row in rows {
            if !seen.insert(row.id.clone()) {
                return Err(fail(format!("duplicate table ID: {}", row.id)));
            }
            all.push(row);
        }
        count += 1;
        next = continuation;
    }
    let declared_total = total.ok_or("no captured pages")?;
    if next.is_some() || all.len() != declared_total {
        return Err(fail("incomplete catalogue"));
    }
    Ok(Catalogue {
        stats_code: STATS_CODE.into(),
        declared_total,
        candidates: all,
        pages: count,
    })
}

fn read_limited(response: reqwest::blocking::Response) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    response
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BYTES {
        return Err(fail("response exceeds byte limit"));
    }
    Ok(bytes)
}

fn fetch(
    client: &Client,
    app_id: &str,
    start: usize,
) -> Result<(Vec<u8>, PageMetadata), Box<dyn Error>> {
    let params = vec![
        ("statsCode".to_string(), STATS_CODE.to_string()),
        ("limit".to_string(), PAGE_SIZE.to_string()),
        ("startPosition".to_string(), start.to_string()),
    ];
    let requested_unix_seconds = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let started = Instant::now();
    for attempt in 1..=3 {
        let result = client
            .get(ENDPOINT)
            .query(&[
                ("appId", app_id),
                ("statsCode", STATS_CODE),
                ("limit", &PAGE_SIZE.to_string()),
                ("startPosition", &start.to_string()),
            ])
            .send();
        match result {
            Ok(response) => {
                let status = response.status();
                let bytes = read_limited(response)?;
                if (status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error())
                    && attempt < 3
                {
                    thread::sleep(Duration::from_secs(1 << (attempt - 1)));
                    continue;
                }
                let metadata = PageMetadata {
                    endpoint: ENDPOINT.into(),
                    parameters: params,
                    requested_unix_seconds,
                    duration_ms: started.elapsed().as_millis(),
                    http_status: status.as_u16(),
                    byte_count: bytes.len(),
                    sha256: sha(&bytes),
                    attempts: attempt,
                };
                return Ok((bytes, metadata));
            }
            Err(_) if attempt < 3 => thread::sleep(Duration::from_secs(1 << (attempt - 1))),
            Err(_) => return Err(fail("request failed; URL and credentials suppressed")),
        }
    }
    unreachable!()
}

fn page_files(run: &Path) -> Result<Vec<(PathBuf, PathBuf)>, Box<dyn Error>> {
    let dir = run.join("catalogue_pages");
    let mut result = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        if path.extension().and_then(|x| x.to_str()) != Some("bin") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|x| x.to_str())
            .ok_or("invalid page filename")?;
        let index: usize = name.parse()?;
        result.push((index, path));
    }
    result.sort_by_key(|(i, _)| *i);
    if result.is_empty() {
        return Err(fail("no saved pages"));
    }
    for (position, (index, _)) in result.iter().enumerate() {
        if *index != position + 1 {
            return Err(fail("missing page capture"));
        }
    }
    Ok(result
        .into_iter()
        .map(|(_, bin)| {
            let meta = bin.with_extension("json");
            (bin, meta)
        })
        .collect())
}

pub fn replay(run: &Path) -> Result<Catalogue, Box<dyn Error>> {
    let mut data = Vec::new();
    for (page_index, (bin, meta)) in page_files(run)?.into_iter().enumerate() {
        let m: PageMetadata = serde_json::from_slice(&fs::read(meta)?)?;
        let bytes = fs::read(bin)?;
        if m.endpoint != ENDPOINT
            || m.http_status != 200
            || m.byte_count != bytes.len()
            || m.sha256 != sha(&bytes)
        {
            return Err(fail("captured page metadata/checksum/status mismatch"));
        }
        if m.parameters.iter().any(|(k, _)| k == "appId") {
            return Err(fail("credential in page metadata"));
        }
        let expected_start = (page_index * PAGE_SIZE + 1).to_string();
        let has =
            |name: &str, value: &str| m.parameters.iter().any(|(k, v)| k == name && v == value);
        if !has("statsCode", STATS_CODE)
            || !has("limit", &PAGE_SIZE.to_string())
            || !has("startPosition", &expected_start)
        {
            return Err(fail("captured request parameters mismatch"));
        }
        data.push(bytes);
    }
    let result = assemble(data.iter().map(Vec::as_slice))?;
    println!(
        "Verified {} tables across {} pages",
        result.candidates.len(),
        result.pages
    );
    Ok(result)
}

pub fn inspect(app_id: &str) -> Result<(), Box<dyn Error>> {
    if app_id.is_empty() {
        return Err(fail("ESTAT_APP_ID is empty"));
    }
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let run = Path::new(ROOT).join(format!(
        "catalogue-{}-{}-{}",
        time.as_secs(),
        time.subsec_nanos(),
        std::process::id()
    ));
    let dir = run.join("catalogue_pages");
    fs::create_dir_all(&dir)?;
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()?;
    let mut next = Some(1);
    let mut total = None;
    let mut ids = HashSet::new();
    let mut page = 0;
    while let Some(start) = next {
        if page >= MAX_PAGES {
            return Err(fail("page safety limit exceeded"));
        }
        let (bytes, metadata) = fetch(&client, app_id, start)?;
        page += 1;
        fs::write(dir.join(format!("{page:05}.bin")), &bytes)?;
        fs::write(
            dir.join(format!("{page:05}.json")),
            serde_json::to_vec_pretty(&metadata)?,
        )?;
        if metadata.http_status != 200 {
            return Err(fail(format!(
                "HTTP {} captured in {}",
                metadata.http_status,
                run.display()
            )));
        }
        let (count, rows, continuation) = parse_page(&bytes, start, total)?;
        total = Some(count);
        for row in rows {
            if !ids.insert(row.id) {
                return Err(fail("duplicate table ID"));
            }
        }
        next = continuation;
    }
    let result = replay(&run)?;
    fs::write(
        run.join("catalogue.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    println!("Catalogue saved to {}", run.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn page(from: usize, total: usize, next: Option<usize>, ids: &[&str]) -> Vec<u8> {
        let rows: Vec<Value> = ids.iter().map(|id| serde_json::json!({"@id":id,"TITLE":{"$":"だいこん"},"STAT_NAME":{"@code":STATS_CODE}})).collect();
        serde_json::to_vec(&serde_json::json!({"GET_STATS_LIST":{"RESULT":{"STATUS":0},"PARAMETER":{"STATS_CODE":STATS_CODE},"DATALIST_INF":{"NUMBER":total,"RESULT_INF":{"FROM_NUMBER":from,"TO_NUMBER":from+ids.len()-1,"NEXT_KEY":next},"TABLE_INF":rows}}})).unwrap()
    }
    #[test]
    fn complete_pagination() {
        let a = page(1, 3, Some(3), &["a", "b"]);
        let b = page(3, 3, None, &["c"]);
        let result = assemble([a.as_slice(), b.as_slice()]).unwrap();
        assert_eq!(result.declared_total, 3);
        assert_eq!(result.candidates[2].id, "c");
    }
    #[test]
    fn rejects_incomplete_and_duplicate_and_gap() {
        let a = page(1, 3, Some(3), &["a", "b"]);
        assert!(assemble([a.as_slice()]).is_err());
        let duplicate = page(3, 3, None, &["b"]);
        assert!(assemble([a.as_slice(), duplicate.as_slice()]).is_err());
        let gap = page(4, 4, None, &["d"]);
        assert!(assemble([a.as_slice(), gap.as_slice()]).is_err());
    }
    #[test]
    fn rejects_application_error_and_changed_total() {
        let a = page(1, 2, Some(2), &["a"]);
        let b = page(2, 3, Some(3), &["b"]);
        assert!(assemble([a.as_slice(), b.as_slice()]).is_err());
        let mut bad: Value = serde_json::from_slice(&a).unwrap();
        bad["GET_STATS_LIST"]["RESULT"]["STATUS"] = serde_json::json!(1);
        assert!(assemble([serde_json::to_vec(&bad).unwrap().as_slice()]).is_err());
    }
}
