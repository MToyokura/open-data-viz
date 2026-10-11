use regex::Regex;
use std::{
    collections::BTreeMap,
    error::Error,
    path::Path,
    process::{Command, Stdio},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn xml_unescape(value: &str) -> Result<String> {
    let mut output = String::with_capacity(value.len());
    let mut remaining = value;
    while let Some(start) = remaining.find('&') {
        output.push_str(&remaining[..start]);
        remaining = &remaining[start + 1..];
        let end = remaining
            .find(';')
            .ok_or_else(|| format!("unterminated XML entity in {value:?}"))?;
        let entity = &remaining[..end];
        let decoded = match entity {
            "amp" => '&',
            "apos" => '\'',
            "quot" => '"',
            "lt" => '<',
            "gt" => '>',
            _ if entity.starts_with("#x") => char::from_u32(u32::from_str_radix(&entity[2..], 16)?)
                .ok_or_else(|| format!("invalid XML character reference &{entity};"))?,
            _ if entity.starts_with('#') => char::from_u32(entity[1..].parse()?)
                .ok_or_else(|| format!("invalid XML character reference &{entity};"))?,
            _ => return Err(format!("unknown XML entity &{entity};").into()),
        };
        output.push(decoded);
        remaining = &remaining[end + 1..];
    }
    output.push_str(remaining);
    Ok(output)
}

fn archive_member(path: &Path, member: &str) -> Result<String> {
    let output = Command::new("unzip")
        .args(["-p"])
        .arg(path)
        .arg(member)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()?;
    if !output.status.success() {
        return Err(format!("could not read {member} from {}", path.display()).into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn text_nodes(xml: &str) -> Result<Vec<String>> {
    let text_re = Regex::new(r"(?s)<t(?:\s[^>]*)?>(.*?)</t>")?;
    text_re
        .captures_iter(xml)
        .map(|capture| xml_unescape(&capture[1]))
        .collect()
}

pub fn spreadsheet_cells(path: &Path) -> Result<BTreeMap<String, String>> {
    let shared_xml = archive_member(path, "xl/sharedStrings.xml").ok();
    let shared_strings = if let Some(xml) = shared_xml {
        let item_re = Regex::new(r"(?s)<si(?:\s[^>]*)?>(.*?)</si>")?;
        item_re
            .captures_iter(&xml)
            .map(|capture| Ok(text_nodes(&capture[1])?.concat()))
            .collect::<Result<Vec<_>>>()?
    } else {
        Vec::new()
    };
    let sheet = archive_member(path, "xl/worksheets/sheet1.xml")?;
    let cell_re = Regex::new(r"(?s)<c\b([^>]*)>(.*?)</c>")?;
    let reference_re = Regex::new(r#"\br="([A-Z]+[0-9]+)""#)?;
    let type_re = Regex::new(r#"\bt="([^"]+)""#)?;
    let value_re = Regex::new(r"(?s)<v(?:\s[^>]*)?>(.*?)</v>")?;
    let mut cells = BTreeMap::new();

    for cell in cell_re.captures_iter(&sheet) {
        let attributes = &cell[1];
        let reference = reference_re
            .captures(attributes)
            .ok_or("XLSX cell is missing its reference")?[1]
            .to_owned();
        let body = &cell[2];
        let value = if let Some(value) = value_re.captures(body) {
            let value = xml_unescape(&value[1])?;
            if type_re
                .captures(attributes)
                .is_some_and(|capture| &capture[1] == "s")
            {
                shared_strings
                    .get(value.parse::<usize>()?)
                    .ok_or("XLSX shared string index is out of range")?
                    .clone()
            } else {
                value
            }
        } else if body.contains("<is") {
            text_nodes(body)?.concat()
        } else {
            continue;
        };
        cells.insert(reference, value);
    }
    Ok(cells)
}

pub fn is_zip_archive(bytes: &[u8]) -> bool {
    if bytes.len() < 22 {
        return false;
    }
    let search_start = bytes.len().saturating_sub(65_557);
    bytes[search_start..]
        .windows(4)
        .any(|window| window == b"PK\x05\x06")
}

pub fn column_name(mut number: usize) -> String {
    let mut result = String::new();
    while number > 0 {
        number -= 1;
        result.insert(0, char::from(b'A' + (number % 26) as u8));
        number /= 26;
    }
    result
}
