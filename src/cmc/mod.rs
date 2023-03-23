use reqwest::RequestBuilder;
use serde::Deserialize;

use crate::cmc::status::Status;

pub mod enums;
pub mod status;

pub mod cryptocurrencies;
pub mod fiats;

pub struct API {
    endpoint: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub status: Status,
    pub data: Option<T>,
}

impl API {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            client: reqwest::Client::new(),
        }
    }

    fn get(&self, endpoint: &str) -> RequestBuilder {
        self.client
            .get([&self.endpoint, endpoint].join(""))
            .header("Accept", "application/json")
    }
}
