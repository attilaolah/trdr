use clap::Parser;
use reqwest;




use tokio_postgres::{NoTls};

use trdr::cmc::fiat::Fiat;
use trdr::cmc::{Response, API};
use trdr::error::Error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// CoinMarketCap API domain
    #[arg(long, default_value = "sandbox-api.coinmarketcap.com")]
    cmc_domain: String,

    /// CoinMarketCap API key
    #[arg(long, default_value = "b54bcf4d-1bca-4e8e-9a24-22ff2c3d462c")]
    cmc_api_key: String,

    /// PostgreSQL database path
    #[arg(long, default_value = "host=/var/run/postgresql dbname=markets")]
    db: String,
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

    let api = API::new(&args.cmc_domain, &args.cmc_api_key);

    let req = api.get(endpoint).query(&query_params).build()?;
    let url = req.url().as_str().to_string();
    let res: Response<Vec<Fiat>> = client.execute(req).await?.json().await?;

    println!("STATUS: {:#?}", &res.status);
    let update = res.status.insert(&pg, &url).await?;

    for fiat in res.data {
        fiat.insert(&pg, update).await?;
    }

    Ok(())
}
