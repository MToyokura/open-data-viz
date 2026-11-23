use serde::Serialize;

use super::{EstatClient, EstatError, decoder::decode_response, get_meta_info::ExplanationFlag};

pub(crate) enum StatsDataId<'a> {
    Table(&'a str),
    // Dataset requests use the same endpoint. Current pipelines use table IDs.
    #[allow(dead_code)]
    Dataset(&'a str),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetStatsDataParams<'a> {
    app_id: &'a str,
    stats_data_id: Option<&'a str>,
    data_set_id: Option<&'a str>,
    #[serde(flatten)]
    options: StatsDataOptions<'a>,
}

impl<'a> GetStatsDataParams<'a> {
    pub(crate) fn new(app_id: &'a str, id: StatsDataId<'a>, options: StatsDataOptions<'a>) -> Self {
        let (stats_data_id, data_set_id) = match id {
            StatsDataId::Table(id) => (Some(id), None),
            StatsDataId::Dataset(id) => (None, Some(id)),
        };
        Self {
            app_id,
            stats_data_id,
            data_set_id,
            options,
        }
    }
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StatsDataOptions<'a> {
    pub(crate) lv_tab: Option<&'a str>,
    pub(crate) cd_tab: Option<&'a str>,
    pub(crate) cd_tab_from: Option<&'a str>,
    pub(crate) cd_tab_to: Option<&'a str>,
    pub(crate) lv_time: Option<&'a str>,
    pub(crate) cd_time: Option<&'a str>,
    pub(crate) cd_time_from: Option<&'a str>,
    pub(crate) cd_time_to: Option<&'a str>,
    pub(crate) lv_area: Option<&'a str>,
    pub(crate) cd_area: Option<&'a str>,
    pub(crate) cd_area_from: Option<&'a str>,
    pub(crate) cd_area_to: Option<&'a str>,
    pub(crate) lv_cat01: Option<&'a str>,
    pub(crate) cd_cat01: Option<&'a str>,
    pub(crate) cd_cat01_from: Option<&'a str>,
    pub(crate) cd_cat01_to: Option<&'a str>,
    pub(crate) lv_cat02: Option<&'a str>,
    pub(crate) cd_cat02: Option<&'a str>,
    pub(crate) cd_cat02_from: Option<&'a str>,
    pub(crate) cd_cat02_to: Option<&'a str>,
    pub(crate) lv_cat03: Option<&'a str>,
    pub(crate) cd_cat03: Option<&'a str>,
    pub(crate) cd_cat03_from: Option<&'a str>,
    pub(crate) cd_cat03_to: Option<&'a str>,
    pub(crate) lv_cat04: Option<&'a str>,
    pub(crate) cd_cat04: Option<&'a str>,
    pub(crate) cd_cat04_from: Option<&'a str>,
    pub(crate) cd_cat04_to: Option<&'a str>,
    pub(crate) lv_cat05: Option<&'a str>,
    pub(crate) cd_cat05: Option<&'a str>,
    pub(crate) cd_cat05_from: Option<&'a str>,
    pub(crate) cd_cat05_to: Option<&'a str>,
    pub(crate) lv_cat06: Option<&'a str>,
    pub(crate) cd_cat06: Option<&'a str>,
    pub(crate) cd_cat06_from: Option<&'a str>,
    pub(crate) cd_cat06_to: Option<&'a str>,
    pub(crate) lv_cat07: Option<&'a str>,
    pub(crate) cd_cat07: Option<&'a str>,
    pub(crate) cd_cat07_from: Option<&'a str>,
    pub(crate) cd_cat07_to: Option<&'a str>,
    pub(crate) lv_cat08: Option<&'a str>,
    pub(crate) cd_cat08: Option<&'a str>,
    pub(crate) cd_cat08_from: Option<&'a str>,
    pub(crate) cd_cat08_to: Option<&'a str>,
    pub(crate) lv_cat09: Option<&'a str>,
    pub(crate) cd_cat09: Option<&'a str>,
    pub(crate) cd_cat09_from: Option<&'a str>,
    pub(crate) cd_cat09_to: Option<&'a str>,
    pub(crate) lv_cat10: Option<&'a str>,
    pub(crate) cd_cat10: Option<&'a str>,
    pub(crate) cd_cat10_from: Option<&'a str>,
    pub(crate) cd_cat10_to: Option<&'a str>,
    pub(crate) lv_cat11: Option<&'a str>,
    pub(crate) cd_cat11: Option<&'a str>,
    pub(crate) cd_cat11_from: Option<&'a str>,
    pub(crate) cd_cat11_to: Option<&'a str>,
    pub(crate) lv_cat12: Option<&'a str>,
    pub(crate) cd_cat12: Option<&'a str>,
    pub(crate) cd_cat12_from: Option<&'a str>,
    pub(crate) cd_cat12_to: Option<&'a str>,
    pub(crate) lv_cat13: Option<&'a str>,
    pub(crate) cd_cat13: Option<&'a str>,
    pub(crate) cd_cat13_from: Option<&'a str>,
    pub(crate) cd_cat13_to: Option<&'a str>,
    pub(crate) lv_cat14: Option<&'a str>,
    pub(crate) cd_cat14: Option<&'a str>,
    pub(crate) cd_cat14_from: Option<&'a str>,
    pub(crate) cd_cat14_to: Option<&'a str>,
    pub(crate) lv_cat15: Option<&'a str>,
    pub(crate) cd_cat15: Option<&'a str>,
    pub(crate) cd_cat15_from: Option<&'a str>,
    pub(crate) cd_cat15_to: Option<&'a str>,
    pub(crate) start_position: Option<u32>,
    pub(crate) limit: Option<u32>,
    pub(crate) meta_get_flg: Option<&'a str>,
    pub(crate) cnt_get_flg: Option<&'a str>,
    pub(crate) explanation_get_flg: Option<ExplanationFlag>,
    pub(crate) annotation_get_flg: Option<&'a str>,
    pub(crate) replace_sp_char: Option<u32>,
}

const URL: &str = "https://api.e-stat.go.jp/rest/3.0/app/json/getStatsData";

impl EstatClient {
    pub(crate) fn get_stats_data(
        &self,
        params: GetStatsDataParams<'_>,
    ) -> Result<String, EstatError> {
        decode_response(&self.get(URL, &params)?, "GET_STATS_DATA")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn query(params: &GetStatsDataParams<'_>) -> HashMap<String, String> {
        let request = reqwest::blocking::Client::new()
            .get(URL)
            .query(params)
            .build()
            .unwrap();
        request.url().query_pairs().into_owned().collect()
    }

    #[test]
    fn sends_exactly_one_identifier() {
        for (id, key) in [
            (StatsDataId::Table("0003000"), "statsDataId"),
            (StatsDataId::Dataset("example"), "dataSetId"),
        ] {
            let expected_id = match id {
                StatsDataId::Table(id) | StatsDataId::Dataset(id) => id,
            };
            let params = GetStatsDataParams::new("test app", id, StatsDataOptions::default());
            let query = query(&params);
            assert_eq!(query.len(), 2);
            assert_eq!(query["appId"], "test app");
            assert_eq!(query[key], expected_id);
        }
    }

    #[test]
    fn serializes_filters_and_flags_with_api_parameter_names() {
        let params = GetStatsDataParams::new(
            "test_app",
            StatsDataId::Table("0003000"),
            StatsDataOptions {
                cd_cat02: Some("001,002"),
                cd_cat15_from: Some("001"),
                start_position: Some(1),
                explanation_get_flg: Some(ExplanationFlag::Exclude),
                ..Default::default()
            },
        );
        assert_eq!(
            query(&params),
            [
                ("appId", "test_app"),
                ("statsDataId", "0003000"),
                ("cdCat02", "001,002"),
                ("cdCat15From", "001"),
                ("startPosition", "1"),
                ("explanationGetFlg", "N"),
            ]
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .into(),
        );
    }
}
