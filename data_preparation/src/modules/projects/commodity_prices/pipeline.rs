//! Deterministic Commodity prices bundle generation and capture replay.
use crate::modules::estat_api::{
    EstatClient,
    get_meta_info::{GetMetaInfoParamsBuilder, MandatoryParams},
    get_stats_data::{GetStatsDataParams, StatsDataId, StatsDataOptions},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

const TABLE_ID: &str = "0003421913";
const ROOT: &str = "output/commodity_prices";
const CHUNK: usize = 10;
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn remove(path: &Path) -> Result<(), Box<dyn Error>> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}
fn classes(v: &Value) -> Vec<&Value> {
    match v {
        Value::Array(a) => a.iter().collect(),
        Value::Object(_) => vec![v],
        _ => vec![],
    }
}
fn number(v: &Value) -> Option<u64> {
    v.as_u64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}
fn validate_response_page<'a>(
    page: &'a Value,
    code: &str,
    start: u32,
) -> Result<(Vec<&'a Value>, u64, Option<u32>), Box<dyn Error>> {
    let stats = &page["GET_STATS_DATA"]["STATISTICAL_DATA"];
    let info = &stats["RESULT_INF"];
    let total = number(&info["TOTAL_NUMBER"]).ok_or("e-Stat response missing TOTAL_NUMBER")?;
    let vals = classes(&stats["DATA_INF"]["VALUE"]);
    if total == 0 {
        if !vals.is_empty() {
            return Err("Zero-row response contained values".into());
        }
        return Ok((vals, 0, None));
    }
    if vals.is_empty() {
        return Err(format!("Unexpected empty page for item {code} at {start}").into());
    }
    let from = number(&info["FROM_NUMBER"]).ok_or("Missing FROM_NUMBER")?;
    let to = number(&info["TO_NUMBER"]).ok_or("Missing TO_NUMBER")?;
    if from != start as u64 || to != from + vals.len() as u64 - 1 || to > total {
        return Err(format!("Invalid page boundary for {code}: {from}-{to}/{total}").into());
    }
    for value in &vals {
        if value["@cat02"].as_str() != Some(code) {
            return Err(format!("Response identity mismatch for requested item {code}").into());
        }
    }
    let next = if to == total {
        None
    } else {
        let n = number(&info["NEXT_KEY"]).ok_or("Missing NEXT_KEY on non-final page")?;
        if n != to + 1 {
            return Err(format!("Invalid NEXT_KEY for item {code}").into());
        }
        Some(u32::try_from(n)?)
    };
    Ok((vals, total, next))
}
fn class_list(meta: &Value, id: &str) -> Vec<(String, String)> {
    let objs = classes(&meta["GET_META_INFO"]["METADATA_INF"]["CLASS_INF"]["CLASS_OBJ"]);
    objs.into_iter()
        .find(|o| o["@id"] == id)
        .map(|o| {
            classes(&o["CLASS"])
                .into_iter()
                .filter_map(|c| {
                    Some((
                        c["@code"].as_str()?.to_owned(),
                        c["@name"].as_str()?.to_owned(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Replay a verified capture and atomically replace only the versioned browser bundle.
pub fn transform() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(ROOT);
    let raw = root.join("raw_data");
    let meta_dir = root.join("meta_data");
    let record_path = meta_dir.join("acquisition.json");
    if !record_path.exists() {
        return Err("No verified Commodity prices capture. Run a full refresh first.".into());
    }
    let record: Value = serde_json::from_slice(&fs::read(&record_path)?)?;
    if record["tableId"] != TABLE_ID {
        return Err(format!("Capture table ID does not match {TABLE_ID}").into());
    }
    let metadata_path = raw.join("get_commodity_meta_info.json");
    let metadata_bytes = fs::read(&metadata_path)?;
    verify_file(&record, "metadata", &metadata_path, &metadata_bytes)?;
    let metadata: Value = serde_json::from_slice(&metadata_bytes)?;
    let items = class_list(&metadata, "cat02");
    let areas = class_list(&metadata, "area");
    if items.is_empty() || areas.is_empty() {
        return Err("Capture metadata has no item or area classifications".into());
    }
    let staging = root.join(".staging");
    let bundle = staging.join("bundle");
    remove(&staging)?;
    fs::create_dir_all(bundle.join("meta_data"))?;
    fs::create_dir_all(bundle.join("transformed_data"))?;
    let item_lookup: Vec<Value> = items
        .iter()
        .map(|(code, name)| json!({"cat02_code":code,"cat02_name":name}))
        .collect();
    let area_lookup: Vec<Value> = areas
        .iter()
        .map(|(code, name)| json!({"area_code":code,"area_name":name}))
        .collect();
    fs::write(
        bundle.join("meta_data/cat02_lookup.json"),
        serde_json::to_vec_pretty(&item_lookup)?,
    )?;
    fs::write(
        bundle.join("meta_data/area_lookup.json"),
        serde_json::to_vec_pretty(&area_lookup)?,
    )?;
    let item_index: BTreeMap<String, usize> = items
        .iter()
        .enumerate()
        .map(|(i, (c, _))| (c.clone(), i))
        .collect();
    let area_names: BTreeMap<String, String> = areas.iter().cloned().collect();
    let entries = record["items"]
        .as_array()
        .ok_or("Capture record has no item pages")?;
    if entries.len() != items.len() {
        return Err("Capture record item count does not match metadata".into());
    }
    let mut entries_by_chunk = vec![BTreeMap::new(); items.len().div_ceil(CHUNK)];
    for entry in entries {
        let code = entry["code"].as_str().ok_or("Capture item missing code")?;
        let idx = *item_index
            .get(code)
            .ok_or("Capture record contains unknown item")?;
        if entries_by_chunk[idx / CHUNK].insert(code, entry).is_some() {
            return Err(format!("Duplicate capture item {code}").into());
        }
    }
    for (group, entries) in entries_by_chunk.into_iter().enumerate() {
        let mut chunks: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        for (code, entry) in entries {
            let idx = item_index[code];
            let mut grouped: BTreeMap<String, Vec<Value>> = BTreeMap::new();
            let mut observations = HashSet::new();
            let pages = entry["pages"]
                .as_array()
                .ok_or("Capture item missing pages")?;
            let mut seen = 0usize;
            for page in pages {
                let rel = page["path"].as_str().ok_or("Capture page missing path")?;
                let path = raw.join(rel);
                let bytes = fs::read(&path)?;
                verify_page(page, &bytes)?;
                let data: Value = serde_json::from_slice(&bytes)?;
                let vals =
                    classes(&data["GET_STATS_DATA"]["STATISTICAL_DATA"]["DATA_INF"]["VALUE"]);
                if page["rowCount"].as_u64() != Some(vals.len() as u64) {
                    return Err(format!("Row count mismatch in {rel}").into());
                }
                seen += vals.len();
                for v in vals {
                    let item = v["@cat02"].as_str().ok_or("Observation missing @cat02")?;
                    if item != code {
                        return Err(format!(
                            "Observation item {item} differs from requested {code}"
                        )
                        .into());
                    }
                    let area = v["@area"].as_str().ok_or("Observation missing @area")?;
                    let time = v["@time"].as_str().ok_or("Observation missing @time")?;
                    if !observations.insert((area.to_owned(), time.to_owned())) {
                        return Err(format!(
                            "Duplicate observation for item {code}, area {area}, time {time}"
                        )
                        .into());
                    }
                    area_names
                        .get(area)
                        .ok_or("Observation area is absent from metadata")?;
                    grouped
                        .entry(area.to_owned())
                        .or_default()
                        .push(json!({"time":time,"value":v["$"]}));
                }
            }
            if entry["rowCount"].as_u64() != Some(seen as u64) {
                return Err(format!("Capture total row count mismatch for {code}").into());
            }
            for (area, pairs) in grouped {
                let series = json!({"cat02_code":code,"cat02_name":items[idx].1,"area_code":area,"area_name":area_names[&area],"time_value_pairs":pairs});
                chunks.entry(area).or_default().push(series);
            }
        }
        let first = group * CHUNK;
        let last = (first + CHUNK).min(items.len()) - 1;
        for (area, series) in chunks {
            let name = format!(
                "area_{area}_items_{}_{}.json",
                items[first].0, items[last].0
            );
            fs::write(
                bundle.join("transformed_data").join(name),
                serde_json::to_vec(&series)?,
            )?;
        }
    }
    let active = root.join("v2");
    let backup = root.join(".previous_v2");
    remove(&backup)?;
    if active.exists() {
        fs::rename(&active, &backup)?;
    }
    if let Err(e) = fs::rename(&bundle, &active) {
        if backup.exists() {
            fs::rename(&backup, &active)?;
        }
        return Err(e.into());
    }
    remove(&backup)?;
    remove(&staging)?;
    Ok(())
}

/// Acquire a fresh verified capture, then build and promote its matching bundle.
pub fn acquire(app_id: &str) -> Result<(), Box<dyn Error>> {
    if app_id.is_empty() {
        return Err("ESTAT_APP_ID is required".into());
    }
    let client = EstatClient::new()?;
    let root = PathBuf::from(ROOT);
    let stage = root.join(".staging/capture");
    remove(&stage)?;
    let raw = stage.join("raw_data");
    let meta_dir = stage.join("meta_data");
    fs::create_dir_all(&raw)?;
    fs::create_dir_all(&meta_dir)?;
    let metadata = client.get_meta_info(
        GetMetaInfoParamsBuilder::new(MandatoryParams {
            app_id,
            stats_data_id: TABLE_ID,
        })
        .build(),
    )?;
    let mut mv: Value = serde_json::from_str(&metadata)?;
    redact(&mut mv);
    let metadata_bytes = serde_json::to_vec(&mv)?;
    fs::write(raw.join("get_commodity_meta_info.json"), &metadata_bytes)?;
    let items = class_list(&mv, "cat02");
    if items.is_empty() {
        return Err("e-Stat metadata contained no cat02 items".into());
    }
    let mut entries = Vec::new();
    for (code, _) in &items {
        let mut pages = Vec::new();
        let mut start = 1u32;
        let mut total_seen = None;
        let mut row_count = 0u64;
        loop {
            let response = client.get_stats_data(GetStatsDataParams::new(
                app_id,
                StatsDataId::Table(TABLE_ID),
                StatsDataOptions {
                    cd_cat02: Some(code),
                    start_position: Some(start),
                    limit: Some(100000),
                    ..Default::default()
                },
            ))?;
            let mut page: Value = serde_json::from_str(&response)?;
            redact(&mut page);
            let (vals, total, next) = validate_response_page(&page, code, start)?;
            if total_seen.is_some_and(|n| n != total) {
                return Err(format!("TOTAL_NUMBER changed while paging item {code}").into());
            }
            total_seen = Some(total);
            if total == 0 {
                if !vals.is_empty() {
                    return Err("Zero-row response contained values".into());
                }
                break;
            }
            let name = format!("item_{code}_{start:010}.json");
            let bytes = serde_json::to_vec(&page)?;
            fs::write(raw.join(&name), &bytes)?;
            row_count += vals.len() as u64;
            pages.push(json!({"path":name,"rowCount":vals.len(),"sha256":sha(&bytes)}));
            let Some(next) = next else { break };
            start = next;
        }
        entries
            .push(json!({"code":code,"rowCount":row_count,"pages":pages,"zeroRows":row_count==0}));
    }
    let acquired = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let acquisition = json!({"tableId":TABLE_ID,"acquiredAtUnix":acquired,"files":{"metadata":{"path":"get_commodity_meta_info.json","sha256":sha(&metadata_bytes)}},"items":entries});
    fs::write(
        meta_dir.join("acquisition.json"),
        serde_json::to_vec_pretty(&acquisition)?,
    )?;
    // Swap the verified capture into place; restore it if bundle validation fails.
    let old_raw = root.join(".previous_raw");
    let old_meta = root.join(".previous_meta");
    remove(&old_raw)?;
    remove(&old_meta)?;
    if root.join("raw_data").exists() {
        fs::rename(root.join("raw_data"), &old_raw)?;
    }
    if root.join("meta_data").exists() {
        fs::rename(root.join("meta_data"), &old_meta)?;
    }
    fs::rename(&raw, root.join("raw_data"))?;
    fs::rename(&meta_dir, root.join("meta_data"))?;
    if let Err(e) = transform() {
        remove(&root.join("raw_data"))?;
        remove(&root.join("meta_data"))?;
        if old_raw.exists() {
            fs::rename(&old_raw, root.join("raw_data"))?
        }
        if old_meta.exists() {
            fs::rename(&old_meta, root.join("meta_data"))?
        }
        return Err(e);
    }
    remove(&old_raw)?;
    remove(&old_meta)?;
    remove(&stage)?;
    Ok(())
}
fn redact(v: &mut Value) {
    match v {
        Value::Object(m) => {
            m.remove("appId");
            m.remove("@appId");
            for x in m.values_mut() {
                redact(x)
            }
        }
        Value::Array(a) => {
            for x in a {
                redact(x)
            }
        }
        _ => {}
    }
}
fn verify_file(record: &Value, key: &str, path: &Path, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let files = record["files"]
        .as_object()
        .ok_or("Capture record missing file checksums")?;
    let m = &files[key];
    if m["path"].as_str() != Some(path.file_name().unwrap().to_str().unwrap())
        || m["sha256"].as_str() != Some(&sha(bytes))
    {
        return Err("Capture metadata checksum mismatch".into());
    }
    Ok(())
}
fn verify_page(page: &Value, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    if page["sha256"].as_str() != Some(&sha(bytes)) {
        return Err("Capture page checksum mismatch".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(total: u64, from: u64, to: u64, values: Value, next: Option<u64>) -> Value {
        let mut info = json!({"TOTAL_NUMBER":total,"FROM_NUMBER":from,"TO_NUMBER":to});
        if let Some(n) = next {
            info["NEXT_KEY"] = json!(n)
        }
        json!({"GET_STATS_DATA":{"STATISTICAL_DATA":{"RESULT_INF":info,"DATA_INF":{"VALUE":values}}}})
    }
    #[test]
    fn accepts_complete_response() {
        let page = fixture(2, 1, 2, json!([{"@cat02":"i1"},{"@cat02":"i1"}]), None);
        let (rows, total, next) = validate_response_page(&page, "i1", 1).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(total, 2);
        assert_eq!(next, None);
    }
    #[test]
    fn records_zero_row_response() {
        let page = fixture(0, 0, 0, json!([]), None);
        let (rows, total, next) = validate_response_page(&page, "i1", 1).unwrap();
        assert!(rows.is_empty());
        assert_eq!(total, 0);
        assert_eq!(next, None);
    }
    #[test]
    fn rejects_incomplete_and_mismatched_pages() {
        let missing = fixture(2, 1, 1, json!([{"@cat02":"i1"}]), Some(3));
        assert!(validate_response_page(&missing, "i1", 1).is_err());
        let bad_identity = fixture(1, 1, 1, json!([{"@cat02":"other"}]), None);
        assert!(validate_response_page(&bad_identity, "i1", 1).is_err());
    }
}
