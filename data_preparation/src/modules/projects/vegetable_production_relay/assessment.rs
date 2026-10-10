//! M2-B: reproducible, offline-only catalogue assessment. No table is approved here.
use super::discovery::{self, Catalogue, TableCandidate};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
    fs,
    path::Path,
};

const COMMODITY: &str = "だいこん";
const OUTPUT_JSON: &str = "daikon_assessment.json";
const OUTPUT_MD: &str = "daikon_assessment.md";

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum TitleKind {
    ExplicitMonth,
    AnnualTotal,
    CombinedYear,
    Historical,
    Unclassified,
}

#[derive(Debug, Serialize)]
pub struct AssessmentRow {
    pub id: String,
    pub title: String,
    pub statistics_name: Option<String>,
    pub survey_date: Option<String>,
    pub open_date: Option<String>,
    pub updated_date: Option<String>,
    pub table_sub_category1: Option<String>,
    pub year_hint: Option<u16>,
    pub kind: TitleKind,
    pub month_hint: Option<u8>,
    pub evidence: Vec<String>,
    pub verification_required: Vec<String>,
    pub decision: &'static str,
}

#[derive(Debug, Serialize)]
pub struct YearGroup {
    pub year_hint: Option<u16>,
    pub candidate_count: usize,
    pub explicit_months: Vec<u8>,
    pub repeated_months: Vec<u8>,
    pub missing_explicit_months: Vec<u8>,
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub stats_code: String,
    pub commodity: &'static str,
    pub catalogue_total: usize,
    pub catalogue_pages: usize,
    pub catalogue_sha256: String,
    pub candidate_count: usize,
    pub selection_status: &'static str,
    pub groups: Vec<YearGroup>,
    pub candidates: Vec<AssessmentRow>,
}

fn fail(message: &str) -> Box<dyn Error> {
    message.to_owned().into()
}
fn year_hint(s: Option<&str>) -> Option<u16> {
    let s = s?;
    let (from, to) = s.split_once('-')?;
    if from.len() != 6
        || to.len() != 6
        || &from[0..4] != &to[0..4]
        || &from[4..] != "01"
        || &to[4..] != "12"
    {
        return None;
    }
    from[0..4].parse().ok()
}
fn fullwidth_ascii(s: &str) -> String {
    s.chars()
        .map(|c| {
            if ('０'..='９').contains(&c) {
                char::from_u32(c as u32 - '０' as u32 + '0' as u32).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}
// Exact trailing labels only: avoid finding arbitrary months elsewhere in long titles.
fn classify(title: &str, stats: Option<&str>) -> (TitleKind, Option<u8>, String) {
    if stats.is_some_and(|s| s.contains("長期累年")) || title.contains("累年統計") {
        return (
            TitleKind::Historical,
            None,
            "historical/long-term cumulative label".into(),
        );
    }
    let suffix = title.rsplit_once(COMMODITY).map(|(_, tail)| tail.trim());
    let Some(suffix) = suffix else {
        return (
            TitleKind::Unclassified,
            None,
            "commodity token absent".into(),
        );
    };
    let ascii = fullwidth_ascii(suffix);
    let bare = ascii
        .strip_suffix('月')
        .and_then(|s| s.trim().parse::<u8>().ok());
    if let Some(month) = bare.filter(|m| (1..=12).contains(m)) {
        return (
            TitleKind::ExplicitMonth,
            Some(month),
            "exact trailing commodity + month label".into(),
        );
    }
    if suffix == "計" {
        return (
            TitleKind::AnnualTotal,
            None,
            "exact trailing commodity + 計 label".into(),
        );
    }
    if suffix.contains("1月～12月")
        || suffix.contains("１月～１２月")
        || ascii.contains("1月～12月")
    {
        return (
            TitleKind::CombinedYear,
            None,
            "title explicitly mentions January–December; internal time dimension unknown".into(),
        );
    }
    (
        TitleKind::Unclassified,
        None,
        "no supported temporal title pattern".into(),
    )
}

pub fn assess(catalogue: &Catalogue, bytes: &[u8]) -> Result<Report, Box<dyn Error>> {
    if catalogue.stats_code != "00500226" || catalogue.declared_total != catalogue.candidates.len()
    {
        return Err(fail("unexpected or incomplete catalogue"));
    }
    let mut ids = HashSet::new();
    if catalogue
        .candidates
        .iter()
        .any(|c| c.id.is_empty() || !ids.insert(c.id.as_str()))
    {
        return Err(fail("duplicate or empty table ID"));
    }
    let mut candidates: Vec<AssessmentRow> = catalogue
        .candidates
        .iter()
        .filter(|c| c.title.contains(COMMODITY))
        .map(|c: &TableCandidate| {
            let (kind, month_hint, label) = classify(&c.title, c.statistics_name.as_deref());
            AssessmentRow {
                id: c.id.clone(),
                title: c.title.clone(),
                statistics_name: c.statistics_name.clone(),
                survey_date: c.survey_date.clone(),
                open_date: c.open_date.clone(),
                updated_date: c.updated_date.clone(),
                table_sub_category1: c.table_sub_category1.clone(),
                year_hint: year_hint(c.survey_date.as_deref()),
                kind,
                month_hint,
                evidence: vec![
                    format!("Title: {}", c.title),
                    format!("Classification basis: {label}"),
                    format!(
                        "Survey date (not observation coverage): {}",
                        c.survey_date.as_deref().unwrap_or("absent")
                    ),
                ],
                verification_required: vec![
                    "getMetaInfo: geographic and origin classification codes".into(),
                    "getMetaInfo: time dimension and observation coverage".into(),
                    "getMetaInfo: measurement concepts and units".into(),
                    "revision compatibility and duplicate observations".into(),
                ],
                decision: "pending_metadata_verification",
            }
        })
        .collect();
    candidates.sort_by(|a, b| (a.year_hint, &a.id).cmp(&(b.year_hint, &b.id)));
    let mut groups: BTreeMap<Option<u16>, Vec<&AssessmentRow>> = BTreeMap::new();
    for row in &candidates {
        groups.entry(row.year_hint).or_default().push(row);
    }
    let groups = groups
        .into_iter()
        .map(|(year, rows)| {
            let mut months = Vec::new();
            let mut repeated = Vec::new();
            for row in &rows {
                if let Some(m) = row.month_hint {
                    if months.contains(&m) {
                        if !repeated.contains(&m) {
                            repeated.push(m);
                        }
                    } else {
                        months.push(m);
                    }
                }
            }
            months.sort();
            repeated.sort();
            let missing = (1..=12).filter(|m| !months.contains(m)).collect();
            YearGroup {
                year_hint: year,
                candidate_count: rows.len(),
                explicit_months: months,
                repeated_months: repeated,
                missing_explicit_months: missing,
                ids: rows.iter().map(|r| r.id.clone()).collect(),
            }
        })
        .collect();
    Ok(Report {
        schema_version: 1,
        stats_code: catalogue.stats_code.clone(),
        commodity: COMMODITY,
        catalogue_total: catalogue.declared_total,
        catalogue_pages: catalogue.pages,
        catalogue_sha256: format!("{:x}", Sha256::digest(bytes)),
        candidate_count: candidates.len(),
        selection_status: "no_tables_selected_metadata_required",
        groups,
        candidates,
    })
}

fn markdown(report: &Report) -> String {
    let mut s = format!(
        "# Daikon catalogue assessment (M2-B)\n\nSource: complete replay-verified catalogue; SHA-256 `{}`.\n\n**No tables selected.** All {} candidates require metadata verification. `SURVEY_DATE` is only a year hint, never proof of observation coverage.\n\n| Survey-year hint | Candidates | Explicit months | Missing explicit month labels | Repeated months |\n|---|---:|---|---|---|\n",
        report.catalogue_sha256, report.candidate_count
    );
    for g in &report.groups {
        s.push_str(&format!(
            "| {} | {} | {:?} | {:?} | {:?} |\n",
            g.year_hint
                .map(|y| y.to_string())
                .unwrap_or("unknown".into()),
            g.candidate_count,
            g.explicit_months,
            g.missing_explicit_months,
            g.repeated_months
        ));
    }
    s.push_str("\n## Candidates (all pending getMetaInfo)\n\n| Table ID | Year hint | Title pattern | Month hint | Title |\n|---|---|---|---|---|\n");
    for c in &report.candidates {
        s.push_str(&format!(
            "| `{}` | {} | {:?} | {} | {} |\n",
            c.id,
            c.year_hint.map(|v| v.to_string()).unwrap_or("?".into()),
            c.kind,
            c.month_hint.map(|v| v.to_string()).unwrap_or("—".into()),
            c.title.replace('|', "\\|")
        ));
    }
    s.push_str("\n## Interpretation\n\nExplicit monthly labels describe catalogue titles, not verified monthly observations. A missing monthly title does not prove missing months within a table. Do not aggregate, choose a revision, or publish until metadata and observations are independently checked.\n");
    s
}

pub fn run(dir: &Path) -> Result<(), Box<dyn Error>> {
    // Reconstruct from raw page captures, validating checksums and pagination first.
    let verified = discovery::replay(dir)?;
    let catalogue_bytes = fs::read(dir.join("catalogue.json"))?;
    let cached: Catalogue = serde_json::from_slice(&catalogue_bytes)?;
    if serde_json::to_value(&verified)? != serde_json::to_value(&cached)? {
        return Err(fail("catalogue.json differs from verified page evidence"));
    }
    let report = assess(&verified, &catalogue_bytes)?;
    // Never overwrite an existing assessment; failures cannot silently replace reviewed findings.
    if dir.join(OUTPUT_JSON).exists() || dir.join(OUTPUT_MD).exists() {
        return Err(fail(
            "assessment outputs already exist; use a new capture or remove generated outputs explicitly",
        ));
    }
    let json_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join(OUTPUT_JSON))?;
    serde_json::to_writer_pretty(json_file, &report)?;
    let mut md_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join(OUTPUT_MD))?;
    use std::io::Write;
    md_file.write_all(markdown(&report).as_bytes())?;
    println!(
        "Assessed {} daikon candidates; none selected. Reports saved in {}",
        report.candidate_count,
        dir.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample(id: &str, title: &str, date: &str) -> TableCandidate {
        TableCandidate {
            id: id.into(),
            title: title.into(),
            survey_date: Some(date.into()),
            statistics_name: Some("青果物卸売市場調査 確報".into()),
            open_date: None,
            updated_date: None,
            table_sub_category1: None,
        }
    }
    #[test]
    fn title_classification_handles_fullwidth_and_never_approves() {
        let c = Catalogue {
            stats_code: "00500226".into(),
            declared_total: 3,
            pages: 1,
            candidates: vec![
                sample("1", "野菜 だいこん １月", "201801-201812"),
                sample("2", "野菜 だいこん 計_1月～12月", "202101-202112"),
                sample("3", "野菜 にんじん ２月", "201801-201812"),
            ],
        };
        let r = assess(&c, b"fixture").unwrap();
        assert_eq!(r.candidate_count, 2);
        assert_eq!(r.candidates[0].kind, TitleKind::ExplicitMonth);
        assert_eq!(r.candidates[0].month_hint, Some(1));
        assert_eq!(r.candidates[1].kind, TitleKind::CombinedYear);
        assert!(
            r.candidates
                .iter()
                .all(|c| c.decision == "pending_metadata_verification")
        );
    }
    #[test]
    fn year_and_duplicate_checks() {
        assert_eq!(year_hint(Some("201801-201812")), Some(2018));
        assert_eq!(year_hint(Some("201801-201901")), None);
        let one = sample("x", "だいこん 12月", "201801-201812");
        let c = Catalogue {
            stats_code: "00500226".into(),
            declared_total: 2,
            pages: 1,
            candidates: vec![sample("x", "だいこん 12月", "201801-201812"), one],
        };
        assert!(assess(&c, b"fixture").is_err());
    }
    #[test]
    fn no_month_from_broad_title() {
        assert_eq!(classify("だいこん 計", None).0, TitleKind::AnnualTotal);
        assert_eq!(classify("だいこん", None).0, TitleKind::Unclassified);
        assert_eq!(classify("だいこん 13月", None).0, TitleKind::Unclassified);
        assert_eq!(
            classify("だいこん(昭和60年～平成30年)", None).0,
            TitleKind::Unclassified
        );
    }
}
