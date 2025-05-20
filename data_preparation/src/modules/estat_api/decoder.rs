use serde_json::{Value, from_str, to_string_pretty};
use std::error::Error;

/// Function to decode escaped characters in a JSON string
///
/// The JSON returned from e-stat API contains escaped characters.
/// This function decodes those characters to make the JSON human-readable.
pub fn decode_escaped_json_string(json_string: &str) -> Result<String, Box<dyn Error>> {
    let value: Value = from_str(json_string)?;
    let decoded_json_string = to_string_pretty(&value)?;
    Ok(decoded_json_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_data() -> Result<(), Box<dyn Error>> {
        let input = r#"{"GET_STATS_LIST":{"PARAMETER":{"DATA_FORMAT":"J"},"RESULT":{"DATE":"2025-03-01T12:00:00.000+09:00","ERROR_MSG":"\u8A8D\u8A3C\u306B\u5931\u6557\u3057\u307E\u3057\u305F\u3002\u30A2\u30D7\u30EA\u30B1\u30FC\u30B7\u30E7\u30F3ID\u3092\u78BA\u8A8D\u3057\u3066\u4E0B\u3055\u3044\u3002","STATUS":100}}}"#;
        let expected_json = json!({
            "GET_STATS_LIST": {
                "PARAMETER": {
                    "DATA_FORMAT": "J"
                },
                "RESULT": {
                    "DATE": "2025-03-01T12:00:00.000+09:00",
                    "ERROR_MSG": "認証に失敗しました。アプリケーションIDを確認して下さい。",
                    "STATUS": 100
                }
            }
        });
        let expected = to_string_pretty(&expected_json)?;
        let result = decode_escaped_json_string(input)?;
        assert_eq!(result, expected);
        Ok(())
    }
}
