use std::collections::HashMap;
use std::error::Error;

use super::{
    decoder::decode_escaped_json_string,
    get::{GetRequestBuilder, get},
};

#[allow(dead_code)]
pub enum ExplanationFlag {
    Include,
    Exclude,
}

impl ExplanationFlag {
    pub fn as_str(&self) -> &str {
        match self {
            ExplanationFlag::Include => "Y",
            ExplanationFlag::Exclude => "N",
        }
    }
}

#[allow(dead_code)]
pub enum StatsNameListFlag {
    Include,
    Exclude,
}

#[allow(dead_code)]
impl StatsNameListFlag {
    pub fn as_str(&self) -> &str {
        match self {
            StatsNameListFlag::Include => "Y",
            StatsNameListFlag::Exclude => "N",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum SearchKind {
    Statistics = 1,
    RegionMesh = 2,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum CollectArea {
    National = 1,
    Prefecture = 2,
    Municipality = 3,
}

#[derive(Default)]
pub struct GetStatsListParams<'a> {
    pub app_id: &'a str,
    pub lang: Option<&'a str>,                        // "J" or "E"
    pub survey_years: Option<&'a str>,                // "yyyy", "yyyymm", "yyyymm-yyyyymm"
    pub open_years: Option<&'a str>,                  // "yyyy", "yyyymm", "yyyymm-yyyyymm"
    pub stats_field: Option<&'a str>,                 // "2-digit or 4-digit number"
    pub stats_code: Option<&'a str>,                  // "5-digit or 8-digit number"
    pub search_word: Option<&'a str>,                 // Search keywords, allows AND/OR/NOT
    pub search_kind: Option<SearchKind>,              // 1: Statistics, 2: Region Mesh
    pub collect_area: Option<CollectArea>,            // 1: National, 2: Prefecture, 3: Municipality
    pub explanation_get_flg: Option<ExplanationFlag>, // "Y" or "N"
    pub stats_name_list: Option<StatsNameListFlag>,   // "Y" to get stat names list
    pub start_position: Option<u32>,                  // Starting position (for pagination)
    pub limit: Option<u32>,                           // Number of records to fetch
    pub updated_date: Option<&'a str>, // "yyyy", "yyyymmdd", or range "yyyymmdd-yyyyymmdd"
    pub callback: Option<&'a str>,     // JSONP callback function
}

fn construct_query_params<'a>(params: &GetStatsListParams<'a>) -> HashMap<String, String> {
    let mut query_params = HashMap::new();

    // Mandatory parameters
    query_params.insert("appId".to_string(), params.app_id.to_string());

    // Optional parameters (add only if Some)
    if let Some(lang) = params.lang {
        query_params.insert("lang".to_string(), lang.to_string());
    }
    if let Some(survey_years) = params.survey_years {
        query_params.insert("surveyYears".to_string(), survey_years.to_string());
    }
    if let Some(open_years) = params.open_years {
        query_params.insert("openYears".to_string(), open_years.to_string());
    }
    if let Some(stats_field) = params.stats_field {
        query_params.insert("statsField".to_string(), stats_field.to_string());
    }
    if let Some(stats_code) = params.stats_code {
        query_params.insert("statsCode".to_string(), stats_code.to_string());
    }
    if let Some(search_word) = params.search_word {
        query_params.insert("searchWord".to_string(), search_word.to_string());
    }
    if let Some(search_kind) = &params.search_kind {
        query_params.insert("searchKind".to_string(), (*search_kind as i32).to_string());
    }
    if let Some(collect_area) = &params.collect_area {
        query_params.insert(
            "collectArea".to_string(),
            (*collect_area as i32).to_string(),
        );
    }
    if let Some(explanation_get_flg) = &params.explanation_get_flg {
        query_params.insert(
            "explanationGetFlg".to_string(),
            explanation_get_flg.as_str().to_string(),
        );
    }
    if let Some(stats_name_list) = &params.stats_name_list {
        query_params.insert(
            "statsNameList".to_string(),
            stats_name_list.as_str().to_string(),
        );
    }
    if let Some(start_position) = params.start_position {
        query_params.insert("startPosition".to_string(), start_position.to_string());
    }
    if let Some(limit) = params.limit {
        query_params.insert("limit".to_string(), limit.to_string());
    }
    if let Some(updated_date) = params.updated_date {
        query_params.insert("updatedDate".to_string(), updated_date.to_string());
    }
    if let Some(callback) = params.callback {
        query_params.insert("callback".to_string(), callback.to_string());
    }

    query_params
}

#[allow(dead_code)]
pub fn get_stats_list(params: GetStatsListParams) -> Result<String, Box<dyn Error>> {
    let query_params = construct_query_params(&params);

    let url = "https://api.e-stat.go.jp/rest/3.0/app/json/getStatsList";
    let request = GetRequestBuilder {
        url,
        params: Some(&query_params),
    };

    let decoded_json_string = decode_escaped_json_string(&get(&request)?)?;

    Ok(decoded_json_string)
}
