use chrono::{DateTime, Utc};
use clap::Parser;
use reqwest;
use serde::Deserialize;
use std::time::Duration;
use tokio_postgres::{Client, NoTls};

const API_HEADER: &str = "X-CMC_PRO_API_KEY";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// CoinMarketCap API domain
    #[arg(long, default_value = "sandbox-api.coinmarketcap.com")]
    api_domain: String,

    /// CoinMarketCap API key
    #[arg(long, default_value = "b54bcf4d-1bca-4e8e-9a24-22ff2c3d462c")]
    api_key: String,

    /// PostgreSQL database path
    #[arg(long, default_value = "host=/var/run/postgresql dbname=markets")]
    db: String,
}

#[derive(Debug, Deserialize)]
struct Status {
    error_code: i32,
    error_message: Option<String>,
    credit_count: i32,
    timestamp: DateTime<Utc>,
    #[serde(rename = "elapsed")]
    elapsed_ms: u64,
    notice: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Fiat {
    id: i32,
    name: String,
    sign: Option<String>,

    symbol: String,
    code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Response {
    status: Status,
    data: Vec<Fiat>,
}

const INSERT_UPDATE: &str = "INSERT INTO updates (
    url,
    error_code,
    error_message,
    credit_count,
    timestamp,
    elapsed,
    notice
) VALUES ($1, $2, $3, $4, $5, make_interval(secs => $6), $7)
RETURNING id";

impl Status {
    async fn insert(&self, pg: &Client, url: &str) -> Result<i32, Error> {
        Ok(pg
            .query_one(
                INSERT_UPDATE,
                &[
                    &url,
                    &self.error_code,
                    &self.error_message,
                    &self.credit_count,
                    &self.timestamp.naive_utc(),
                    &self.elapsed().as_secs_f64(),
                    &self.notice,
                ],
            )
            .await?
            .get(0))
    }

    fn elapsed(&self) -> Duration {
        Duration::from_millis(self.elapsed_ms)
    }
}

#[derive(Debug)]
enum Error {
    RqError(reqwest::Error),
    PgError(tokio_postgres::Error),
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::RqError(error)
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(error: tokio_postgres::Error) -> Self {
        Error::PgError(error)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let client = reqwest::Client::new();
    let (pg, conn) = tokio_postgres::connect(&args.db, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("PSQL CONN: {}", e);
        }
    });

    let endpoint = "/v1/fiat/map";
    let query_params = [("include_metals", "true")];

    let req = client
        .get(format!("https://{}{}", args.api_domain, endpoint))
        .header(API_HEADER, args.api_key)
        .query(&query_params)
        .build()?;
    let url = req.url().as_str().to_string();
    let res: Response = client.execute(req).await?.json().await?;

    println!("STATUS: {:#?}", &res.status);
    println!("UPDATE: {}", res.status.insert(&pg, &url).await?);

    Ok(())
}
