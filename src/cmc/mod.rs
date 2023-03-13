use reqwest::RequestBuilder;
use serde::Deserialize;

use crate::cmc::status::Status;

pub mod enums;
pub mod status;

pub mod cryptocurrencies;
pub mod fiats;

const API_HEADER: &str = "X-CMC_PRO_API_KEY";

pub struct API {
    endpoint: String,
    api_key: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub status: Status,
    pub data: Option<T>,
}

impl API {
    pub fn new(endpoint: &str, api_key: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    fn get(&self, endpoint: &str) -> RequestBuilder {
        self.client
            .get([&self.endpoint, endpoint].join(""))
            .header("Accept", "application/json")
            .header(API_HEADER, &self.api_key)
    }
}
