use std::{collections::HashMap, error::Error, fs, path::Path};

use super::{EXPENDITURE_TABLE_ID, INCOME_TABLE_ID, OUTPUT_DIR};
use crate::modules::estat_api::{
    get_meta_info::{
        ExplanationFlag, GetMetaInfoParamsBuilder, MandatoryParams as MetaParams, get_meta_info,
    },
    get_stats_data::{GetStatsDataParamsBuilder, MandatoryParams as DataParams, get_stats_data},
};
use serde_json::{Value, json};

const SOURCE_NOTE: &str = "本図は、e-Stat「2024年度社会保障給付費収支表」において、当該制度に計上された収入・支出の項目と金額を示す。矢印は収入項目・制度・支出項目を整理するもので、特定の収入と個別支出の対応、実際の資金移動、制度間移転の相手先を示さない。";

pub fn run(app_id: &str) -> Result<(), Box<dyn Error>> {
    let out = Path::new(OUTPUT_DIR);
    fs::create_dir_all(out.join("raw_data"))?;
    let (income, income_values) = acquire(app_id, INCOME_TABLE_ID, out)?;
    let (expenditure, expenditure_values) = acquire(app_id, EXPENDITURE_TABLE_ID, out)?;
    let (programs, income_classes) = parts(&income)?;
    let (_, expenditure_classes) = parts(&expenditure)?;
    let incomes = map_values(income_values);
    let expenditures = map_values(expenditure_values);
    let mut rows = Vec::new();
    let mut catalog_programs = Vec::new();
    let mut unknowns = Vec::new();
    let in_candidates: Vec<_> = income_classes
        .iter()
        .filter(|c| !matches!(c["@code"].as_str(), Some("18" | "20" | "41")))
        .collect();
    let ex_candidates: Vec<_> = expenditure_classes
        .iter()
        .filter(|c| !matches!(c["@code"].as_str(), Some("34" | "38" | "40" | "41")))
        .collect();
    let in_names: HashMap<String, String> = in_candidates
        .iter()
        .map(|c| (s(&c["@code"]), s(&c["@name"])))
        .collect();
    let ex_names: HashMap<String, String> = ex_candidates
        .iter()
        .map(|c| (s(&c["@code"]), s(&c["@name"])))
        .collect();
    for p in programs {
        let code = s(&p["@code"]);
        let name = s(&p["@name"]);
        let is_total = p["@level"] == "1" || name == "総計";
        let is_repeat = name.contains("再掲");
        let eligible = !is_total && !is_repeat;
        let mut income_nodes = Vec::new();
        let mut expenditure_nodes = Vec::new();
        for (kind, classes, candidates, values, dest) in [
            (
                "income",
                &income_classes,
                &in_names,
                &incomes,
                &mut income_nodes,
            ),
            (
                "expenditure",
                &expenditure_classes,
                &ex_names,
                &expenditures,
                &mut expenditure_nodes,
            ),
        ] {
            for c in classes {
                let item_code = s(&c["@code"]);
                let raw = values.get(&(code.clone(), item_code.clone())).cloned();
                let item_name = s(&c["@name"]);
                let (status, display, reason) = classify(raw.as_deref(), &item_name);
                let label = if kind == "expenditure" {
                    item_name.replace('_', "／")
                } else {
                    item_name.replace('_', "／")
                };
                let visible = display == "表示" && eligible && candidates.contains_key(&item_code);
                if display == "保留" && eligible {
                    unknowns.push(json!({"program_code":code,"program":name,"category":kind,"item_code":item_code,"item":item_name,"raw_value":raw,"status":status,"reason":reason,"table_id":if kind=="income" {INCOME_TABLE_ID} else {EXPENDITURE_TABLE_ID}}));
                }
                if visible {
                    let group = diagram_group(kind, &item_code)
                        .ok_or_else(|| format!("no diagram group for {kind} item {item_code}"))?;
                    let prefix = if kind == "income" { "I" } else { "E" };
                    let value = raw.as_deref().and_then(|v| v.parse::<f64>().ok());
                    dest.push(json!({
                        "id": format!("{prefix}{item_code}"),
                        "label": label,
                        "group": group,
                        "value": value,
                    }));
                }
                let decision = if !eligible {
                    "対象外".to_owned()
                } else if !candidates.contains_key(&item_code) {
                    "非表示（集計項目）".to_owned()
                } else {
                    display
                };
                rows.push(json!({"program_code":code,"program":name,"item_code":item_code,"item":item_name,"category":kind,"raw_value":raw,"status":status,"display_decision":decision,"reason":reason,"table_id":if kind=="income" {INCOME_TABLE_ID} else {EXPENDITURE_TABLE_ID},"table_url":format!("https://www.e-stat.go.jp/dbview?sid={}",if kind=="income" {INCOME_TABLE_ID} else {EXPENDITURE_TABLE_ID})}));
            }
        }
        if !eligible {
            continue;
        }
        let short = name.split('_').skip(1).collect::<Vec<_>>().join("／");
        let display_name = if short.is_empty() { &name } else { &short };
        let id = format!("program-{code}");
        catalog_programs.push(json!({
            "id": id,
            "name": display_name,
            "income": income_nodes,
            "expenditure": expenditure_nodes,
            "note": SOURCE_NOTE,
        }));
    }
    let master: Vec<_> = programs_all(&income).into_iter().map(|p| {let name=s(&p["@name"]);let repeat=name.contains("再掲");let total=p["@level"]=="1"||name=="総計";json!({"id":format!("program-{}",s(&p["@code"])),"estat_code":p["@code"],"estat_name":name,"major_category":name.split('_').next().unwrap_or(""),"kind":if total {"総計"} else if repeat {"再掲"} else if name=="他の社会保障制度" {"集合区分"} else {"通常区分"},"in_scope":!total&&!repeat,"parent_code":p.get("@parentCode")})}).collect();
    fs::write(
        out.join("programs.json"),
        serde_json::to_vec_pretty(&master)?,
    )?;
    fs::write(
        out.join("program_item_records.json"),
        serde_json::to_vec_pretty(&rows)?,
    )?;
    fs::write(out.join("evidence.json"), serde_json::to_vec_pretty(&rows)?)?;
    fs::write(
        out.join("unknowns.json"),
        serde_json::to_vec_pretty(&unknowns)?,
    )?;
    let item_master: Vec<_> = [
        ("income", &income_classes, &in_names),
        ("expenditure", &expenditure_classes, &ex_names),
    ]
    .into_iter()
    .flat_map(|(kind, classes, candidates)| {
        classes.iter().map(move |item| {
            let code = s(&item["@code"]);
            json!({"id":format!("{kind}-{code}"),"estat_code":code,"estat_name":item["@name"],"category":kind,"item_type":item_type(kind, &code),"parent_code":item.get("@parentCode"),"display_candidate":candidates.contains_key(&code)})
        })
    })
    .collect();
    fs::write(
        out.join("item_master.json"),
        serde_json::to_vec_pretty(&item_master)?,
    )?;
    fs::write(
        out.join("generation_manifest.json"),
        serde_json::to_vec_pretty(
            &json!({"generated_at_unix":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),"year":2024,"source_table_ids":[INCOME_TABLE_ID,EXPENDITURE_TABLE_ID],"acquisition_method":"e-Stat getMetaInfo + getStatsData through existing estat_api; single page limit 5000","income_record_count":incomes.len(),"income_reported_total":385,"expenditure_record_count":expenditures.len(),"expenditure_reported_total":735,"record_count":rows.len(),"program_count":master.len(),"diagram_count":catalog_programs.len(),"diagram_representation":"frontend-rendered Mermaid from categorized node data","rule_version":"social-security-budget-flow-v3","special_symbol_policy":"―/－は計数なしとして非表示。その他の未確認記号は保留","negative_value_policy":"負値は保留。ただし収支差の負値は収入－支出合計の差額として非表示","aggregates_retained_in_records":true}),
        )?,
    )?;
    let catalog = serde_json::to_vec_pretty(&json!({"programs":catalog_programs}))?;
    fs::write(out.join("catalog.json"), &catalog)?;
    Ok(())
}

fn acquire(app_id: &str, id: &str, out: &Path) -> Result<(Value, Vec<Value>), Box<dyn Error>> {
    let meta = get_meta_info(
        GetMetaInfoParamsBuilder::new(MetaParams {
            app_id,
            stats_data_id: id,
        })
        .lang("J")
        .explanation_get_flg(ExplanationFlag::Include)
        .build(),
    )?;
    let data = get_stats_data(
        GetStatsDataParamsBuilder::new(DataParams { app_id })
            .stats_data_id(id)
            .start_position(1)
            .limit(5000)
            .meta_get_flg("N")
            .explanation_get_flg(ExplanationFlag::Include)
            .build(),
    )?;
    let mut m: Value = serde_json::from_str(&meta)?;
    let mut d: Value = serde_json::from_str(&data)?;
    redact(&mut m);
    redact(&mut d);
    let stat = &d["GET_STATS_DATA"]["STATISTICAL_DATA"];
    let total = stat["RESULT_INF"]["TOTAL_NUMBER"].as_u64().unwrap_or(0);
    let values = as_vec(&stat["DATA_INF"]["VALUE"]);
    if values.len() as u64 != total {
        return Err(format!("table {id}: received {} of {total} values", values.len()).into());
    }
    fs::write(
        out.join("raw_data").join(format!("{id}_metadata.json")),
        serde_json::to_vec_pretty(&m)?,
    )?;
    fs::write(
        out.join("raw_data").join(format!("{id}_values.json")),
        serde_json::to_vec_pretty(&d)?,
    )?;
    Ok((m, values))
}
fn parts(meta: &Value) -> Result<(Vec<Value>, Vec<Value>), Box<dyn Error>> {
    let classes = as_vec(&meta["GET_META_INFO"]["METADATA_INF"]["CLASS_INF"]["CLASS_OBJ"]);
    let p = classes
        .iter()
        .find(|x| x["@id"] == "cat01")
        .ok_or("missing cat01")?;
    let items = classes
        .iter()
        .find(|x| x["@id"] == "cat02")
        .ok_or("missing cat02")?;
    let values = as_vec(&items["CLASS"]);
    Ok((as_vec(&p["CLASS"]), values))
}
fn programs_all(meta: &Value) -> Vec<Value> {
    let classes = as_vec(&meta["GET_META_INFO"]["METADATA_INF"]["CLASS_INF"]["CLASS_OBJ"]);
    classes
        .iter()
        .find(|x| x["@id"] == "cat01")
        .map(|x| as_vec(&x["CLASS"]))
        .unwrap_or_default()
}
fn map_values(values: Vec<Value>) -> HashMap<(String, String), String> {
    values
        .into_iter()
        .map(|v| ((s(&v["@cat01"]), s(&v["@cat02"])), s(&v["$"])))
        .collect()
}
fn classify(raw: Option<&str>, item_name: &str) -> (String, String, String) {
    match raw {
        None => ("未確認".into(), "保留".into(), "取得値がありません".into()),
        Some(v) if v == "0" => ("計上なし".into(), "非表示".into(), "数値ゼロ".into()),
        Some("―" | "－") => ("計上なし".into(), "非表示".into(), "計数なし（e-Stat表章記号）".into()),
        Some(v) if v.parse::<f64>().is_ok_and(|n| n > 0.0) => {
            ("計上あり".into(), "表示".into(), "正の計上値".into())
        }
        Some(v) if item_name == "収支差" && v.parse::<f64>().is_ok_and(|n| n < 0.0) => (
            "負値・調整値".into(),
            "非表示".into(),
            "収支差（収入－支出合計の差額）の負値".into(),
        ),
        Some(v) if v.parse::<f64>().is_ok_and(|n| n < 0.0) => (
            "負値・調整値".into(),
            "保留".into(),
            "負値の解釈を確認".into(),
        ),
        Some(v) => (
            "未確認".into(),
            "保留".into(),
            format!("未確認の特殊記号: {v}"),
        ),
    }
}
fn item_type(kind: &str, code: &str) -> &'static str {
    match (kind, code.parse::<u32>().unwrap_or_default()) {
        ("income", 19) => "制度間移転収入",
        ("income", 18 | 20 | 41) => "集計",
        ("income", _) => "財源・収入",
        ("expenditure", 21..=33) => "給付",
        ("expenditure", 35..=37) => "給付以外の支出",
        ("expenditure", 39) => "制度間移転支出",
        _ => "集計",
    }
}
fn diagram_group(kind: &str, code: &str) -> Option<&'static str> {
    let code = code.parse::<u32>().ok()?;
    match (kind, code) {
        ("income", 19) => Some("transfer"),
        ("income", _) => Some("income"),
        ("expenditure", 21..=33) => Some("benefit"),
        ("expenditure", 35..=37) => Some("other"),
        ("expenditure", 39) => Some("transfer"),
        _ => None,
    }
}
fn s(v: &Value) -> String {
    v.as_str().unwrap_or("").to_owned()
}
fn as_vec(v: &Value) -> Vec<Value> {
    match v {
        Value::Array(a) => a.clone(),
        Value::Object(_) => vec![v.clone()],
        _ => vec![],
    }
}
fn redact(v: &mut Value) {
    match v {
        Value::Object(m) => {
            for (k, v) in m.iter_mut() {
                if k.eq_ignore_ascii_case("appid") {
                    *v = Value::String("REDACTED".into())
                } else {
                    redact(v)
                }
            }
        }
        Value::Array(a) => {
            for v in a {
                redact(v)
            }
        }
        _ => {}
    }
}
