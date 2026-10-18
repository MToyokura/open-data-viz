use serde_json::Value;
use std::collections::BTreeMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const ALLOWED_EXTENSIONS: &[&str] = &["json", "geojson"];
const CHUNK_SIZE: usize = 1024 * 1024;

pub fn run(args: &[String]) -> bool {
    let Some(command) = args.first().map(String::as_str) else {
        return false;
    };
    let result = match command {
        "gzip-stage" => run_stage(&args[1..]),
        "gzip-reconcile" => run_reconcile(&args[1..]),
        _ => return false,
    };
    if let Err(error) = result {
        eprintln!("gzip publication error: {error}");
        std::process::exit(1);
    }
    true
}

fn run_stage(args: &[String]) -> Result<()> {
    if args.len() != 2 {
        return Err("usage: gzip-stage PUBLISHED_COPY_DIR STAGE_DIR".into());
    }
    let source_arg = Path::new(&args[0]);
    let stage_arg = Path::new(&args[1]);
    reject_symlink(source_arg)?;
    if !source_arg.is_dir() {
        return Err(format!("input directory does not exist: {}", source_arg.display()).into());
    }
    let source = source_arg.canonicalize()?;
    let stage = absolute_path(stage_arg)?;
    if source == stage || source.starts_with(&stage) || stage.starts_with(&source) {
        return Err("input and stage directories must not contain one another".into());
    }
    if stage.exists() || fs::symlink_metadata(&stage).is_ok() {
        return Err(format!("stage directory already exists: {}", stage.display()).into());
    }

    let gzip_version = gnu_gzip_version("gzip")?;
    let files = collect_files(&source)?;
    if files.is_empty() {
        return Err(format!(
            "input contains no JSON or GeoJSON files: {}",
            source.display()
        )
        .into());
    }

    fs::create_dir_all(&stage)?;
    let result = (|| -> Result<()> {
        println!("Using {gzip_version}");
        for (relative, original) in &files {
            let bytes = fs::read(original)?;
            if bytes.is_empty() {
                return Err(format!("empty input file: {}", original.display()).into());
            }
            serde_json::from_slice::<Value>(&bytes)
                .map_err(|error| format!("invalid JSON in {}: {error}", original.display()))?;

            let destination = stage.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            let output = File::create(&destination)?;
            let output = Command::new("gzip")
                .args([
                    OsStr::new("-n"),
                    OsStr::new("-6"),
                    OsStr::new("-c"),
                    OsStr::new("--"),
                ])
                .arg(original)
                .stdout(Stdio::from(output))
                .stderr(Stdio::piped())
                .spawn()?
                .wait_with_output()?;
            if !output.status.success() {
                return Err(format!(
                    "gzip failed for {}: {}",
                    original.display(),
                    String::from_utf8_lossy(&output.stderr).trim()
                )
                .into());
            }
            verify_gzip_contents(&destination, original)?;
            println!("{}", relative.display());
        }
        println!(
            "Staged and verified {} file(s) in {}",
            files.len(),
            stage.display()
        );
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&stage);
    }
    result
}

fn run_reconcile(args: &[String]) -> Result<()> {
    let options = parse_options(args)?;
    let stage_arg = PathBuf::from(required(&options, "stage-dir")?);
    let source_arg = PathBuf::from(required(&options, "source-dir")?);
    let verification = PathBuf::from(required(&options, "verification-dir")?);
    let remote_prefix = required(&options, "remote-prefix")?;
    let content_type = required(&options, "content-type")?;
    let cache_control = required(&options, "cache-control")?;

    if !valid_remote_prefix(&remote_prefix) {
        return Err("--remote-prefix must be an explicit gs://bucket/prefix without wildcard or trailing slash".into());
    }
    if !cache_control
        .split(',')
        .any(|part| part.trim().eq_ignore_ascii_case("no-transform"))
    {
        return Err("--cache-control must include no-transform".into());
    }
    if content_type != "application/json" && content_type != "application/geo+json" {
        return Err("--content-type must be application/json or application/geo+json".into());
    }

    let (stage_root, staged) = collect_root(&stage_arg, "stage")?;
    let (source_root, sources) = collect_root(&source_arg, "source")?;
    let stage_names: Vec<_> = staged.keys().collect();
    let source_names: Vec<_> = sources.keys().collect();
    if stage_names != source_names {
        return Err(format!(
            "stage/source file sets differ (stage={} files, source={} files)",
            staged.len(),
            sources.len()
        )
        .into());
    }
    for relative in staged.keys() {
        let expected = content_type_for(relative)?;
        if expected != content_type {
            return Err(format!(
                "--content-type {content_type} does not match {} ({expected})",
                relative.display()
            )
            .into());
        }
    }
    reject_symlink(&verification)?;
    if verification.exists() {
        return Err(format!(
            "verification directory must not already exist: {}",
            verification.display()
        )
        .into());
    }
    fs::create_dir_all(&verification)?;

    println!(
        "Reconciling {} enumerated file(s) under {remote_prefix}",
        staged.len()
    );
    for (relative, staged_file) in &staged {
        let source_file = &sources[relative];
        if fs::metadata(source_file)?.len() == 0 {
            return Err(format!("empty source file is not allowed: {}", relative.display()).into());
        }
        verify_gzip_contents(staged_file, source_file)?;
        let (local_size, local_crc32c) = local_hash(staged_file)?;
        let remote = format!(
            "{remote_prefix}/{}",
            relative.to_string_lossy().replace('\\', "/")
        );
        let before = describe(&remote)?;
        let bytes_match = before
            .as_ref()
            .is_some_and(|metadata| metadata.size == local_size && metadata.crc32c == local_crc32c);
        let meta_match = before
            .as_ref()
            .is_some_and(|metadata| metadata_matches(metadata, &content_type, &cache_control));

        let action = if bytes_match && meta_match {
            "unchanged"
        } else if let Some(metadata) = before.as_ref().filter(|_| bytes_match) {
            let output = gcloud(&[
                "storage",
                "objects",
                "update",
                &remote,
                &format!("--content-type={content_type}"),
                "--content-encoding=gzip",
                &format!("--cache-control={cache_control}"),
                &format!("--if-generation-match={}", metadata.generation),
                &format!("--if-metageneration-match={}", metadata.metageneration),
            ])?;
            let _ = output;
            "metadata-updated"
        } else {
            let condition = before.as_ref().map_or_else(
                || "--if-generation-match=0".to_owned(),
                |metadata| format!("--if-generation-match={}", metadata.generation),
            );
            gcloud(&[
                "storage",
                "cp",
                &staged_file.to_string_lossy(),
                &remote,
                &format!("--content-type={content_type}"),
                "--content-encoding=gzip",
                &format!("--cache-control={cache_control}"),
                &condition,
            ])?;
            if before.is_some() {
                "reuploaded"
            } else {
                "uploaded"
            }
        };

        let after = describe(&remote)?
            .ok_or_else(|| format!("object disappeared after {action}: {}", relative.display()))?;
        if action == "metadata-updated" {
            let old = before.as_ref().ok_or("missing pre-update metadata")?;
            if after.size != old.size
                || after.crc32c != old.crc32c
                || after.generation != old.generation
            {
                return Err(format!(
                    "metadata-only update changed object bytes or generation: {}",
                    relative.display()
                )
                .into());
            }
            if after.metageneration <= old.metageneration {
                return Err(format!(
                    "metadata-only update did not increase metageneration: {}",
                    relative.display()
                )
                .into());
            }
        } else if action == "reuploaded" {
            let old = before.as_ref().ok_or("missing pre-upload metadata")?;
            if after.generation == old.generation {
                return Err(
                    format!("reupload did not change generation: {}", relative.display()).into(),
                );
            }
        }
        if after.size != local_size || after.crc32c != local_crc32c {
            return Err(format!(
                "stored size/CRC32C differs from stage after {action}: {}",
                relative.display()
            )
            .into());
        }
        if !metadata_matches(&after, &content_type, &cache_control) {
            return Err(format!(
                "stored metadata differs from requested values after {action}: {}",
                relative.display()
            )
            .into());
        }

        let downloaded = verification.join(relative);
        if let Some(parent) = downloaded.parent() {
            fs::create_dir_all(parent)?;
        }
        gcloud(&[
            "storage",
            "cp",
            "--do-not-decompress",
            &remote,
            &downloaded.to_string_lossy(),
        ])?;
        compare_files(staged_file, &downloaded)?;
        verify_gzip_contents(&downloaded, source_file)?;
        let final_metadata = describe(&remote)?.ok_or_else(|| {
            format!(
                "object disappeared during verification: {}",
                relative.display()
            )
        })?;
        if final_metadata != after {
            return Err(format!(
                "GCS object changed during verification: {}",
                relative.display()
            )
            .into());
        }
        println!(
            "{action}: {} generation={} metageneration={}",
            relative.display(),
            final_metadata.generation,
            final_metadata.metageneration
        );
    }
    println!(
        "Verified {} object(s); raw downloads retained in {}",
        staged.len(),
        verification.display()
    );
    let _ = (stage_root, source_root);
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct ObjectMetadata {
    size: u64,
    crc32c: String,
    generation: u64,
    metageneration: u64,
    content_type: String,
    content_encoding: String,
    cache_control: String,
}

fn metadata_matches(metadata: &ObjectMetadata, content_type: &str, cache_control: &str) -> bool {
    metadata.content_type == content_type
        && metadata.content_encoding.eq_ignore_ascii_case("gzip")
        && metadata.cache_control == cache_control
}

fn describe(remote: &str) -> Result<Option<ObjectMetadata>> {
    let output =
        match gcloud_allow_missing(&["storage", "objects", "describe", remote, "--format=json"])? {
            Some(output) => output,
            None => return Ok(None),
        };
    let value: Value = serde_json::from_slice(&output.stdout)?;
    let object = value.as_object().ok_or("unexpected gcloud describe JSON")?;
    Ok(Some(ObjectMetadata {
        size: number_field(object, &["size"])?,
        crc32c: string_field(object, &["crc32c", "crc32c_hash"])?,
        generation: number_field(object, &["generation"])?,
        metageneration: number_field(object, &["metageneration"])?,
        content_type: optional_string_field(object, &["contentType", "content_type"]),
        content_encoding: optional_string_field(object, &["contentEncoding", "content_encoding"]),
        cache_control: optional_string_field(object, &["cacheControl", "cache_control"]),
    }))
}

fn number_field(object: &serde_json::Map<String, Value>, names: &[&str]) -> Result<u64> {
    let value = names
        .iter()
        .find_map(|name| object.get(*name))
        .ok_or_else(|| format!("missing metadata field {}", names[0]))?;
    value
        .as_u64()
        .or_else(|| value.as_str()?.parse().ok())
        .ok_or_else(|| format!("invalid metadata field {}", names[0]).into())
}

fn string_field(object: &serde_json::Map<String, Value>, names: &[&str]) -> Result<String> {
    names
        .iter()
        .find_map(|name| object.get(*name))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("missing metadata field {}", names[0]).into())
}

fn optional_string_field(object: &serde_json::Map<String, Value>, names: &[&str]) -> String {
    names
        .iter()
        .find_map(|name| object.get(*name))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned()
}

fn local_hash(path: &Path) -> Result<(u64, String)> {
    let output = gcloud(&["storage", "hash", "--skip-md5", &path.to_string_lossy()])?;
    let stdout = String::from_utf8(output.stdout)?;
    let crc = stdout
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            (key.trim() == "crc32c_hash").then(|| value.trim().to_owned())
        })
        .ok_or("could not read CRC32C from gcloud storage hash")?;
    let size = fs::metadata(path)?.len();
    Ok((size, crc))
}

fn gcloud(args: &[&str]) -> Result<std::process::Output> {
    let output = Command::new("gcloud").args(args).output()?;
    if !output.status.success() {
        return Err(format!(
            "gcloud {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(output)
}

fn gcloud_allow_missing(args: &[&str]) -> Result<Option<std::process::Output>> {
    let output = Command::new("gcloud").args(args).output()?;
    if output.status.success() {
        return Ok(Some(output));
    }
    let error = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if error.contains("404")
        || error.contains("NotFound")
        || error.to_lowercase().contains("matched no objects")
    {
        return Ok(None);
    }
    Err(format!("gcloud {} failed: {}", args.join(" "), error.trim()).into())
}

fn parse_options(args: &[String]) -> Result<BTreeMap<String, String>> {
    let mut options = BTreeMap::new();
    let mut index = 0;
    while index < args.len() {
        let key = args[index]
            .strip_prefix("--")
            .ok_or_else(|| format!("expected --option, got {}", args[index]))?;
        if index + 1 >= args.len() || args[index + 1].starts_with("--") {
            return Err(format!("missing value for --{key}").into());
        }
        if options
            .insert(key.to_owned(), args[index + 1].clone())
            .is_some()
        {
            return Err(format!("duplicate option --{key}").into());
        }
        index += 2;
    }
    Ok(options)
}

fn required(options: &BTreeMap<String, String>, name: &str) -> Result<String> {
    options
        .get(name)
        .cloned()
        .ok_or_else(|| format!("missing --{name}").into())
}

fn valid_remote_prefix(prefix: &str) -> bool {
    prefix.starts_with("gs://")
        && !prefix.ends_with('/')
        && !prefix.contains('*')
        && !prefix.contains('?')
        && !prefix.contains('#')
        && !prefix.chars().any(char::is_whitespace)
        && prefix[5..].contains('/')
}

fn collect_root(root_arg: &Path, label: &str) -> Result<(PathBuf, BTreeMap<PathBuf, PathBuf>)> {
    reject_symlink(root_arg)?;
    if !root_arg.is_dir() {
        return Err(format!(
            "{label} must be an existing directory: {}",
            root_arg.display()
        )
        .into());
    }
    let root = root_arg.canonicalize()?;
    let files = collect_files(&root)?;
    if files.is_empty() {
        return Err(format!("{label} contains no JSON/GeoJSON files: {}", root.display()).into());
    }
    Ok((root, files))
}

fn collect_files(root: &Path) -> Result<BTreeMap<PathBuf, PathBuf>> {
    let mut files = BTreeMap::new();
    collect_recursively(root, root, &mut files)?;
    Ok(files)
}

fn collect_recursively(
    root: &Path,
    current: &Path,
    files: &mut BTreeMap<PathBuf, PathBuf>,
) -> Result<()> {
    let mut entries = fs::read_dir(current)?.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(format!("symlink is not allowed: {}", path.display()).into());
        }
        if metadata.is_dir() {
            collect_recursively(root, &path, files)?;
        } else if metadata.is_file() {
            let extension = path
                .extension()
                .and_then(OsStr::to_str)
                .unwrap_or("")
                .to_ascii_lowercase();
            if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
                return Err(format!("unsupported file extension: {}", path.display()).into());
            }
            let relative = path.strip_prefix(root)?.to_path_buf();
            let name = relative
                .to_str()
                .ok_or_else(|| format!("non-UTF-8 file path is not allowed: {}", path.display()))?;
            if name
                .chars()
                .any(|character| matches!(character, '?' | '#' | '\r' | '\n'))
            {
                return Err(format!("unsupported character in relative path: {name}").into());
            }
            files.insert(relative, path);
        } else {
            return Err(format!("non-regular file is not allowed: {}", path.display()).into());
        }
    }
    Ok(())
}

fn content_type_for(path: &Path) -> Result<&'static str> {
    match path
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "json" => Ok("application/json"),
        "geojson" => Ok("application/geo+json"),
        _ => Err(format!("unsupported file extension: {}", path.display()).into()),
    }
}

fn reject_symlink(path: &Path) -> Result<()> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(format!("symlinks are not allowed: {}", path.display()).into());
    }
    Ok(())
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn gnu_gzip_version(binary: &str) -> Result<String> {
    let output = Command::new(binary).arg("--version").output()?;
    if !output.status.success() {
        return Err("could not read gzip version".into());
    }
    let version = String::from_utf8_lossy(&output.stdout);
    let first_line = version.lines().next().unwrap_or("").trim();
    if !first_line.starts_with("gzip ") || !version.contains("Free Software Foundation") {
        return Err(format!("GNU gzip is required; found {first_line:?}").into());
    }
    Ok(first_line.to_owned())
}

fn verify_gzip_contents(gzip_path: &Path, original_path: &Path) -> Result<()> {
    let mut child = Command::new("gzip")
        .args(["-d", "-c", "--"])
        .arg(gzip_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().ok_or("gzip stdout was not piped")?;
    let mut child_stdout = stdout;
    let mut original = File::open(original_path)?;
    let compare = compare_readers(&mut child_stdout, &mut original);
    if compare.is_err() {
        let _ = child.kill();
        let _ = child.wait();
        return compare;
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "gzip verification failed for {}: {}",
            gzip_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(())
}

fn compare_readers(left: &mut impl Read, right: &mut impl Read) -> Result<()> {
    let mut left_buffer = vec![0; CHUNK_SIZE];
    let mut right_buffer = vec![0; CHUNK_SIZE];
    loop {
        let left_size = read_up_to(left, &mut left_buffer)?;
        let right_size = read_up_to(right, &mut right_buffer)?;
        if left_size != right_size || left_buffer[..left_size] != right_buffer[..right_size] {
            return Err("decompressed bytes differ from original".into());
        }
        if left_size == 0 {
            return Ok(());
        }
    }
}

fn read_up_to(reader: &mut impl Read, buffer: &mut [u8]) -> io::Result<usize> {
    let mut filled = 0;
    while filled < buffer.len() {
        match reader.read(&mut buffer[filled..])? {
            0 => break,
            read => filled += read,
        }
    }
    Ok(filled)
}

fn compare_files(left: &Path, right: &Path) -> Result<()> {
    let mut left_file = File::open(left)?;
    let mut right_file = File::open(right)?;
    compare_readers(&mut left_file, &mut right_file)
}
