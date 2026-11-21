use std::{collections::BTreeMap, error::Error, fs, path::Path, process::{Command, Output}, time::{SystemTime, UNIX_EPOCH}};

const URL: &str = "https://www.e-stat.go.jp/stat-search/file-download?statInfId=000040416370&fileKind=0";
const ROOT: &str = "output/marine_fishery_catch";

pub fn run(args: &[String]) -> bool {
    if args.first().map(String::as_str) != Some("marine-fishery-catch") { return false; }
    let result = match &args[1..] {
        [] => acquire(),
        [flag] if flag == "--transform-only" => transform(),
        [flag] if flag == "--help" || flag == "-h" => { println!("Usage: marine-fishery-catch [--transform-only]"); return true; }
        _ => { eprintln!("Invalid marine-fishery-catch arguments"); std::process::exit(2); }
    };
    if let Err(error) = result { eprintln!("Marine fishery catch failed: {error}"); std::process::exit(1); }
    true
}

fn acquire() -> Result<(), Box<dyn Error>> {
    let root = Path::new(ROOT);
    let stage = root.join(".staging");
    if stage.exists() { fs::remove_dir_all(&stage)?; }
    fs::create_dir_all(stage.join("raw_data"))?;
    let response = reqwest::blocking::get(URL)?.error_for_status()?;
    let bytes = response.bytes()?;
    if bytes.len() < 10_000 || !bytes.starts_with(b"PK") { return Err("e-Stat response is not a readable XLSX archive".into()); }
    let source = stage.join("raw_data/marine_fishery_catch.xlsx");
    fs::write(&source, &bytes)?;
    fs::write(stage.join("raw_data/capture.json"), format!("{{\"sourceUrl\":\"{URL}\",\"retrievedAtEpochSeconds\":{},\"sha256\":\"{}\"}}\n", SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(), sha256(&bytes)))?;
    transform_to(&source, &stage.join("transformed_data/catches.json"))?;
    commit(root, &stage)?;
    Ok(())
}

fn transform() -> Result<(), Box<dyn Error>> {
    let root = Path::new(ROOT);
    let source = root.join("raw_data/marine_fishery_catch.xlsx");
    if !source.exists() { return Err("saved workbook is missing; run marine-fishery-catch first".into()); }
    let stage = root.join(".staging");
    if stage.exists() { fs::remove_dir_all(&stage)?; }
    fs::create_dir_all(stage.join("transformed_data"))?;
    transform_to(&source, &stage.join("transformed_data/catches.json"))?;
    fs::create_dir_all(root.join("transformed_data"))?;
    fs::rename(stage.join("transformed_data/catches.json"), root.join("transformed_data/catches.json"))?;
    fs::remove_dir_all(stage)?;
    Ok(())
}

fn transform_to(source: &Path, output: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(output.parent().unwrap())?;
    let sheets = unzip_member(source, "xl/worksheets/sheet1.xml")?;
    let shared = unzip_member(source, "xl/sharedStrings.xml")?;
    let strings = parse_shared_strings(std::str::from_utf8(&shared)?)?;
    let rows = parse_sheet(std::str::from_utf8(&sheets)?, &strings)?;
    let cell = |row: usize, col: usize| rows.get(&row).and_then(|r| r.get(&col)).cloned().unwrap_or_default();
    if cell(4, 1) != "全国" { return Err("Sheet1!A4 is not 全国".into()); }
    let cols: Vec<usize> = (2..=112).collect();
    let headings: Vec<Vec<String>> = cols.iter().map(|&c| (5..=7).map(|r| cell(r, c)).collect()).collect();
    let fingerprint = serde_json::to_vec(&headings)?;
    if sha256(&fingerprint) != "bd8503b059b31007ac77085f7ca6ae0dcfbb6202125508acbe8186b1520c722b" { return Err("series headings changed; refusing to transform".into()); }
    if cols.iter().any(|&c| cell(8,c) != "t") { return Err("expected unit t in B8:DH8".into()); }
    let mut years = Vec::new();
    let mut labels = Vec::new();
    for row in 9..=77 {
        let label = cell(row,1);
        let year = parse_year(&label).ok_or_else(|| format!("Sheet1!A{row} missing Gregorian year: {label}"))?;
        years.push(year);
        labels.push(label);
    }
    if years.len()!=69 || years.first()!=Some(&1956) || years.last()!=Some(&2024) || years.windows(2).any(|w|w[0]>=w[1]) { return Err("unexpected or non-ascending year block".into()); }
    let mut path: Vec<String> = Vec::new();
    let mut series = Vec::new();
    let mut nulls = 0usize;
    for (index, (&col, parts)) in cols.iter().zip(headings.iter()).enumerate() {
        for (level, part) in parts.iter().enumerate() { if !part.is_empty() { path.truncate(level); path.push(part.clone()); } }
        let label = parts.iter().rev().find(|p| !p.is_empty()).cloned().unwrap_or_default();
        let kind = if index == 0 { "total" } else if label == "小計" || label == "計" { "subtotal" } else { "detail" };
        let mut values = Vec::new();
        for row in 9..=77 {
            let raw = cell(row,col).trim().to_string();
            if raw.starts_with('…') { values.push(serde_json::Value::Null); nulls+=1; }
            else {
                let value: i64 = raw.replace(',',"").parse().map_err(|_| format!("unexpected value at Sheet1!{}{row}: {raw}", column_name(col)))?;
                values.push(serde_json::json!(value));
            }
        }
        series.push(serde_json::json!({"id":format!("col-{}",column_name(col).to_lowercase()),"column":column_name(col),"label":label,"groupPath":path,"type":kind,"values":values}));
    }
    if series.len()!=111 || nulls!=1114 { return Err(format!("expected 111 series and 1,114 missing markers, got {} and {nulls}",series.len()).into()); }
    if series[0]["values"][0] != 4511923 || series[0]["values"][68] != 2786195 { return Err("national total reference values do not match".into()); }
    let sanma = series.iter().find(|s| s["label"]=="さんま").ok_or("さんま series missing")?;
    if sanma["values"][0] != 327813 || sanma["values"][68] != 39335 { return Err("さんま reference values do not match".into()); }
    let notes: Vec<String> = (80..=107).map(|r|cell(r,2)).filter(|s|!s.is_empty()).collect();
    let document = serde_json::json!({"schemaVersion":1,"source":{"title":"海面漁業魚種別漁獲量累年統計（全国）","url":URL,"unit":"t"},"years":years,"yearLabels":labels,"notes":notes,"series":series});
    let mut bytes = serde_json::to_vec(&document)?; bytes.push(b'\n');
    fs::write(output, bytes)?;
    Ok(())
}

fn unzip_member(source: &Path, member: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let result: Output = Command::new("unzip").arg("-p").arg(source).arg(member).output()?;
    if !result.status.success() || result.stdout.is_empty() { return Err(format!("XLSX is missing {member}").into()); }
    Ok(result.stdout)
}

fn parse_shared_strings(xml: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut out=Vec::new();
    for item in blocks(xml,"si")? {
        let item = without_elements(item, "rPh")?;
        out.push(text_nodes(&item,"t")?.join(""));
    }
    Ok(out)
}

fn parse_sheet(xml: &str, strings: &[String]) -> Result<BTreeMap<usize,BTreeMap<usize,String>>,Box<dyn Error>> {
    let mut rows=BTreeMap::new();
    for (_row_attrs,row) in elements(xml,"row")? {
        let mut cells=BTreeMap::new();
        let mut row_number=None;
        for (c_attrs,c) in elements(row,"c")? {
            let reference=attr(c_attrs,"r")?;
            let (col,rown)=split_ref(reference).map_err(|e|format!("invalid cell reference {reference}: {e}"))?;
            let value=text_nodes(c,"v")?.first().cloned().unwrap_or_default();
            let rendered=if attr_opt(c_attrs,"t")==Some("s") { let idx=value.parse::<usize>().map_err(|e|format!("invalid shared string index {value:?}: {e}"))?; strings.get(idx).ok_or("invalid shared string index")?.clone() } else { value };
            if row_number.is_some_and(|number| number != rown) { return Err("cell row does not match containing row".into()); }
            row_number=Some(rown);
            cells.insert(col,rendered);
        }
        if let Some(number)=row_number { rows.insert(number,cells); }
    }
    Ok(rows)
}

fn blocks<'a>(xml:&'a str, tag:&str)->Result<Vec<&'a str>,Box<dyn Error>> {
    let open=format!("<{tag}"); let close=format!("</{tag}>"); let mut rest=xml; let mut result=Vec::new();
    while let Some(start)=find_open(rest,&open) {
        let after=&rest[start..]; let end_open=after.find('>').ok_or("malformed XML tag")?;
        if after.as_bytes().get(end_open.wrapping_sub(1))==Some(&b'/') { rest=&after[end_open+1..]; continue; }
        let content_start=end_open+1; let end=after[content_start..].find(&close).ok_or("unterminated XML tag")?+content_start;
        result.push(&after[content_start..end]); rest=&after[end+close.len()..];
    }
    Ok(result)
}
fn without_elements(xml:&str,tag:&str)->Result<String,Box<dyn Error>> {
    let open=format!("<{tag}"); let close=format!("</{tag}>"); let mut rest=xml; let mut result=String::new();
    while let Some(start)=find_open(rest,&open) {
        result.push_str(&rest[..start]);
        let after=&rest[start..]; let end_open=after.find('>').ok_or("malformed XML tag")?;
        if after.as_bytes().get(end_open.wrapping_sub(1))==Some(&b'/') { rest=&after[end_open+1..]; continue; }
        let body_start=end_open+1; let end=after[body_start..].find(&close).ok_or("unterminated XML tag")?+body_start;
        rest=&after[end+close.len()..];
    }
    result.push_str(rest);
    Ok(result)
}
fn elements<'a>(xml:&'a str,tag:&str)->Result<Vec<(&'a str,&'a str)>,Box<dyn Error>> { let open=format!("<{tag}"); let close=format!("</{tag}>"); let mut rest=xml; let mut result=Vec::new(); while let Some(start)=find_open(rest,&open) { let after=&rest[start..]; let end_open=after.find('>').ok_or("malformed XML tag")?; if after.as_bytes().get(end_open.wrapping_sub(1))==Some(&b'/') {rest=&after[end_open+1..];continue;} let body_start=end_open+1;let end=after[body_start..].find(&close).ok_or("unterminated XML tag")?+body_start;result.push((&after[open.len()..end_open],&after[body_start..end]));rest=&after[end+close.len()..];}Ok(result) }
fn find_open(xml:&str,open:&str)->Option<usize> { let mut from=0; while let Some(i)=xml[from..].find(open) { let at=from+i; let next=xml.as_bytes().get(at+open.len()).copied(); if matches!(next,Some(b'>')|Some(b'/')|Some(b' ')|Some(b'\n')|Some(b'\r')|Some(b'\t')) { return Some(at); } from=at+open.len(); } None }
fn text_nodes(xml:&str,tag:&str)->Result<Vec<String>,Box<dyn Error>> { Ok(blocks(xml,tag)?.iter().map(|v|decode_xml(v.trim())).collect()) }
fn attr<'a>(xml:&'a str,name:&str)->Result<&'a str,Box<dyn Error>> { attr_opt(xml,name).ok_or_else(||format!("missing XML attribute {name}").into()) }
fn attr_opt<'a>(xml:&'a str,name:&str)->Option<&'a str> { let key=format!("{name}=\""); let i=xml.find(&key)?+key.len(); let tail=&xml[i..]; Some(&tail[..tail.find('"')?]) }
fn decode_xml(s:&str)->String { s.replace("&amp;","&").replace("&lt;","<").replace("&gt;",">").replace("&quot;", "\"").replace("&apos;", "'") }
fn split_ref(reference:&str)->Result<(usize,usize),Box<dyn Error>> { let split=reference.find(|c:char|c.is_ascii_digit()).ok_or("invalid cell reference")?; let (letters,digits)=reference.split_at(split); let col=letters.bytes().fold(0usize,|n,b|n*26+(b-b'A'+1) as usize); let row=digits.parse::<usize>().map_err(|e|format!("bad row digits {digits:?}: {e}"))?; Ok((col,row)) }
fn column_name(mut n:usize)->String { let mut s=String::new(); while n>0 { let r=(n-1)%26; s.insert(0,(b'A'+r as u8) as char); n=(n-1)/26; } s }
fn parse_year(s:&str)->Option<i32> { let start=s.find('(')?+1; let digits=&s[start..start+4]; digits.parse().ok() }

fn commit(root: &Path, stage: &Path) -> Result<(), Box<dyn Error>> {
    for dir in ["raw_data", "transformed_data"] {
        let dest = root.join(dir);
        let backup = root.join(format!(".{dir}.previous"));
        if backup.exists() { fs::remove_dir_all(&backup)?; }
        if dest.exists() { fs::rename(&dest, &backup)?; }
        if let Err(error) = fs::rename(stage.join(dir), &dest) {
            if backup.exists() { let _ = fs::rename(&backup, &dest); }
            return Err(error.into());
        }
        if backup.exists() { fs::remove_dir_all(backup)?; }
    }
    fs::remove_dir_all(stage)?;
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    // The project already has sha2 as a dependency.
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}
