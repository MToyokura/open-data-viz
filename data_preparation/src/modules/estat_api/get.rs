use reqwest::Url;
use reqwest::blocking;
use std::collections::HashMap;
use std::error::Error;

pub(crate) struct GetRequestBuilder<'a> {
    pub url: &'a str,
    pub params: Option<&'a HashMap<String, String>>,
}

pub(crate) fn get(request: &GetRequestBuilder) -> Result<String, Box<dyn Error>> {
    let client = blocking::Client::new();

    // Parse URL and attach query parameters if provided
    let url = match request.params {
        Some(params) => Url::parse_with_params(request.url, params.iter())?,
        None => Url::parse(request.url)?, // No params, just parse the base URL
    };

    let response_text = client.get(url).send()?.text()?;

    Ok(response_text)
}
