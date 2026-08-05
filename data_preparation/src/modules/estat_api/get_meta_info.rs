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

pub struct MandatoryParams<'a> {
    pub app_id: &'a str,
    pub stats_data_id: &'a str,
}
pub struct GetMetaInfoParams<'a> {
    pub app_id: &'a str,
    pub stats_data_id: &'a str,
    pub lang: Option<&'a str>,                        // "J" or "E"
    pub explanation_get_flg: Option<ExplanationFlag>, // "Y" or "N"
    pub callback: Option<&'a str>,                    // JSONP callback function
}

pub struct GetMetaInfoParamsBuilder<'a> {
    app_id: &'a str,
    stats_data_id: &'a str,
    lang: Option<&'a str>,
    explanation_get_flg: Option<ExplanationFlag>,
    callback: Option<&'a str>,
}

impl<'a> GetMetaInfoParamsBuilder<'a> {
    // Constructor with mandatory parameters
    pub fn new(params: MandatoryParams<'a>) -> Self {
        Self {
            app_id: params.app_id,
            stats_data_id: params.stats_data_id,
            lang: None,
            explanation_get_flg: None,
            callback: None,
        }
    }

    // Fluent methods for optional parameters
    pub fn lang(mut self, lang: &'a str) -> Self {
        self.lang = Some(lang);
        self
    }

    pub fn explanation_get_flg(mut self, flag: ExplanationFlag) -> Self {
        self.explanation_get_flg = Some(flag);
        self
    }

    pub fn callback(mut self, callback: &'a str) -> Self {
        self.callback = Some(callback);
        self
    }

    // Build method to create the final GetMetaInfoParams struct
    pub fn build(self) -> GetMetaInfoParams<'a> {
        GetMetaInfoParams {
            app_id: self.app_id,
            stats_data_id: self.stats_data_id,
            lang: self.lang,
            explanation_get_flg: self.explanation_get_flg,
            callback: self.callback,
        }
    }
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

#[cfg(test)]
mod tests {
    // Import everything from the parent module
    use super::*;

    // A helper function to create a basic, mandatory-only params struct
    fn create_base_params() -> GetMetaInfoParams<'static> {
        GetMetaInfoParamsBuilder::new(MandatoryParams {
            app_id: "test_app",
            stats_data_id: "test_stats_data",
        })
        .build()
    }

    #[test]
    fn test_mandatory_params_only() {
        let params = create_base_params();
        let query_params = construct_query_params(&params);

        // Assert that exactly 2 parameters are present
        assert_eq!(query_params.len(), 2, "Should only have mandatory params");

        // Assert the mandatory values are correct
        assert_eq!(query_params.get("appId"), Some(&"test_app".to_string()));
        assert_eq!(
            query_params.get("statsDataId"),
            Some(&"test_stats_data".to_string())
        );
    }

    #[test]
    fn test_all_optional_params_included() {
        let params = GetMetaInfoParamsBuilder::new(MandatoryParams {
            app_id: "full_test_app",
            stats_data_id: "full_test_stats",
        })
        .lang("E")
        .explanation_get_flg(ExplanationFlag::Exclude)
        .callback("my_jsonp_function")
        .build();

        let query_params = construct_query_params(&params);

        // Assert that all 5 parameters (2 mandatory + 3 optional) are present
        assert_eq!(query_params.len(), 5, "Should have all parameters");

        // Assert the optional values are correctly translated
        assert_eq!(query_params.get("lang"), Some(&"E".to_string()));
        assert_eq!(
            query_params.get("explanationGetFlg"),
            Some(&"N".to_string())
        );
        assert_eq!(
            query_params.get("callback"),
            Some(&"my_jsonp_function".to_string())
        );
    }

    #[test]
    fn test_mixed_optional_params() {
        // Only including lang and explanation_get_flg
        let params = GetMetaInfoParamsBuilder::new(MandatoryParams {
            app_id: "mixed_app",
            stats_data_id: "mixed_stats",
        })
        .lang("J")
        .build(); // No callback or explanation_get_flg set

        let query_params = construct_query_params(&params);

        // Assert that exactly 3 parameters (2 mandatory + 1 optional) are present
        assert_eq!(
            query_params.len(),
            3,
            "Should have 3 parameters: app, stats, lang"
        );

        // Assert the included optional value is correct
        assert_eq!(query_params.get("lang"), Some(&"J".to_string()));

        // Assert the excluded optional values are missing
        assert!(query_params.get("callback").is_none());
        assert!(query_params.get("explanationGetFlg").is_none());
    }
}
