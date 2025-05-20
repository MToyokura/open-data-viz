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

#[derive(Default)]
pub struct GetMetaInfoParams<'a> {
    pub app_id: &'a str,
    pub lang: Option<&'a str>, // "J" or "E"
    pub stats_data_id: &'a str,
    pub explanation_get_flg: Option<ExplanationFlag>, // "Y" or "N"
    pub callback: Option<&'a str>,                    // JSONP callback function
}

fn construct_query_params<'a>(params: &GetMetaInfoParams<'a>) -> HashMap<String, String> {
    let mut query_params = HashMap::new();

    // Mandatory parameters
    query_params.insert("appId".to_string(), params.app_id.to_string());
    query_params.insert("statsDataId".to_string(), params.stats_data_id.to_string());

    // Optional parameters (add only if Some)
    if let Some(lang) = params.lang {
        query_params.insert("lang".to_string(), lang.to_string());
    }
    if let Some(explanation_get_flg) = &params.explanation_get_flg {
        query_params.insert(
            "explanationGetFlg".to_string(),
            explanation_get_flg.as_str().to_string(),
        );
    }
    if let Some(callback) = params.callback {
        query_params.insert("callback".to_string(), callback.to_string());
    }

    query_params
}

pub fn get_meta_info(params: GetMetaInfoParams) -> Result<String, Box<dyn Error>> {
    let query_params = construct_query_params(&params);

    let url = "https://api.e-stat.go.jp/rest/3.0/app/json/getMetaInfo";
    let request = GetRequestBuilder {
        url,
        params: Some(&query_params),
    };

    let decoded_json_string = decode_escaped_json_string(&get(&request)?)?;

    Ok(decoded_json_string)
}
