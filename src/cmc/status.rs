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

const INSERT_STATUS: &str = "INSERT INTO updates (
    url,
    error_code,
    error_message,
    credit_count,
    timestamp,
    elapsed,
    notice
) VALUES ($1, $2, $3, $4, $5, make_interval(secs => $6), $7)
RETURNING id";

impl Status {
    pub async fn insert(&self, pg: &Client, url: &str) -> Result<i32, Error> {
        Ok(pg
            .query_one(
                INSERT_STATUS,
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
