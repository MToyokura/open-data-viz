use std::collections::HashMap;
use std::error::Error;

use super::{
    decoder::decode_escaped_json_string,
    get::{GetRequestBuilder, get},
    get_meta_info::ExplanationFlag,
};

#[allow(dead_code)]
pub enum MetaGetFlg {
    Include,
    Exclude,
}

#[allow(dead_code)]
impl MetaGetFlg {
    pub fn as_str(&self) -> &str {
        match self {
            MetaGetFlg::Include => "Y",
            MetaGetFlg::Exclude => "N",
        }
    }
}

#[derive(Default)]
pub struct GetStatsDataParams<'a> {
    pub app_id: &'a str,
    pub data_set_id: Option<&'a str>,
    pub stats_data_id: Option<&'a str>,
    pub lv_tab: Option<&'a str>,
    pub cd_tab: Option<&'a str>,
    pub cd_tab_from: Option<&'a str>,
    pub cd_tab_to: Option<&'a str>,
    pub lv_time: Option<&'a str>,
    pub cd_time: Option<&'a str>,
    pub cd_time_from: Option<&'a str>,
    pub cd_time_to: Option<&'a str>,
    pub lv_area: Option<&'a str>,
    pub cd_area: Option<&'a str>,
    pub cd_area_from: Option<&'a str>,
    pub cd_area_to: Option<&'a str>,
    pub lv_cat01: Option<&'a str>,
    pub cd_cat01: Option<&'a str>,
    pub cd_cat01_from: Option<&'a str>,
    pub cd_cat01_to: Option<&'a str>,
    pub lv_cat02: Option<&'a str>,
    pub cd_cat02: Option<&'a str>,
    pub cd_cat02_from: Option<&'a str>,
    pub cd_cat02_to: Option<&'a str>,
    pub lv_cat03: Option<&'a str>,
    pub cd_cat03: Option<&'a str>,
    pub cd_cat03_from: Option<&'a str>,
    pub cd_cat03_to: Option<&'a str>,
    pub lv_cat04: Option<&'a str>,
    pub cd_cat04: Option<&'a str>,
    pub cd_cat04_from: Option<&'a str>,
    pub cd_cat04_to: Option<&'a str>,
    pub lv_cat05: Option<&'a str>,
    pub cd_cat05: Option<&'a str>,
    pub cd_cat05_from: Option<&'a str>,
    pub cd_cat05_to: Option<&'a str>,
    pub lv_cat06: Option<&'a str>,
    pub cd_cat06: Option<&'a str>,
    pub cd_cat06_from: Option<&'a str>,
    pub cd_cat06_to: Option<&'a str>,
    pub lv_cat07: Option<&'a str>,
    pub cd_cat07: Option<&'a str>,
    pub cd_cat07_from: Option<&'a str>,
    pub cd_cat07_to: Option<&'a str>,
    pub lv_cat08: Option<&'a str>,
    pub cd_cat08: Option<&'a str>,
    pub cd_cat08_from: Option<&'a str>,
    pub cd_cat08_to: Option<&'a str>,
    pub lv_cat09: Option<&'a str>,
    pub cd_cat09: Option<&'a str>,
    pub cd_cat09_from: Option<&'a str>,
    pub cd_cat09_to: Option<&'a str>,
    pub lv_cat10: Option<&'a str>,
    pub cd_cat10: Option<&'a str>,
    pub cd_cat10_from: Option<&'a str>,
    pub cd_cat10_to: Option<&'a str>,
    pub lv_cat11: Option<&'a str>,
    pub cd_cat11: Option<&'a str>,
    pub cd_cat11_from: Option<&'a str>,
    pub cd_cat11_to: Option<&'a str>,
    pub lv_cat12: Option<&'a str>,
    pub cd_cat12: Option<&'a str>,
    pub cd_cat12_from: Option<&'a str>,
    pub cd_cat12_to: Option<&'a str>,
    pub lv_cat13: Option<&'a str>,
    pub cd_cat13: Option<&'a str>,
    pub cd_cat13_from: Option<&'a str>,
    pub cd_cat13_to: Option<&'a str>,
    pub lv_cat14: Option<&'a str>,
    pub cd_cat14: Option<&'a str>,
    pub cd_cat14_from: Option<&'a str>,
    pub cd_cat14_to: Option<&'a str>,
    pub lv_cat15: Option<&'a str>,
    pub cd_cat15: Option<&'a str>,
    pub cd_cat15_from: Option<&'a str>,
    pub cd_cat15_to: Option<&'a str>,
    pub start_position: Option<u32>,
    pub limit: Option<u32>,
    pub meta_get_flg: Option<&'a str>,
    pub cnt_get_flg: Option<&'a str>,
    pub explanation_get_flg: Option<ExplanationFlag>,
    pub annotation_get_flg: Option<&'a str>,
    pub replace_sp_char: Option<u32>,
    pub callback: Option<&'a str>,
    pub section_header_flg: Option<u32>,
}

fn construct_query_params_stats_data<'a>(
    params: &GetStatsDataParams<'a>,
) -> HashMap<String, String> {
    let mut query_params = HashMap::new();

    // Mandatory parameters
    query_params.insert("appId".to_string(), params.app_id.to_string());

    // Optional parameters
    if let Some(data_set_id) = params.data_set_id {
        query_params.insert("dataSetId".to_string(), data_set_id.to_string());
    }
    if let Some(stats_data_id) = params.stats_data_id {
        query_params.insert("statsDataId".to_string(), stats_data_id.to_string());
    }
    if let Some(lv_tab) = params.lv_tab {
        query_params.insert("lvTab".to_string(), lv_tab.to_string());
    }
    if let Some(cd_tab) = params.cd_tab {
        query_params.insert("cdTab".to_string(), cd_tab.to_string());
    }
    if let Some(cd_tab_from) = params.cd_tab_from {
        query_params.insert("cdTabFrom".to_string(), cd_tab_from.to_string());
    }
    if let Some(cd_tab_to) = params.cd_tab_to {
        query_params.insert("cdTabTo".to_string(), cd_tab_to.to_string());
    }
    if let Some(lv_time) = params.lv_time {
        query_params.insert("lvTime".to_string(), lv_time.to_string());
    }
    if let Some(cd_time) = params.cd_time {
        query_params.insert("cdTime".to_string(), cd_time.to_string());
    }
    if let Some(cd_time_from) = params.cd_time_from {
        query_params.insert("cdTimeFrom".to_string(), cd_time_from.to_string());
    }
    if let Some(cd_time_to) = params.cd_time_to {
        query_params.insert("cdTimeTo".to_string(), cd_time_to.to_string());
    }
    if let Some(lv_area) = params.lv_area {
        query_params.insert("lvArea".to_string(), lv_area.to_string());
    }
    if let Some(cd_area) = params.cd_area {
        query_params.insert("cdArea".to_string(), cd_area.to_string());
    }
    if let Some(cd_area_from) = params.cd_area_from {
        query_params.insert("cdAreaFrom".to_string(), cd_area_from.to_string());
    }
    if let Some(cd_area_to) = params.cd_area_to {
        query_params.insert("cdAreaTo".to_string(), cd_area_to.to_string());
    }
    if let Some(lv_cat01) = params.lv_cat01 {
        query_params.insert("lvCat01".to_string(), lv_cat01.to_string());
    }
    if let Some(cd_cat01) = params.cd_cat01 {
        query_params.insert("cdCat01".to_string(), cd_cat01.to_string());
    }
    if let Some(cd_cat01_from) = params.cd_cat01_from {
        query_params.insert("cdCat01From".to_string(), cd_cat01_from.to_string());
    }
    if let Some(cd_cat01_to) = params.cd_cat01_to {
        query_params.insert("cdCat01To".to_string(), cd_cat01_to.to_string());
    }
    if let Some(lv_cat02) = params.lv_cat02 {
        query_params.insert("lvCat02".to_string(), lv_cat02.to_string());
    }
    if let Some(cd_cat02) = params.cd_cat02 {
        query_params.insert("cdCat02".to_string(), cd_cat02.to_string());
    }
    if let Some(cd_cat02_from) = params.cd_cat02_from {
        query_params.insert("cdCat02From".to_string(), cd_cat02_from.to_string());
    }
    if let Some(cd_cat02_to) = params.cd_cat02_to {
        query_params.insert("cdCat02To".to_string(), cd_cat02_to.to_string());
    }
    if let Some(lv_cat03) = params.lv_cat03 {
        query_params.insert("lvCat03".to_string(), lv_cat03.to_string());
    }
    if let Some(cd_cat03) = params.cd_cat03 {
        query_params.insert("cdCat03".to_string(), cd_cat03.to_string());
    }
    if let Some(cd_cat03_from) = params.cd_cat03_from {
        query_params.insert("cdCat03From".to_string(), cd_cat03_from.to_string());
    }
    if let Some(cd_cat03_to) = params.cd_cat03_to {
        query_params.insert("cdCat03To".to_string(), cd_cat03_to.to_string());
    }
    if let Some(lv_cat04) = params.lv_cat04 {
        query_params.insert("lvCat04".to_string(), lv_cat04.to_string());
    }
    if let Some(cd_cat04) = params.cd_cat04 {
        query_params.insert("cdCat04".to_string(), cd_cat04.to_string());
    }
    if let Some(cd_cat04_from) = params.cd_cat04_from {
        query_params.insert("cdCat04From".to_string(), cd_cat04_from.to_string());
    }
    if let Some(cd_cat04_to) = params.cd_cat04_to {
        query_params.insert("cdCat04To".to_string(), cd_cat04_to.to_string());
    }
    if let Some(lv_cat05) = params.lv_cat05 {
        query_params.insert("lvCat05".to_string(), lv_cat05.to_string());
    }
    if let Some(cd_cat05) = params.cd_cat05 {
        query_params.insert("cdCat05".to_string(), cd_cat05.to_string());
    }
    if let Some(cd_cat05_from) = params.cd_cat05_from {
        query_params.insert("cdCat05From".to_string(), cd_cat05_from.to_string());
    }
    if let Some(cd_cat05_to) = params.cd_cat05_to {
        query_params.insert("cdCat05To".to_string(), cd_cat05_to.to_string());
    }
    if let Some(lv_cat06) = params.lv_cat06 {
        query_params.insert("lvCat06".to_string(), lv_cat06.to_string());
    }
    if let Some(cd_cat06) = params.cd_cat06 {
        query_params.insert("cdCat06".to_string(), cd_cat06.to_string());
    }
    if let Some(cd_cat06_from) = params.cd_cat06_from {
        query_params.insert("cdCat06From".to_string(), cd_cat06_from.to_string());
    }
    if let Some(cd_cat06_to) = params.cd_cat06_to {
        query_params.insert("cdCat06To".to_string(), cd_cat06_to.to_string());
    }
    if let Some(lv_cat07) = params.lv_cat07 {
        query_params.insert("lvCat07".to_string(), lv_cat07.to_string());
    }
    if let Some(cd_cat07) = params.cd_cat07 {
        query_params.insert("cdCat07".to_string(), cd_cat07.to_string());
    }
    if let Some(cd_cat07_from) = params.cd_cat07_from {
        query_params.insert("cdCat07From".to_string(), cd_cat07_from.to_string());
    }
    if let Some(cd_cat07_to) = params.cd_cat07_to {
        query_params.insert("cdCat07To".to_string(), cd_cat07_to.to_string());
    }
    if let Some(lv_cat08) = params.lv_cat08 {
        query_params.insert("lvCat08".to_string(), lv_cat08.to_string());
    }
    if let Some(cd_cat08) = params.cd_cat08 {
        query_params.insert("cdCat08".to_string(), cd_cat08.to_string());
    }
    if let Some(cd_cat08_from) = params.cd_cat08_from {
        query_params.insert("cdCat08From".to_string(), cd_cat08_from.to_string());
    }
    if let Some(cd_cat08_to) = params.cd_cat08_to {
        query_params.insert("cdCat08To".to_string(), cd_cat08_to.to_string());
    }
    if let Some(lv_cat09) = params.lv_cat09 {
        query_params.insert("lvCat09".to_string(), lv_cat09.to_string());
    }
    if let Some(cd_cat09) = params.cd_cat09 {
        query_params.insert("cdCat09".to_string(), cd_cat09.to_string());
    }
    if let Some(cd_cat09_from) = params.cd_cat09_from {
        query_params.insert("cdCat09From".to_string(), cd_cat09_from.to_string());
    }
    if let Some(cd_cat09_to) = params.cd_cat09_to {
        query_params.insert("cdCat09To".to_string(), cd_cat09_to.to_string());
    }
    if let Some(lv_cat10) = params.lv_cat10 {
        query_params.insert("lvCat10".to_string(), lv_cat10.to_string());
    }
    if let Some(cd_cat10) = params.cd_cat10 {
        query_params.insert("cdCat10".to_string(), cd_cat10.to_string());
    }
    if let Some(cd_cat10_from) = params.cd_cat10_from {
        query_params.insert("cdCat10From".to_string(), cd_cat10_from.to_string());
    }
    if let Some(cd_cat10_to) = params.cd_cat10_to {
        query_params.insert("cdCat10To".to_string(), cd_cat10_to.to_string());
    }
    if let Some(lv_cat11) = params.lv_cat11 {
        query_params.insert("lvCat11".to_string(), lv_cat11.to_string());
    }
    if let Some(cd_cat11) = params.cd_cat11 {
        query_params.insert("cdCat11".to_string(), cd_cat11.to_string());
    }
    if let Some(cd_cat11_from) = params.cd_cat11_from {
        query_params.insert("cdCat11From".to_string(), cd_cat11_from.to_string());
    }
    if let Some(cd_cat11_to) = params.cd_cat11_to {
        query_params.insert("cdCat11To".to_string(), cd_cat11_to.to_string());
    }
    if let Some(lv_cat12) = params.lv_cat12 {
        query_params.insert("lvCat12".to_string(), lv_cat12.to_string());
    }
    if let Some(cd_cat12) = params.cd_cat12 {
        query_params.insert("cdCat12".to_string(), cd_cat12.to_string());
    }
    if let Some(cd_cat12_from) = params.cd_cat12_from {
        query_params.insert("cdCat12From".to_string(), cd_cat12_from.to_string());
    }
    if let Some(cd_cat12_to) = params.cd_cat12_to {
        query_params.insert("cdCat12To".to_string(), cd_cat12_to.to_string());
    }
    if let Some(lv_cat13) = params.lv_cat13 {
        query_params.insert("lvCat13".to_string(), lv_cat13.to_string());
    }
    if let Some(cd_cat13) = params.cd_cat13 {
        query_params.insert("cdCat13".to_string(), cd_cat13.to_string());
    }
    if let Some(cd_cat13_from) = params.cd_cat13_from {
        query_params.insert("cdCat13From".to_string(), cd_cat13_from.to_string());
    }
    if let Some(cd_cat13_to) = params.cd_cat13_to {
        query_params.insert("cdCat13To".to_string(), cd_cat13_to.to_string());
    }
    if let Some(lv_cat14) = params.lv_cat14 {
        query_params.insert("lvCat14".to_string(), lv_cat14.to_string());
    }
    if let Some(cd_cat14) = params.cd_cat14 {
        query_params.insert("cdCat14".to_string(), cd_cat14.to_string());
    }
    if let Some(cd_cat14_from) = params.cd_cat14_from {
        query_params.insert("cdCat14From".to_string(), cd_cat14_from.to_string());
    }
    if let Some(cd_cat14_to) = params.cd_cat14_to {
        query_params.insert("cdCat14To".to_string(), cd_cat14_to.to_string());
    }
    if let Some(lv_cat15) = params.lv_cat15 {
        query_params.insert("lvCat15".to_string(), lv_cat15.to_string());
    }
    if let Some(cd_cat15) = params.cd_cat15 {
        query_params.insert("cdCat15".to_string(), cd_cat15.to_string());
    }
    if let Some(cd_cat15_from) = params.cd_cat15_from {
        query_params.insert("cdCat15From".to_string(), cd_cat15_from.to_string());
    }
    if let Some(cd_cat15_to) = params.cd_cat15_to {
        query_params.insert("cdCat15To".to_string(), cd_cat15_to.to_string());
    }
    if let Some(start_position) = params.start_position {
        query_params.insert("startPosition".to_string(), start_position.to_string());
    }
    if let Some(limit) = params.limit {
        query_params.insert("limit".to_string(), limit.to_string());
    }
    if let Some(meta_get_flg) = params.meta_get_flg {
        query_params.insert("metaGetFlg".to_string(), meta_get_flg.to_string());
    }
    if let Some(cnt_get_flg) = params.cnt_get_flg {
        query_params.insert("cntGetFlg".to_string(), cnt_get_flg.to_string());
    }
    if let Some(explanation_get_flg) = &params.explanation_get_flg {
        query_params.insert(
            "explanationGetFlg".to_string(),
            explanation_get_flg.as_str().to_string(),
        );
    }
    if let Some(annotation_get_flg) = params.annotation_get_flg {
        query_params.insert(
            "annotationGetFlg".to_string(),
            annotation_get_flg.to_string(),
        );
    }
    if let Some(replace_sp_char) = params.replace_sp_char {
        query_params.insert("replaceSpChar".to_string(), replace_sp_char.to_string());
    }
    if let Some(callback) = params.callback {
        query_params.insert("callback".to_string(), callback.to_string());
    }
    if let Some(section_header_flg) = params.section_header_flg {
        query_params.insert(
            "sectionHeaderFlg".to_string(),
            section_header_flg.to_string(),
        );
    }

    query_params
}

pub fn get_stats_data(params: GetStatsDataParams) -> Result<String, Box<dyn Error>> {
    let query_params = construct_query_params_stats_data(&params);

    let url = "https://api.e-stat.go.jp/rest/3.0/app/json/getStatsData";
    let request = GetRequestBuilder {
        url,
        params: Some(&query_params),
    };

    let decoded_json_string = decode_escaped_json_string(&get(&request)?)?;

    Ok(decoded_json_string)
}
