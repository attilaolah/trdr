use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use tokio_postgres::{Client, Statement};

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
    pub async fn update_cryptocurrencies(&self, pg: &Client) -> Result<usize, Error> {
        let mut total = 0;
        loop {
            let fetched = update_cryptocurrencies(self, pg, total + 1).await?;
            total += fetched;
            if fetched == 0 {
                break;
            }
        }

        Ok(total)
    }

    fn cryptocurrency_map(&self, start: usize) -> Result<reqwest::Request, Error> {
        Ok(self
            .get("/cryptocurrency/map")
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
                ("start", start.to_string()),
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
        "platform",
        "platform_token",
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
        upsert_into("cryptocurrencies", Self::COLS, n, Self::COLS[0])
    }

    fn chunk_size() -> usize {
        MAX_PARAMS / Self::COLS.len()
    }
}

impl Platform {
    fn sql_vals(&self) -> SqlVals {
        vec![&self.id, &self.token_address]
    }
    fn sql_vals_or_none(p: &Option<Self>) -> SqlVals {
        match p {
            Some(p) => p.sql_vals(),
            None => vec![&None::<i32>, &None::<&str>],
        }
    }
}

fn parse_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(u32::deserialize(deserializer)? != 0)
}

async fn update_cryptocurrencies(api: &API, pg: &Client, page: usize) -> Result<usize, Error> {
    let req = api.cryptocurrency_map(page)?;
    let url = req.url().as_str().to_string();
    let res: Response<Vec<Cryptocurrency>> = api.client.execute(req).await?.json().await?;
    res.status.check()?;

    let mut stmt_size: usize = 0;
    let mut stmt: Option<Statement> = None;
    let update = res.status.insert(&pg, &url).await?;

    Ok(match res.data {
        Some(data) => {
            let mut total = 0;
            for chunk in data.chunks(Cryptocurrency::chunk_size()) {
                if chunk.len() != stmt_size || stmt.is_none() {
                    stmt_size = chunk.len();
                    let stmt_text = Cryptocurrency::upsert_into(stmt_size);
                    stmt = Some(pg.prepare(&stmt_text).await?);
                }
                let vals: SqlVals = chunk
                    .iter()
                    .flat_map(|c| {
                        vec![
                            c.sql_vals(),
                            Platform::sql_vals_or_none(&c.platform),
                            vec![&update],
                        ]
                        .into_iter()
                        .flatten()
                    })
                    .collect();
                pg.execute(stmt.as_ref().unwrap(), vals.as_slice()).await?;
                total += chunk.len();
            }
            total
        }
        None => 0,
    })
}
