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
        // First download all cryptocurrencies, page-by-page.
        // For each object, store the corresponding "update" value.
        let mut data: Vec<(Cryptocurrency, i32)> = vec![];

        loop {
            let req = self.cryptocurrency_map(data.len() + 1)?;
            let url = req.url().as_str().to_string();
            let res: Response<Vec<Cryptocurrency>> = self.client.execute(req).await?.json().await?;
            res.status.check()?;

            if res.data.as_ref().map_or(true, |v| v.is_empty()) {
                break;
            }

            match res.data {
                Some(page) => {
                    if page.is_empty() {
                        break;
                    }
                    let update = res.status.insert(&pg, &url).await?;
                    data.extend(page.into_iter().map(|c| (c, update)));
                }
                None => break,
            }
        }
        let data = data; // freeze
        let total = data.len();

        // Then insert them in chunks.
        let mut stmt_size: usize = 0;
        let mut stmt: Option<Statement> = None;

        for chunk in data.chunks(Cryptocurrency::chunk_size()) {
            if chunk.len() != stmt_size || stmt.is_none() {
                stmt_size = chunk.len();
                let stmt_text = Cryptocurrency::upsert_into(stmt_size);
                stmt = Some(pg.prepare(&stmt_text).await?);
            }
            let vals: SqlVals = chunk
                .iter()
                .flat_map(|(c, u)| vec![c.sql_vals(), vec![u]].into_iter().flatten())
                .collect();
            pg.execute(stmt.as_ref().unwrap(), vals.as_slice()).await?;
        }

        // Finally, update any platform references in a second run.
        let platforms: Vec<_> = data
            .into_iter()
            .map(|(c, _)| c)
            .filter_map(|c| c.platform.map(|p| (p, c.id)))
            .collect();
        stmt_size = 0;
        stmt = None;

        for chunk in platforms.chunks(Platform::chunk_size()) {
            if chunk.len() != stmt_size || stmt.is_none() {
                stmt_size = chunk.len();
                let stmt_text = Platform::upsert_into(stmt_size);
                stmt = Some(pg.prepare(&stmt_text).await?);
            }
            let vals: SqlVals = chunk.iter().flat_map(|(p, id)| p.sql_vals(id)).collect();
            pg.execute(stmt.as_ref().unwrap(), vals.as_slice()).await?;
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
    const TBL: &str = "cryptocurrencies";
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
        upsert_into(Self::TBL, Self::COLS, n, Self::COLS[0])
    }

    fn chunk_size() -> usize {
        MAX_PARAMS / Self::COLS.len()
    }
}

impl Platform {
    const TBL: &str = "cryptocurrency_platforms";
    const COLS: &[&'static str] = &["id", "platform", "token_address"];

    fn sql_vals<'a>(&'a self, id: &'a i32) -> SqlVals {
        vec![id, &self.id, &self.token_address]
    }

    fn upsert_into(n: usize) -> String {
        upsert_into(Self::TBL, Self::COLS, n, Self::COLS[0])
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
