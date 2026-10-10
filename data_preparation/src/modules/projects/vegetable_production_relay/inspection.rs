use reqwest::{StatusCode, blocking::Client};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const ENDPOINT: &str = "https://api.e-stat.go.jp/rest/3.0/app/json/getStatsList";
const ROOT: &str = "inspections/vegetable_production_relay/runs";
const MAX_RESPONSE_BYTES: usize = 32 * 1024 * 1024;
const MAX_ATTEMPTS: usize = 3;

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptureMetadata {
    pub endpoint: String,
    pub parameters: Vec<(String, String)>,
    pub requested_unix_seconds: u64,
    pub duration_ms: u128,
    pub http_status: u16,
    pub byte_count: usize,
    pub sha256: String,
    pub attempts: usize,
}

fn checksum(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn create_run_dir() -> Result<PathBuf, Box<dyn Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let path = Path::new(ROOT).join(format!(
        "{}-{}-{}",
        now.as_secs(),
        now.subsec_nanos(),
        std::process::id()
    ));
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn save_new(path: &Path, data: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(data)?;
    file.sync_all()?;
    Ok(())
}

fn read_bounded(reader: impl Read) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    reader
        .take((MAX_RESPONSE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err("response exceeded inspection size limit".into());
    }
    Ok(bytes)
}

fn capture_response(
    dir: &Path,
    bytes: &[u8],
    status: StatusCode,
    params: Vec<(String, String)>,
    requested_unix_seconds: u64,
    elapsed: Duration,
    attempts: usize,
) -> Result<(), Box<dyn Error>> {
    let metadata = CaptureMetadata {
        endpoint: ENDPOINT.to_string(),
        parameters: params,
        requested_unix_seconds,
        duration_ms: elapsed.as_millis(),
        http_status: status.as_u16(),
        byte_count: bytes.len(),
        sha256: checksum(bytes),
        attempts,
    };
    save_new(&dir.join("response.bin"), bytes)?;
    save_new(
        &dir.join("metadata.json"),
        &serde_json::to_vec_pretty(&metadata)?,
    )?;
    Ok(())
}

/// Capture one bounded getStatsList response; never serialize the app ID or raw URL.
pub fn inspect_stats_list(app_id: &str) -> Result<(), Box<dyn Error>> {
    if app_id.is_empty() {
        return Err("ESTAT_APP_ID is empty".into());
    }
    let dir = create_run_dir()?;
    let params = vec![
        ("statsCode".to_string(), "00500226".to_string()),
        ("limit".to_string(), "10".to_string()),
    ];
    let started_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let started = Instant::now();
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()?;
    for attempt in 1..=MAX_ATTEMPTS {
        let response = client
            .get(ENDPOINT)
            .query(&[
                ("appId", app_id),
                ("statsCode", "00500226"),
                ("limit", "10"),
            ])
            .send();
        match response {
            Ok(response) => {
                let status = response.status();
                let bytes = read_bounded(response)?;
                if (status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error())
                    && attempt < MAX_ATTEMPTS
                {
                    thread::sleep(Duration::from_secs(1 << (attempt - 1)));
                    continue;
                }
                capture_response(
                    &dir,
                    &bytes,
                    status,
                    params,
                    started_at,
                    started.elapsed(),
                    attempt,
                )?;
                println!("Inspection saved to {}", dir.display());
                if !status.is_success() {
                    return Err(format!(
                        "e-Stat returned HTTP {} (response captured)",
                        status.as_u16()
                    )
                    .into());
                }
                return Ok(());
            }
            Err(_error) if attempt < MAX_ATTEMPTS => {
                thread::sleep(Duration::from_secs(1 << (attempt - 1)));
            }
            Err(_error) => {
                save_new(
                    &dir.join("error.txt"),
                    b"Request failed before an HTTP response was received\n",
                )?;
                return Err("request failed (details suppressed to protect credentials)".into());
            }
        }
    }
    unreachable!()
}

/// Offline integrity check: does not construct a network client or read credentials.
pub fn replay(dir: &Path) -> Result<(), Box<dyn Error>> {
    let metadata: CaptureMetadata = serde_json::from_slice(&fs::read(dir.join("metadata.json"))?)?;
    let bytes = fs::read(dir.join("response.bin"))?;
    if bytes.len() != metadata.byte_count || checksum(&bytes) != metadata.sha256 {
        return Err("inspection response checksum/size mismatch".into());
    }
    println!(
        "Verified {} bytes (HTTP {})",
        bytes.len(),
        metadata.http_status
    );
    if !(200..300).contains(&metadata.http_status) {
        return Err(format!("captured response was HTTP {}", metadata.http_status).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksums_are_sensitive_to_changes() {
        assert_eq!(
            checksum(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_ne!(checksum(b"abc"), checksum(b"abd"));
    }

    #[test]
    fn bounded_reader_rejects_oversized_input() {
        assert!(read_bounded(std::io::repeat(0)).is_err());
    }

    #[test]
    fn captured_failure_replays_as_failure_and_detects_tampering() {
        let dir = std::env::temp_dir().join(format!(
            "vegetable-inspection-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        let result = (|| {
            capture_response(
                &dir,
                b"failure",
                StatusCode::BAD_REQUEST,
                vec![],
                0,
                Duration::ZERO,
                1,
            )
            .unwrap();
            assert!(replay(&dir).is_err());
            assert!(
                !fs::read_to_string(dir.join("metadata.json"))
                    .unwrap()
                    .contains("appId")
            );
            fs::write(dir.join("response.bin"), b"tampered").unwrap();
            assert!(
                replay(&dir)
                    .unwrap_err()
                    .to_string()
                    .contains("checksum/size mismatch")
            );
        })();
        fs::remove_dir_all(&dir).unwrap();
        result
    }
}
