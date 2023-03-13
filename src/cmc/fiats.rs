use serde::Deserialize;
use tokio::join;
use tokio_postgres::{Client, Statement};

use crate::cmc::enums::MetalUnit;
use crate::cmc::{Response, API};
use crate::error::Error;

#[derive(Debug, Deserialize)]
pub struct Fiat {
    id: i32,
    name: String,
    sign: Option<String>,

    symbol: String,
    code: Option<String>,
}

impl API {
    pub async fn update_fiats(&self, pg: &Client, metals: bool) -> Result<(), Error> {
        let req = self.fiat_map(metals)?;
        let url = req.url().as_str().to_string();
        let res: Response<Vec<Fiat>> = self.client.execute(req).await?.json().await?;
        res.status.check()?;

        let (stmt, stmt_metal, update) = join!(
            pg.prepare(include_str!("sql/fiats_insert.sql")),
            pg.prepare(include_str!("sql/metals_insert.sql")),
            res.status.insert(&pg, &url),
        );
        let (stmt, stmt_metal, update) = (stmt?, stmt_metal?, update?);

        // TODO: Insert all in a single statement.
        for fiat in res.data.unwrap_or(vec![]) {
            fiat.insert(&pg, &stmt, &stmt_metal, update).await?;
        }

        Ok(())
    }

    fn fiat_map(&self, metals: bool) -> Result<reqwest::Request, Error> {
        Ok(self
            .get("/v1/fiat/map")
            .query(&[("include_metals", metals)])
            .build()?)
    }
}

impl Fiat {
    pub async fn insert(
        &self,
        pg: &Client,
        stmt: &Statement,
        stmt_metal: &Statement,
        update: i32,
    ) -> Result<(), Error> {
        if let Some(code) = &self.code {
            // Metals have a code instead of a symbol.
            return self.insert_as_metal(pg, stmt_metal, code, update).await;
        }
        pg.execute(
            stmt,
            &[&self.id, &self.name, &self.sign, &self.symbol, &update],
        )
        .await?;

        Ok(())
    }

    async fn insert_as_metal(
        &self,
        pg: &Client,
        stmt: &Statement,
        code: &str,
        update: i32,
    ) -> Result<(), Error> {
        let unit = MetalUnit::from_suffix(&self.name).ok_or_else(|| {
            Error::new(format!("unknown unit for: {}", &self.name.to_lowercase()))
        })?;
        pg.execute(
            stmt,
            &[
                &self.id,
                &unit.trim_suffix(&self.name),
                &code,
                &unit,
                &update,
            ],
        )
        .await?;

        Ok(())
    }
}
