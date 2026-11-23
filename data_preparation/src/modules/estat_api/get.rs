use reqwest::blocking;
use reqwest::{Error as RequestError, Url};
use std::collections::HashMap;
use std::error::Error;

pub(crate) struct GetRequestBuilder<'a> {
    pub url: &'a str,
    pub params: Option<&'a HashMap<String, String>>,
}

pub(crate) struct EstatClient(blocking::Client);

impl EstatClient {
    pub(crate) fn new() -> Result<Self, reqwest::Error> {
        blocking::Client::builder()
            .build()
            .map(Self)
            .map_err(RequestError::without_url)
    }

    pub(super) fn get(&self, request: &GetRequestBuilder) -> Result<String, Box<dyn Error>> {
        // Parse URL and attach query parameters if provided
        let url = match request.params {
            Some(params) => Url::parse_with_params(request.url, params.iter())?,
            None => Url::parse(request.url)?, // No params, just parse the base URL
        };

        let response_text = self
            .0
            .get(url)
            .send()
            .map_err(RequestError::without_url)?
            .error_for_status()
            .map_err(RequestError::without_url)?
            .text()
            .map_err(RequestError::without_url)?;

        Ok(response_text)
    }
}
