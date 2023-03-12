use serde::Deserialize;
use tokio_postgres::types::private::BytesMut;
use tokio_postgres::types::to_sql_checked;
use tokio_postgres::types::{IsNull, ToSql, Type};
use tokio_postgres::Client;

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

#[derive(Debug)]
struct MetalUnit(Option<String>);

impl API {
    pub async fn update_fiats(&self, pg: &Client, metals: bool) -> Result<(), Error> {
        let req = self.fiat_map(metals)?;
        let url = req.url().as_str().to_string();
        let res: Response<Vec<Fiat>> = self.client.execute(req).await?.json().await?;

        let update = res.status.insert(&pg, &url).await?;
        for fiat in res.data {
            fiat.insert(&pg, update).await?;
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
    pub async fn insert(&self, pg: &Client, update: i32) -> Result<(), Error> {
        if let Some(code) = &self.code {
            // Metals have a code instead of a symbol.
            return self.insert_as_metal(pg, code, update).await;
        }
        pg.execute(
            include_str!("sql/fiats_insert.sql"),
            &[&self.id, &self.name, &self.sign, &self.symbol, &update],
        )
        .await?;

        Ok(())
    }

    async fn insert_as_metal(&self, pg: &Client, code: &str, update: i32) -> Result<(), Error> {
        let unit = MetalUnit::from_suffix(&self.name);
        pg.execute(
            include_str!("sql/metals_insert.sql"),
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

impl MetalUnit {
    fn from_suffix(name: &str) -> Self {
        let lname = name.to_lowercase();
        if lname.ends_with("troy ounce") {
            MetalUnit(Some(String::from("troy_ounce")))
        } else if lname.ends_with("ounce") {
            MetalUnit(Some(String::from("ounce")))
        } else {
            MetalUnit(None)
        }
    }

    fn trim_suffix<'a>(&'a self, name: &'a str) -> String {
        if let Some(s) = &self.0 {
            let len = s.len();
            if name.len() >= len {
                let end = name.len() - len;
                if name[end..].replace(" ", "_").eq_ignore_ascii_case(s) {
                    return name[..end].trim().to_string();
                }
            }
        }

        name.to_string()
    }
}

impl ToSql for MetalUnit {
    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        if let Some(s) = &self.0 {
            out.extend(s.bytes());
            Ok(IsNull::No)
        } else {
            Ok(IsNull::Yes)
        }
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "metal_unit"
    }

    to_sql_checked!();
}
