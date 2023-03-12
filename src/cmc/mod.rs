use reqwest::RequestBuilder;
use serde::Deserialize;

use crate::cmc::status::Status;

pub mod fiat;
pub mod status;

const API_HEADER: &str = "X-CMC_PRO_API_KEY";

pub struct API {
    domain: String,
    api_key: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub status: Status,
    pub data: T,
}

impl API {
    pub fn new(domain: &str, api_key: &str) -> Self {
        Self {
            domain: domain.to_string(),
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    fn get(&self, endpoint: &str) -> RequestBuilder {
        self.client
            .get(format!("https://{}{}", self.domain, endpoint))
            .header(API_HEADER, &self.api_key)
    }
}
