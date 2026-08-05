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

pub struct MandatoryParams<'a> {
    pub app_id: &'a str,
}

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

pub struct GetStatsListParamsBuilder<'a> {
    app_id: &'a str,
    lang: Option<&'a str>,
    survey_years: Option<&'a str>,
    open_years: Option<&'a str>,
    stats_field: Option<&'a str>,
    stats_code: Option<&'a str>,
    search_word: Option<&'a str>,
    search_kind: Option<SearchKind>,
    collect_area: Option<CollectArea>,
    explanation_get_flg: Option<ExplanationFlag>,
    stats_name_list: Option<StatsNameListFlag>,
    start_position: Option<u32>,
    limit: Option<u32>,
    updated_date: Option<&'a str>,
    callback: Option<&'a str>,
}

impl<'a> GetStatsListParamsBuilder<'a> {
    pub fn new(params: MandatoryParams<'a>) -> Self {
        Self {
            app_id: params.app_id,
            lang: None,
            survey_years: None,
            open_years: None,
            stats_field: None,
            stats_code: None,
            search_word: None,
            search_kind: None,
            collect_area: None,
            explanation_get_flg: None,
            stats_name_list: None,
            start_position: None,
            limit: None,
            updated_date: None,
            callback: None,
        }
    }

    pub fn lang(mut self, v: &'a str) -> Self { self.lang = Some(v); self }
    pub fn survey_years(mut self, v: &'a str) -> Self { self.survey_years = Some(v); self }
    pub fn open_years(mut self, v: &'a str) -> Self { self.open_years = Some(v); self }
    pub fn stats_field(mut self, v: &'a str) -> Self { self.stats_field = Some(v); self }
    pub fn stats_code(mut self, v: &'a str) -> Self { self.stats_code = Some(v); self }
    pub fn search_word(mut self, v: &'a str) -> Self { self.search_word = Some(v); self }
    pub fn search_kind(mut self, v: SearchKind) -> Self { self.search_kind = Some(v); self }
    pub fn collect_area(mut self, v: CollectArea) -> Self { self.collect_area = Some(v); self }
    pub fn explanation_get_flg(mut self, v: ExplanationFlag) -> Self { self.explanation_get_flg = Some(v); self }
    pub fn stats_name_list(mut self, v: StatsNameListFlag) -> Self { self.stats_name_list = Some(v); self }
    pub fn start_position(mut self, v: u32) -> Self { self.start_position = Some(v); self }
    pub fn limit(mut self, v: u32) -> Self { self.limit = Some(v); self }
    pub fn updated_date(mut self, v: &'a str) -> Self { self.updated_date = Some(v); self }
    pub fn callback(mut self, v: &'a str) -> Self { self.callback = Some(v); self }

    pub fn build(self) -> GetStatsListParams<'a> {
        GetStatsListParams {
            app_id: self.app_id,
            lang: self.lang,
            survey_years: self.survey_years,
            open_years: self.open_years,
            stats_field: self.stats_field,
            stats_code: self.stats_code,
            search_word: self.search_word,
            search_kind: self.search_kind,
            collect_area: self.collect_area,
            explanation_get_flg: self.explanation_get_flg,
            stats_name_list: self.stats_name_list,
            start_position: self.start_position,
            limit: self.limit,
            updated_date: self.updated_date,
            callback: self.callback,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mandatory_params_only() {
        let params = GetStatsListParamsBuilder::new(MandatoryParams { app_id: "test_app" }).build();
        let query_params = construct_query_params(&params);

        assert_eq!(query_params.len(), 1);
        assert_eq!(query_params.get("appId"), Some(&"test_app".to_string()));
    }

    #[test]
    fn test_enum_params_as_values() {
        let params = GetStatsListParamsBuilder::new(MandatoryParams { app_id: "test_app" })
            .search_kind(SearchKind::Statistics)
            .collect_area(CollectArea::Prefecture)
            .explanation_get_flg(ExplanationFlag::Exclude)
            .stats_name_list(StatsNameListFlag::Include)
            .build();
        let query_params = construct_query_params(&params);

        assert_eq!(query_params.len(), 5);
        assert_eq!(query_params.get("searchKind"), Some(&"1".to_string()));
        assert_eq!(query_params.get("collectArea"), Some(&"2".to_string()));
        assert_eq!(query_params.get("explanationGetFlg"), Some(&"N".to_string()));
        assert_eq!(query_params.get("statsNameList"), Some(&"Y".to_string()));
    }

    #[test]
    fn test_optional_str_params() {
        let params = GetStatsListParamsBuilder::new(MandatoryParams { app_id: "test_app" })
            .lang("E")
            .search_word("population")
            .build();
        let query_params = construct_query_params(&params);

        assert_eq!(query_params.len(), 3);
        assert_eq!(query_params.get("lang"), Some(&"E".to_string()));
        assert_eq!(query_params.get("searchWord"), Some(&"population".to_string()));
    }
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
