use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::time::Duration;
use tokio_postgres::Client;

use crate::error::Error;

#[derive(Debug, Deserialize)]
pub struct Status {
    error_code: i32,
    error_message: Option<String>,
    credit_count: i32,
    timestamp: DateTime<Utc>,
    #[serde(rename = "elapsed")]
    elapsed_ms: u64,
    notice: Option<String>,
}

impl Status {
    pub fn check(&self) -> Result<(), Error> {
        match self.error_code {
            0 => Ok(()),
            code => Err(Error::new_with_code(
                self.error_message.as_ref().unwrap_or(&"".to_string()),
                code,
            )),
        }
    }

    pub async fn insert(&self, pg: &Client, url: &str) -> Result<i32, Error> {
        Ok(pg
            .query_one(
                include_str!("sql/updates_insert.sql"),
                &[
                    &url,
                    &self.error_code,
                    &self.error_message,
                    &self.credit_count,
                    &self.timestamp.naive_utc(),
                    &self.elapsed().as_secs_f64(),
                    &self.notice,
                ],
            )
            .await?
            .get(0))
    }

    fn elapsed(&self) -> Duration {
        Duration::from_millis(self.elapsed_ms)
    }
}
