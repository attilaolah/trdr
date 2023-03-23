use serde::Deserialize;
use tokio::join;
use tokio_postgres::Client;

use crate::cmc::enums::MetalUnit;
use crate::cmc::{Response, API};
use crate::error::Error;
use crate::sql::{upsert_into, SqlVals};

#[derive(Debug, Deserialize)]
struct FiatMetal {
    id: i32,
    name: String,
    symbol: String,       // fiats only
    sign: Option<String>, // fiats only
    code: Option<String>, // metals only
}

struct Fiat {
    id: i32,
    name: String,
    sign: String,
    symbol: String,
}

struct Metal {
    id: i32,
    name: String,
    code: String,
    unit: Option<MetalUnit>,
}

impl API {
    pub async fn update_fiats(&self, pg: &Client, metals: bool) -> Result<(), Error> {
        let req = self.fiat_map(metals)?;
        let url = req.url().as_str().to_string();
        let res: Response<Vec<FiatMetal>> = self.client.execute(req).await?.json().await?;
        res.status.check()?;

        let data = res
            .data
            .ok_or(Error::new("response contains no data".to_string()))?;

        let (fiats, metals): (Vec<FiatMetal>, Vec<FiatMetal>) =
            data.into_iter().partition(|fm| fm.is_fiat());
        let fiats: Vec<Fiat> = fiats.into_iter().map(|f| f.as_fiat()).flatten().collect();
        let metals: Vec<Metal> = metals.into_iter().map(|m| m.as_metal()).flatten().collect();

        let stmt_fiats_text = Fiat::upsert_into(fiats.len());
        let stmt_metals_text = Metal::upsert_into(metals.len());
        let (stmt_fiats, stmt_metals, update) = join!(
            pg.prepare(&stmt_fiats_text),
            pg.prepare(&stmt_metals_text),
            res.status.insert(&pg, &url),
        );
        let (stmt_fiats, stmt_metals, update) = (stmt_fiats?, stmt_metals?, update?);

        let fiats_vals: SqlVals = fiats
            .iter()
            .flat_map(|f| vec![f.sql_vals(), vec![&update]].into_iter().flatten())
            .collect();
        let metals_vals: SqlVals = metals
            .iter()
            .flat_map(|m| vec![m.sql_vals(), vec![&update]].into_iter().flatten())
            .collect();

        let (insert_fiats, insert_metals) = join!(
            pg.execute(&stmt_fiats, fiats_vals.as_slice()),
            pg.execute(&stmt_metals, metals_vals.as_slice()),
        );
        (insert_fiats?, insert_metals?);

        Ok(())
    }

    fn fiat_map(&self, metals: bool) -> Result<reqwest::Request, Error> {
        Ok(self
            .get("/fiat/map")
            .query(&[("include_metals", metals)])
            .build()?)
    }
}

impl FiatMetal {
    fn is_fiat(&self) -> bool {
        self.sign.is_some() && self.code.is_none()
    }
    fn as_fiat(self) -> Option<Fiat> {
        match self.sign {
            Some(sign) => Some(Fiat {
                id: self.id,
                name: self.name,
                sign: sign,
                symbol: self.symbol,
            }),
            None => None,
        }
    }

    fn as_metal(self) -> Option<Metal> {
        match self.code {
            Some(code) => {
                let unit = MetalUnit::from_suffix(&self.name);
                Some(Metal {
                    id: self.id,
                    name: match &unit {
                        Some(unit) => unit.trim_suffix(&self.name),
                        None => self.name,
                    },
                    code: code,
                    unit: unit,
                })
            }
            None => None,
        }
    }
}

impl Fiat {
    const COLS: &[&'static str] = &["id", "name", "sign", "symbol", "last_update"];

    fn sql_vals(&self) -> SqlVals {
        vec![&self.id, &self.name, &self.sign, &self.symbol]
    }

    fn upsert_into(n: usize) -> String {
        upsert_into("fiats", Self::COLS, n, Self::COLS[0])
    }
}

impl Metal {
    const COLS: &[&'static str] = &["id", "name", "code", "unit", "last_update"];

    fn sql_vals(&self) -> SqlVals {
        vec![&self.id, &self.name, &self.code, &self.unit]
    }

    fn upsert_into(n: usize) -> String {
        upsert_into("metals", Self::COLS, n, Self::COLS[0])
    }
}
