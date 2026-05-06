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

pub struct MandatoryParams<'a> {
    pub app_id: &'a str,
}

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

pub struct GetStatsDataParamsBuilder<'a> {
    app_id: &'a str,
    data_set_id: Option<&'a str>,
    stats_data_id: Option<&'a str>,
    lv_tab: Option<&'a str>,
    cd_tab: Option<&'a str>,
    cd_tab_from: Option<&'a str>,
    cd_tab_to: Option<&'a str>,
    lv_time: Option<&'a str>,
    cd_time: Option<&'a str>,
    cd_time_from: Option<&'a str>,
    cd_time_to: Option<&'a str>,
    lv_area: Option<&'a str>,
    cd_area: Option<&'a str>,
    cd_area_from: Option<&'a str>,
    cd_area_to: Option<&'a str>,
    lv_cat01: Option<&'a str>,
    cd_cat01: Option<&'a str>,
    cd_cat01_from: Option<&'a str>,
    cd_cat01_to: Option<&'a str>,
    lv_cat02: Option<&'a str>,
    cd_cat02: Option<&'a str>,
    cd_cat02_from: Option<&'a str>,
    cd_cat02_to: Option<&'a str>,
    lv_cat03: Option<&'a str>,
    cd_cat03: Option<&'a str>,
    cd_cat03_from: Option<&'a str>,
    cd_cat03_to: Option<&'a str>,
    lv_cat04: Option<&'a str>,
    cd_cat04: Option<&'a str>,
    cd_cat04_from: Option<&'a str>,
    cd_cat04_to: Option<&'a str>,
    lv_cat05: Option<&'a str>,
    cd_cat05: Option<&'a str>,
    cd_cat05_from: Option<&'a str>,
    cd_cat05_to: Option<&'a str>,
    lv_cat06: Option<&'a str>,
    cd_cat06: Option<&'a str>,
    cd_cat06_from: Option<&'a str>,
    cd_cat06_to: Option<&'a str>,
    lv_cat07: Option<&'a str>,
    cd_cat07: Option<&'a str>,
    cd_cat07_from: Option<&'a str>,
    cd_cat07_to: Option<&'a str>,
    lv_cat08: Option<&'a str>,
    cd_cat08: Option<&'a str>,
    cd_cat08_from: Option<&'a str>,
    cd_cat08_to: Option<&'a str>,
    lv_cat09: Option<&'a str>,
    cd_cat09: Option<&'a str>,
    cd_cat09_from: Option<&'a str>,
    cd_cat09_to: Option<&'a str>,
    lv_cat10: Option<&'a str>,
    cd_cat10: Option<&'a str>,
    cd_cat10_from: Option<&'a str>,
    cd_cat10_to: Option<&'a str>,
    lv_cat11: Option<&'a str>,
    cd_cat11: Option<&'a str>,
    cd_cat11_from: Option<&'a str>,
    cd_cat11_to: Option<&'a str>,
    lv_cat12: Option<&'a str>,
    cd_cat12: Option<&'a str>,
    cd_cat12_from: Option<&'a str>,
    cd_cat12_to: Option<&'a str>,
    lv_cat13: Option<&'a str>,
    cd_cat13: Option<&'a str>,
    cd_cat13_from: Option<&'a str>,
    cd_cat13_to: Option<&'a str>,
    lv_cat14: Option<&'a str>,
    cd_cat14: Option<&'a str>,
    cd_cat14_from: Option<&'a str>,
    cd_cat14_to: Option<&'a str>,
    lv_cat15: Option<&'a str>,
    cd_cat15: Option<&'a str>,
    cd_cat15_from: Option<&'a str>,
    cd_cat15_to: Option<&'a str>,
    start_position: Option<u32>,
    limit: Option<u32>,
    meta_get_flg: Option<&'a str>,
    cnt_get_flg: Option<&'a str>,
    explanation_get_flg: Option<ExplanationFlag>,
    annotation_get_flg: Option<&'a str>,
    replace_sp_char: Option<u32>,
    callback: Option<&'a str>,
    section_header_flg: Option<u32>,
}

impl<'a> GetStatsDataParamsBuilder<'a> {
    pub fn new(params: MandatoryParams<'a>) -> Self {
        Self {
            app_id: params.app_id,
            data_set_id: None,
            stats_data_id: None,
            lv_tab: None,
            cd_tab: None,
            cd_tab_from: None,
            cd_tab_to: None,
            lv_time: None,
            cd_time: None,
            cd_time_from: None,
            cd_time_to: None,
            lv_area: None,
            cd_area: None,
            cd_area_from: None,
            cd_area_to: None,
            lv_cat01: None,
            cd_cat01: None,
            cd_cat01_from: None,
            cd_cat01_to: None,
            lv_cat02: None,
            cd_cat02: None,
            cd_cat02_from: None,
            cd_cat02_to: None,
            lv_cat03: None,
            cd_cat03: None,
            cd_cat03_from: None,
            cd_cat03_to: None,
            lv_cat04: None,
            cd_cat04: None,
            cd_cat04_from: None,
            cd_cat04_to: None,
            lv_cat05: None,
            cd_cat05: None,
            cd_cat05_from: None,
            cd_cat05_to: None,
            lv_cat06: None,
            cd_cat06: None,
            cd_cat06_from: None,
            cd_cat06_to: None,
            lv_cat07: None,
            cd_cat07: None,
            cd_cat07_from: None,
            cd_cat07_to: None,
            lv_cat08: None,
            cd_cat08: None,
            cd_cat08_from: None,
            cd_cat08_to: None,
            lv_cat09: None,
            cd_cat09: None,
            cd_cat09_from: None,
            cd_cat09_to: None,
            lv_cat10: None,
            cd_cat10: None,
            cd_cat10_from: None,
            cd_cat10_to: None,
            lv_cat11: None,
            cd_cat11: None,
            cd_cat11_from: None,
            cd_cat11_to: None,
            lv_cat12: None,
            cd_cat12: None,
            cd_cat12_from: None,
            cd_cat12_to: None,
            lv_cat13: None,
            cd_cat13: None,
            cd_cat13_from: None,
            cd_cat13_to: None,
            lv_cat14: None,
            cd_cat14: None,
            cd_cat14_from: None,
            cd_cat14_to: None,
            lv_cat15: None,
            cd_cat15: None,
            cd_cat15_from: None,
            cd_cat15_to: None,
            start_position: None,
            limit: None,
            meta_get_flg: None,
            cnt_get_flg: None,
            explanation_get_flg: None,
            annotation_get_flg: None,
            replace_sp_char: None,
            callback: None,
            section_header_flg: None,
        }
    }

    pub fn data_set_id(mut self, v: &'a str) -> Self { self.data_set_id = Some(v); self }
    pub fn stats_data_id(mut self, v: &'a str) -> Self { self.stats_data_id = Some(v); self }
    pub fn lv_tab(mut self, v: &'a str) -> Self { self.lv_tab = Some(v); self }
    pub fn cd_tab(mut self, v: &'a str) -> Self { self.cd_tab = Some(v); self }
    pub fn cd_tab_from(mut self, v: &'a str) -> Self { self.cd_tab_from = Some(v); self }
    pub fn cd_tab_to(mut self, v: &'a str) -> Self { self.cd_tab_to = Some(v); self }
    pub fn lv_time(mut self, v: &'a str) -> Self { self.lv_time = Some(v); self }
    pub fn cd_time(mut self, v: &'a str) -> Self { self.cd_time = Some(v); self }
    pub fn cd_time_from(mut self, v: &'a str) -> Self { self.cd_time_from = Some(v); self }
    pub fn cd_time_to(mut self, v: &'a str) -> Self { self.cd_time_to = Some(v); self }
    pub fn lv_area(mut self, v: &'a str) -> Self { self.lv_area = Some(v); self }
    pub fn cd_area(mut self, v: &'a str) -> Self { self.cd_area = Some(v); self }
    pub fn cd_area_from(mut self, v: &'a str) -> Self { self.cd_area_from = Some(v); self }
    pub fn cd_area_to(mut self, v: &'a str) -> Self { self.cd_area_to = Some(v); self }
    pub fn lv_cat01(mut self, v: &'a str) -> Self { self.lv_cat01 = Some(v); self }
    pub fn cd_cat01(mut self, v: &'a str) -> Self { self.cd_cat01 = Some(v); self }
    pub fn cd_cat01_from(mut self, v: &'a str) -> Self { self.cd_cat01_from = Some(v); self }
    pub fn cd_cat01_to(mut self, v: &'a str) -> Self { self.cd_cat01_to = Some(v); self }
    pub fn lv_cat02(mut self, v: &'a str) -> Self { self.lv_cat02 = Some(v); self }
    pub fn cd_cat02(mut self, v: &'a str) -> Self { self.cd_cat02 = Some(v); self }
    pub fn cd_cat02_from(mut self, v: &'a str) -> Self { self.cd_cat02_from = Some(v); self }
    pub fn cd_cat02_to(mut self, v: &'a str) -> Self { self.cd_cat02_to = Some(v); self }
    pub fn lv_cat03(mut self, v: &'a str) -> Self { self.lv_cat03 = Some(v); self }
    pub fn cd_cat03(mut self, v: &'a str) -> Self { self.cd_cat03 = Some(v); self }
    pub fn cd_cat03_from(mut self, v: &'a str) -> Self { self.cd_cat03_from = Some(v); self }
    pub fn cd_cat03_to(mut self, v: &'a str) -> Self { self.cd_cat03_to = Some(v); self }
    pub fn lv_cat04(mut self, v: &'a str) -> Self { self.lv_cat04 = Some(v); self }
    pub fn cd_cat04(mut self, v: &'a str) -> Self { self.cd_cat04 = Some(v); self }
    pub fn cd_cat04_from(mut self, v: &'a str) -> Self { self.cd_cat04_from = Some(v); self }
    pub fn cd_cat04_to(mut self, v: &'a str) -> Self { self.cd_cat04_to = Some(v); self }
    pub fn lv_cat05(mut self, v: &'a str) -> Self { self.lv_cat05 = Some(v); self }
    pub fn cd_cat05(mut self, v: &'a str) -> Self { self.cd_cat05 = Some(v); self }
    pub fn cd_cat05_from(mut self, v: &'a str) -> Self { self.cd_cat05_from = Some(v); self }
    pub fn cd_cat05_to(mut self, v: &'a str) -> Self { self.cd_cat05_to = Some(v); self }
    pub fn lv_cat06(mut self, v: &'a str) -> Self { self.lv_cat06 = Some(v); self }
    pub fn cd_cat06(mut self, v: &'a str) -> Self { self.cd_cat06 = Some(v); self }
    pub fn cd_cat06_from(mut self, v: &'a str) -> Self { self.cd_cat06_from = Some(v); self }
    pub fn cd_cat06_to(mut self, v: &'a str) -> Self { self.cd_cat06_to = Some(v); self }
    pub fn lv_cat07(mut self, v: &'a str) -> Self { self.lv_cat07 = Some(v); self }
    pub fn cd_cat07(mut self, v: &'a str) -> Self { self.cd_cat07 = Some(v); self }
    pub fn cd_cat07_from(mut self, v: &'a str) -> Self { self.cd_cat07_from = Some(v); self }
    pub fn cd_cat07_to(mut self, v: &'a str) -> Self { self.cd_cat07_to = Some(v); self }
    pub fn lv_cat08(mut self, v: &'a str) -> Self { self.lv_cat08 = Some(v); self }
    pub fn cd_cat08(mut self, v: &'a str) -> Self { self.cd_cat08 = Some(v); self }
    pub fn cd_cat08_from(mut self, v: &'a str) -> Self { self.cd_cat08_from = Some(v); self }
    pub fn cd_cat08_to(mut self, v: &'a str) -> Self { self.cd_cat08_to = Some(v); self }
    pub fn lv_cat09(mut self, v: &'a str) -> Self { self.lv_cat09 = Some(v); self }
    pub fn cd_cat09(mut self, v: &'a str) -> Self { self.cd_cat09 = Some(v); self }
    pub fn cd_cat09_from(mut self, v: &'a str) -> Self { self.cd_cat09_from = Some(v); self }
    pub fn cd_cat09_to(mut self, v: &'a str) -> Self { self.cd_cat09_to = Some(v); self }
    pub fn lv_cat10(mut self, v: &'a str) -> Self { self.lv_cat10 = Some(v); self }
    pub fn cd_cat10(mut self, v: &'a str) -> Self { self.cd_cat10 = Some(v); self }
    pub fn cd_cat10_from(mut self, v: &'a str) -> Self { self.cd_cat10_from = Some(v); self }
    pub fn cd_cat10_to(mut self, v: &'a str) -> Self { self.cd_cat10_to = Some(v); self }
    pub fn lv_cat11(mut self, v: &'a str) -> Self { self.lv_cat11 = Some(v); self }
    pub fn cd_cat11(mut self, v: &'a str) -> Self { self.cd_cat11 = Some(v); self }
    pub fn cd_cat11_from(mut self, v: &'a str) -> Self { self.cd_cat11_from = Some(v); self }
    pub fn cd_cat11_to(mut self, v: &'a str) -> Self { self.cd_cat11_to = Some(v); self }
    pub fn lv_cat12(mut self, v: &'a str) -> Self { self.lv_cat12 = Some(v); self }
    pub fn cd_cat12(mut self, v: &'a str) -> Self { self.cd_cat12 = Some(v); self }
    pub fn cd_cat12_from(mut self, v: &'a str) -> Self { self.cd_cat12_from = Some(v); self }
    pub fn cd_cat12_to(mut self, v: &'a str) -> Self { self.cd_cat12_to = Some(v); self }
    pub fn lv_cat13(mut self, v: &'a str) -> Self { self.lv_cat13 = Some(v); self }
    pub fn cd_cat13(mut self, v: &'a str) -> Self { self.cd_cat13 = Some(v); self }
    pub fn cd_cat13_from(mut self, v: &'a str) -> Self { self.cd_cat13_from = Some(v); self }
    pub fn cd_cat13_to(mut self, v: &'a str) -> Self { self.cd_cat13_to = Some(v); self }
    pub fn lv_cat14(mut self, v: &'a str) -> Self { self.lv_cat14 = Some(v); self }
    pub fn cd_cat14(mut self, v: &'a str) -> Self { self.cd_cat14 = Some(v); self }
    pub fn cd_cat14_from(mut self, v: &'a str) -> Self { self.cd_cat14_from = Some(v); self }
    pub fn cd_cat14_to(mut self, v: &'a str) -> Self { self.cd_cat14_to = Some(v); self }
    pub fn lv_cat15(mut self, v: &'a str) -> Self { self.lv_cat15 = Some(v); self }
    pub fn cd_cat15(mut self, v: &'a str) -> Self { self.cd_cat15 = Some(v); self }
    pub fn cd_cat15_from(mut self, v: &'a str) -> Self { self.cd_cat15_from = Some(v); self }
    pub fn cd_cat15_to(mut self, v: &'a str) -> Self { self.cd_cat15_to = Some(v); self }
    pub fn start_position(mut self, v: u32) -> Self { self.start_position = Some(v); self }
    pub fn limit(mut self, v: u32) -> Self { self.limit = Some(v); self }
    pub fn meta_get_flg(mut self, v: &'a str) -> Self { self.meta_get_flg = Some(v); self }
    pub fn cnt_get_flg(mut self, v: &'a str) -> Self { self.cnt_get_flg = Some(v); self }
    pub fn explanation_get_flg(mut self, v: ExplanationFlag) -> Self { self.explanation_get_flg = Some(v); self }
    pub fn annotation_get_flg(mut self, v: &'a str) -> Self { self.annotation_get_flg = Some(v); self }
    pub fn replace_sp_char(mut self, v: u32) -> Self { self.replace_sp_char = Some(v); self }
    pub fn callback(mut self, v: &'a str) -> Self { self.callback = Some(v); self }
    pub fn section_header_flg(mut self, v: u32) -> Self { self.section_header_flg = Some(v); self }

    pub fn build(self) -> GetStatsDataParams<'a> {
        GetStatsDataParams {
            app_id: self.app_id,
            data_set_id: self.data_set_id,
            stats_data_id: self.stats_data_id,
            lv_tab: self.lv_tab,
            cd_tab: self.cd_tab,
            cd_tab_from: self.cd_tab_from,
            cd_tab_to: self.cd_tab_to,
            lv_time: self.lv_time,
            cd_time: self.cd_time,
            cd_time_from: self.cd_time_from,
            cd_time_to: self.cd_time_to,
            lv_area: self.lv_area,
            cd_area: self.cd_area,
            cd_area_from: self.cd_area_from,
            cd_area_to: self.cd_area_to,
            lv_cat01: self.lv_cat01,
            cd_cat01: self.cd_cat01,
            cd_cat01_from: self.cd_cat01_from,
            cd_cat01_to: self.cd_cat01_to,
            lv_cat02: self.lv_cat02,
            cd_cat02: self.cd_cat02,
            cd_cat02_from: self.cd_cat02_from,
            cd_cat02_to: self.cd_cat02_to,
            lv_cat03: self.lv_cat03,
            cd_cat03: self.cd_cat03,
            cd_cat03_from: self.cd_cat03_from,
            cd_cat03_to: self.cd_cat03_to,
            lv_cat04: self.lv_cat04,
            cd_cat04: self.cd_cat04,
            cd_cat04_from: self.cd_cat04_from,
            cd_cat04_to: self.cd_cat04_to,
            lv_cat05: self.lv_cat05,
            cd_cat05: self.cd_cat05,
            cd_cat05_from: self.cd_cat05_from,
            cd_cat05_to: self.cd_cat05_to,
            lv_cat06: self.lv_cat06,
            cd_cat06: self.cd_cat06,
            cd_cat06_from: self.cd_cat06_from,
            cd_cat06_to: self.cd_cat06_to,
            lv_cat07: self.lv_cat07,
            cd_cat07: self.cd_cat07,
            cd_cat07_from: self.cd_cat07_from,
            cd_cat07_to: self.cd_cat07_to,
            lv_cat08: self.lv_cat08,
            cd_cat08: self.cd_cat08,
            cd_cat08_from: self.cd_cat08_from,
            cd_cat08_to: self.cd_cat08_to,
            lv_cat09: self.lv_cat09,
            cd_cat09: self.cd_cat09,
            cd_cat09_from: self.cd_cat09_from,
            cd_cat09_to: self.cd_cat09_to,
            lv_cat10: self.lv_cat10,
            cd_cat10: self.cd_cat10,
            cd_cat10_from: self.cd_cat10_from,
            cd_cat10_to: self.cd_cat10_to,
            lv_cat11: self.lv_cat11,
            cd_cat11: self.cd_cat11,
            cd_cat11_from: self.cd_cat11_from,
            cd_cat11_to: self.cd_cat11_to,
            lv_cat12: self.lv_cat12,
            cd_cat12: self.cd_cat12,
            cd_cat12_from: self.cd_cat12_from,
            cd_cat12_to: self.cd_cat12_to,
            lv_cat13: self.lv_cat13,
            cd_cat13: self.cd_cat13,
            cd_cat13_from: self.cd_cat13_from,
            cd_cat13_to: self.cd_cat13_to,
            lv_cat14: self.lv_cat14,
            cd_cat14: self.cd_cat14,
            cd_cat14_from: self.cd_cat14_from,
            cd_cat14_to: self.cd_cat14_to,
            lv_cat15: self.lv_cat15,
            cd_cat15: self.cd_cat15,
            cd_cat15_from: self.cd_cat15_from,
            cd_cat15_to: self.cd_cat15_to,
            start_position: self.start_position,
            limit: self.limit,
            meta_get_flg: self.meta_get_flg,
            cnt_get_flg: self.cnt_get_flg,
            explanation_get_flg: self.explanation_get_flg,
            annotation_get_flg: self.annotation_get_flg,
            replace_sp_char: self.replace_sp_char,
            callback: self.callback,
            section_header_flg: self.section_header_flg,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mandatory_params_only() {
        let params = GetStatsDataParamsBuilder::new(MandatoryParams { app_id: "test_app" }).build();
        let query_params = construct_query_params_stats_data(&params);

        assert_eq!(query_params.len(), 1);
        assert_eq!(query_params.get("appId"), Some(&"test_app".to_string()));
    }

    #[test]
    fn test_explanation_get_flg_as_str() {
        let params = GetStatsDataParamsBuilder::new(MandatoryParams { app_id: "test_app" })
            .explanation_get_flg(ExplanationFlag::Exclude)
            .build();
        let query_params = construct_query_params_stats_data(&params);

        assert_eq!(query_params.len(), 2);
        assert_eq!(
            query_params.get("explanationGetFlg"),
            Some(&"N".to_string())
        );
    }

    #[test]
    fn test_optional_str_param() {
        let params = GetStatsDataParamsBuilder::new(MandatoryParams { app_id: "test_app" })
            .stats_data_id("0003000")
            .cd_cat02("001")
            .build();
        let query_params = construct_query_params_stats_data(&params);

        assert_eq!(query_params.len(), 3);
        assert_eq!(
            query_params.get("statsDataId"),
            Some(&"0003000".to_string())
        );
        assert_eq!(query_params.get("cdCat02"), Some(&"001".to_string()));
    }
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
