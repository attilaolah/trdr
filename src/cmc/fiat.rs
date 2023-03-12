use serde::Deserialize;
use tokio_postgres::types::private::BytesMut;
use tokio_postgres::types::to_sql_checked;
use tokio_postgres::types::{IsNull, ToSql, Type};
use tokio_postgres::Client;

use crate::error::Error;

const INSERT_FIAT: &str = "INSERT INTO fiats (
    id,
    name,
    sign,
    symbol,
    last_update
) VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (id) DO UPDATE SET
    name = excluded.name,
    sign = excluded.sign,
    symbol = excluded.symbol,
    last_update = excluded.last_update
";

const INSERT_METAL: &str = "INSERT INTO metals (
    id,
    name,
    code,
    unit,
    last_update
) VALUES ($1, $2, $3, $4::metal_unit, $5)
ON CONFLICT (id) DO UPDATE SET
    name = excluded.name,
    code = excluded.code,
    unit = excluded.unit,
    last_update = excluded.last_update
";

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

impl Fiat {
    pub async fn insert(&self, pg: &Client, update: i32) -> Result<(), Error> {
        if let Some(code) = &self.code {
            // Metals have a code instead of a symbol.
            return self.insert_as_metal(pg, code, update).await;
        }
        pg.execute(
            INSERT_FIAT,
            &[&self.id, &self.name, &self.sign, &self.symbol, &update],
        )
        .await?;

        Ok(())
    }

    async fn insert_as_metal(&self, pg: &Client, code: &str, update: i32) -> Result<(), Error> {
        let unit = MetalUnit::from_suffix(&self.name);
        pg.execute(
            INSERT_METAL,
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
                if name[end..].eq_ignore_ascii_case(s) {
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
