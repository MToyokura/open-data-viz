use reqwest::blocking::Client;
use reqwest::header::RANGE;
use serde_json::Value;
use std::collections::BTreeSet;
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

const ARCHIVE_NAME: &str = "N03-20260101_GML.zip";
const SOURCE_FILE_NAME: &str = "N03-20260101_prefecture.geojson";
const OUTPUT_FILE_NAME: &str = "N03-20260101_prefecture_simplified.geojson";
const ARCHIVE_URL: &str = "https://nlftp.mlit.go.jp/ksj/gml/data/N03/N03-2026/N03-20260101_GML.zip";
const PREFECTURE_NAMES: [&str; 47] = [
    "北海道",
    "青森県",
    "岩手県",
    "宮城県",
    "秋田県",
    "山形県",
    "福島県",
    "茨城県",
    "栃木県",
    "群馬県",
    "埼玉県",
    "千葉県",
    "東京都",
    "神奈川県",
    "新潟県",
    "富山県",
    "石川県",
    "福井県",
    "山梨県",
    "長野県",
    "岐阜県",
    "静岡県",
    "愛知県",
    "三重県",
    "滋賀県",
    "京都府",
    "大阪府",
    "兵庫県",
    "奈良県",
    "和歌山県",
    "鳥取県",
    "島根県",
    "岡山県",
    "広島県",
    "山口県",
    "徳島県",
    "香川県",
    "愛媛県",
    "高知県",
    "福岡県",
    "佐賀県",
    "長崎県",
    "熊本県",
    "大分県",
    "宮崎県",
    "鹿児島県",
    "沖縄県",
];

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn update_prefectures() -> Result<()> {
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let input_dir = data_dir.join("input/prefectures/N03-2026");
    let archive_path = input_dir.join(ARCHIVE_NAME);
    let source_path = input_dir.join(SOURCE_FILE_NAME);
    let output_dir = data_dir.join("output/prefectures");
    let output_path = output_dir.join(OUTPUT_FILE_NAME);

    fs::create_dir_all(&input_dir)?;
    fs::create_dir_all(&output_dir)?;

    if !archive_path.exists() {
        println!(
            "Downloading {ARCHIVE_NAME} from the MLIT National Land Numerical Information site..."
        );
        download_archive(&archive_path)?;
    } else {
        println!("Using existing archive: {}", archive_path.display());
    }

    if !source_path.exists() {
        println!("Extracting {ARCHIVE_NAME}...");
        let status = Command::new("unzip")
            .arg("-o")
            .arg(&archive_path)
            .arg("-d")
            .arg(&input_dir)
            .status()?;
        if !status.success() {
            return Err(format!("unzip failed with {status}").into());
        }
    }

    if !source_path.exists() {
        return Err(format!(
            "Expected source GeoJSON not found: {}\nCheck the archive contents and update SOURCE_DIR_NAME or SOURCE_FILE_NAME if MLIT changes its layout.",
            source_path.display()
        )
        .into());
    }

    println!("Simplifying prefecture boundaries...");
    let mapshaper = data_dir.join("node_modules/mapshaper/bin/mapshaper");
    let status = Command::new("node")
        .arg(mapshaper)
        .arg(&source_path)
        .args([
            "-simplify",
            "0.5%",
            "keep-shapes",
            "stats",
            "-dissolve",
            "N03_001",
            "-filter-islands",
            "min-area=25km2",
            "-o",
            "force",
            "format=geojson",
            "precision=0.0001",
        ])
        .arg(&output_path)
        .status()?;
    if !status.success() {
        return Err(format!("Mapshaper failed with {status}").into());
    }

    validate_prefecture_geojson(&fs::read_to_string(&output_path)?)?;
    println!("Wrote {}", output_path.display());
    Ok(())
}

fn download_archive(archive_path: &Path) -> Result<()> {
    let partial_path = archive_path.with_extension("zip.part");
    let client = Client::builder()
        .user_agent("open_data_viz data preparation")
        .build()?;

    for attempt in 1..=4 {
        let offset = partial_path
            .metadata()
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        let mut request = client.get(ARCHIVE_URL);
        if offset > 0 {
            request = request.header(RANGE, format!("bytes={offset}-"));
        }

        match request.send() {
            Ok(response) if response.status().as_u16() == 416 && offset > 0 => {
                fs::remove_file(&partial_path)?;
            }
            Ok(mut response) if response.status().is_success() => {
                let append = offset > 0 && response.status().as_u16() == 206;
                let mut file = if append {
                    OpenOptions::new().append(true).open(&partial_path)?
                } else {
                    File::create(&partial_path)?
                };
                match io::copy(&mut response, &mut file) {
                    Ok(_) => {
                        file.sync_all()?;
                        fs::rename(&partial_path, archive_path)?;
                        return Ok(());
                    }
                    Err(error) if attempt < 4 => {
                        eprintln!("Download interrupted ({error}); retrying...");
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            Ok(response) => {
                let status = response.status();
                let message = format!("Download failed: HTTP {status}");
                if attempt == 4 {
                    return Err(message.into());
                }
                eprintln!("{message}; retrying...");
            }
            Err(error) if attempt < 4 => {
                eprintln!("Download failed ({error}); retrying...");
            }
            Err(error) => return Err(error.into()),
        }

        if attempt < 4 {
            thread::sleep(Duration::from_secs(attempt));
        }
    }

    Err("Download failed after 4 attempts".into())
}

fn validate_prefecture_geojson(contents: &str) -> Result<()> {
    let geojson: Value = serde_json::from_str(contents)?;
    if geojson["type"] != "FeatureCollection" {
        return Err("Generated GeoJSON is not a FeatureCollection".into());
    }

    let features = geojson["features"]
        .as_array()
        .ok_or("Generated GeoJSON has no features array")?;
    let actual_names: BTreeSet<&str> = features
        .iter()
        .map(|feature| {
            feature["properties"]["N03_001"]
                .as_str()
                .ok_or("Generated feature is missing the N03_001 prefecture name")
        })
        .collect::<std::result::Result<_, _>>()?;
    let expected_names: BTreeSet<&str> = PREFECTURE_NAMES.into_iter().collect();

    if features.len() != PREFECTURE_NAMES.len() || actual_names != expected_names {
        let missing: Vec<_> = expected_names.difference(&actual_names).copied().collect();
        let unexpected: Vec<_> = actual_names.difference(&expected_names).copied().collect();
        return Err(format!(
            "Generated GeoJSON must contain exactly 47 prefectures (found {} features and {} unique names); missing: {missing:?}; unexpected: {unexpected:?}",
            features.len(),
            actual_names.len()
        )
        .into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{PREFECTURE_NAMES, validate_prefecture_geojson};
    use serde_json::json;

    fn geojson_with_prefectures(names: &[&str]) -> String {
        let features: Vec<_> = names
            .iter()
            .map(|name| {
                json!({
                    "type": "Feature",
                    "properties": { "N03_001": name },
                    "geometry": null
                })
            })
            .collect();
        json!({ "type": "FeatureCollection", "features": features }).to_string()
    }

    #[test]
    fn accepts_all_47_prefectures() {
        let contents = geojson_with_prefectures(&PREFECTURE_NAMES);
        assert!(validate_prefecture_geojson(&contents).is_ok());
    }

    #[test]
    fn rejects_missing_prefecture() {
        let contents = geojson_with_prefectures(&PREFECTURE_NAMES[..46]);
        let error = validate_prefecture_geojson(&contents).unwrap_err();
        assert!(error.to_string().contains("exactly 47 prefectures"));
        assert!(error.to_string().contains("沖縄県"));
    }
}
