use tokio_postgres::types::private::BytesMut;
use tokio_postgres::types::to_sql_checked;
use tokio_postgres::types::{IsNull, ToSql, Type};

#[derive(Debug)]
pub struct TrackingStatus(String);

#[derive(Debug)]
pub struct MetalUnit(String);

impl TrackingStatus {
    pub fn new(value: &str) -> Self {
        Self(value.to_lowercase())
    }
}

impl ToSql for TrackingStatus {
    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        out.extend(self.0.bytes());
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "tracking_status"
    }

    to_sql_checked!();
}

impl MetalUnit {
    pub fn from_suffix(name: &str) -> Option<Self> {
        let lname = name.to_lowercase();
        if lname.ends_with("troy ounce") {
            Some(MetalUnit("troy_ounce".to_string()))
        } else if lname.ends_with("ounce") {
            Some(MetalUnit("ounce".to_string()))
        } else {
            None
        }
    }

    pub fn trim_suffix<'a>(&'a self, name: &'a str) -> String {
        let len = self.0.len();
        if name.len() >= len {
            let end = name.len() - len;
            if name[end..].replace(" ", "_").eq_ignore_ascii_case(&self.0) {
                return name[..end].trim().to_string();
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
        out.extend(self.0.bytes());
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "metal_unit"
    }

    to_sql_checked!();
}
