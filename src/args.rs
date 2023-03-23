use clap::Parser;
use std::env;
use tokio_postgres::{Client, NoTls};

use crate::cmc;
use crate::error::Error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// API endpoint (proxy)
    #[arg(long, default_value = "http://127.0.0.1:1100")]
    api: String,

    /// PostgreSQL database host or socket path
    #[arg(long, default_value = "/var/run/postgresql")]
    pub db_host: String,

    /// PostgreSQL database name
    #[arg(long, default_value = "markets")]
    pub db_name: String,

    /// PostgreSQL database user, or $USER if empty
    #[arg(long, default_value = "")]
    pub db_user: String,

    /// PostgreSQL database password, if needed
    #[arg(long, default_value = "")]
    pub db_pass: String,
}

impl Args {
    pub fn cmc_api(&self) -> cmc::API {
        cmc::API::new(&format!("{}/cmc", &self.api))
    }

    pub async fn db_connect(&self) -> Result<Client, Error> {
        let (pg, conn) = tokio_postgres::connect(&self.db_config(), NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("PG ERR: {}", e);
            }
        });

        Ok(pg)
    }

    fn db_config(&self) -> String {
        [
            format!("host={}", self.db_host),
            format!("dbname={}", self.db_name),
            format!(
                "user={}",
                if self.db_user == "" {
                    if let Ok(user) = env::var("USER").or_else(|_| env::var("USERNAME")) {
                        user
                    } else {
                        "".to_string()
                    }
                } else {
                    self.db_user.to_owned()
                }
            ),
            if self.db_pass != "" {
                format!("password={}", self.db_pass)
            } else {
                "".to_string()
            },
        ]
        .join(" ")
        .trim()
        .to_string()
    }
}
