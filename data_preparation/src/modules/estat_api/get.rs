use reqwest::blocking;
use serde::Serialize;

use super::EstatError;

pub(crate) struct EstatClient(blocking::Client);

impl EstatClient {
    pub(crate) fn new() -> Result<Self, EstatError> {
        Ok(Self(blocking::Client::builder().build()?))
    }

    pub(super) fn get(&self, url: &str, query: &impl Serialize) -> Result<String, EstatError> {
        Ok(self
            .0
            .get(url)
            .query(query)
            .send()?
            .error_for_status()?
            .text()?)
    }
}
