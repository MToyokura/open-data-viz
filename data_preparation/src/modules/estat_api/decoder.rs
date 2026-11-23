use serde_json::{Value, from_str, to_string_pretty};
use std::error::Error;

/// Validates the e-Stat result and formats the JSON for saving as a raw input.
pub(crate) fn decode_response(json: &str, root: &str) -> Result<String, Box<dyn Error>> {
    let value: Value = from_str(json)?;
    match value[root]["RESULT"]["STATUS"].as_i64() {
        // Status 1 is a successful request with no matching data.
        Some(0 | 1) => Ok(to_string_pretty(&value)?),
        // Avoid logging untrusted response text that could echo query parameters.
        Some(status) => Err(format!("e-Stat {root} failed with API status {status}").into()),
        None => Err(format!("e-Stat {root} response omitted a valid RESULT.STATUS").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_successful_responses_and_preserves_no_data() {
        for status in [0, 1] {
            let json = format!(
                r#"{{"GET_STATS_LIST":{{"RESULT":{{"STATUS":{status},"ERROR_MSG":"\u7D71\u8A08"}}}}}}"#
            );
            let decoded = decode_response(&json, "GET_STATS_LIST").unwrap();
            assert!(decoded.contains("統計"));
            assert_eq!(
                from_str::<Value>(&decoded).unwrap(),
                from_str::<Value>(&json).unwrap()
            );
        }
    }

    #[test]
    fn rejects_api_errors_without_exposing_upstream_messages() {
        let json = r#"{"GET_STATS_DATA":{"RESULT":{"STATUS":100,"ERROR_MSG":"query details"}}}"#;
        let error = decode_response(json, "GET_STATS_DATA").unwrap_err();
        assert_eq!(
            error.to_string(),
            "e-Stat GET_STATS_DATA failed with API status 100"
        );
    }

    #[test]
    fn requires_a_result_status_under_the_expected_root() {
        let json = r#"{"GET_META_INFO":{"RESULT":{"STATUS":0}}}"#;
        assert!(decode_response(json, "GET_STATS_DATA").is_err());
    }
}
