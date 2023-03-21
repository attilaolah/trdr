use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use tokio::join;
use tokio_postgres::Client;

use crate::cmc::enums::TrackingStatus;
use crate::cmc::{Response, API};
use crate::error::Error;
use crate::sql::{upsert_into, SqlVals, MAX_PARAMS};

#[derive(Debug, Deserialize)]
struct Cryptocurrency {
    id: i32,
    name: String,
    symbol: String,
    slug: String,
    #[serde(deserialize_with = "parse_bool")]
    is_active: bool,
    #[serde(deserialize_with = "TrackingStatus::parse")]
    status: TrackingStatus,
    first_historical_data: Option<DateTime<Utc>>,
    last_historical_data: Option<DateTime<Utc>>,
    platform: Option<Platform>,
}

#[derive(Debug, Deserialize)]
struct Platform {
    id: i32,
    token_address: String,
    // Ignored: [name, symbol, slug].
}

impl API {
    pub async fn update_cryptocurrencies(&self, pg: &Client) -> Result<(), Error> {
        let req = self.cryptocurrency_map()?;
        let url = req.url().as_str().to_string();
        let res: Response<Vec<Cryptocurrency>> = self.client.execute(req).await?.json().await?;
        res.status.check()?;

        for data in res
            .data
            .unwrap_or(vec![])
            .chunks(Cryptocurrency::chunk_size())
        {
            let stmt_text = Cryptocurrency::upsert_into(data.len());
            let (stmt, update) = join!(pg.prepare(&stmt_text), res.status.insert(&pg, &url));
            let (stmt, update) = (stmt?, update?);

            let vals: SqlVals = data
                .iter()
                .flat_map(|c| vec![c.sql_vals(), vec![&update]].into_iter().flatten())
                .collect();
            // TODO: Await all statements in parallel!
            pg.execute(&stmt, vals.as_slice()).await?;
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
    const COLS: &[&'static str] = &[
        "id",
        "name",
        "symbol",
        "slug",
        "is_active",
        "status",
        "first_historical_data",
        "last_historical_data",
        "last_update",
    ];
    fn sql_vals(&self) -> SqlVals {
        vec![
            &self.id,
            &self.name,
            &self.symbol,
            &self.slug,
            &self.is_active,
            &self.status,
            &self.first_historical_data,
            &self.last_historical_data,
        ]
    }

    fn upsert_into(n: usize) -> String {
        upsert_into("cryptocurrencies", Self::COLS, n, "id")
    }

    fn chunk_size() -> usize {
        MAX_PARAMS / Self::COLS.len()
    }
}

fn parse_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(u32::deserialize(deserializer)? != 0)
}
