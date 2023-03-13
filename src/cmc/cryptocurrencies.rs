use chrono::{DateTime, Utc};
use serde::Deserialize;
use tokio_postgres::Client;

use crate::cmc::{Response, API};
use crate::cmc::enums::TrackingStatus;
use crate::error::Error;

#[derive(Debug, Deserialize)]
pub struct Cryptocurrency {
    id: i32,
    name: String,
    symbol: String,
    slug: String,
    is_active: i32,
    status: String,
    first_historical_data: Option<DateTime<Utc>>,
    last_historical_data: Option<DateTime<Utc>>,
    platform: Option<Platform>,
}

#[derive(Debug, Deserialize)]
struct Platform {
    id: i32,
    token_address: String,
    // NOTE: [name, symbol, slug] ignored.
}

impl API {
    pub async fn update_cryptocurrencies(&self, pg: &Client) -> Result<(), Error> {
        let req = self.cryptocurrency_map()?;
        let url = req.url().as_str().to_string();
        let res: Response<Vec<Cryptocurrency>> = self.client.execute(req).await?.json().await?;
        res.status.check()?;

        let update = res.status.insert(&pg, &url).await?;
        for crypto in res.data.unwrap_or(vec![]) {
            crypto.insert(&pg, update).await?;
        }

        Ok(())
    }

    fn cryptocurrency_map(&self) -> Result<reqwest::Request, Error> {
        Ok(self
            .get("/v1/cryptocurrency/map")
            .query(&[
                (
                    "listing_status",
                    ["active", "inactive", "untracked"].join(","),
                ),
                (
                    "aux",
                    [
                        "platform",
                        "first_historical_data",
                        "last_historical_data",
                        "is_active",
                        "status",
                    ]
                    .join(","),
                ),
            ])
            .build()?)
    }
}

impl Cryptocurrency {
    pub async fn insert(&self, pg: &Client, update: i32) -> Result<(), Error> {
        pg.execute(
            include_str!("sql/cryptocurrencies_insert.sql"),
            &[
                &self.id,
                &self.name,
                &self.symbol,
                &self.slug,
                &(if self.is_active == 0 { false } else { true }),
                &TrackingStatus::new(&self.status),
                &self.first_historical_data.map(|dt| dt.naive_utc()),
                &self.last_historical_data.map(|dt| dt.naive_utc()),
                &update,
            ],
        )
        .await?;

        Ok(())
    }
}
