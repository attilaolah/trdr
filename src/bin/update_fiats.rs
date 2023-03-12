use clap::Parser;
use reqwest;
use tokio_postgres::NoTls;

use trdr::args::Args;
use trdr::cmc::fiat::Fiat;
use trdr::cmc::Response;
use trdr::error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    let cmc = args.cmc_api();
    let client = reqwest::Client::new();
    let (pg, conn) = tokio_postgres::connect(&args.db, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("PSQL CONN: {}", e);
        }
    });

    let req = cmc.fiat_map()?;
    let url = req.url().as_str().to_string();
    let res: Response<Vec<Fiat>> = client.execute(req).await?.json().await?;

    println!("{:#?}", &res.status);
    let update = res.status.insert(&pg, &url).await?;

    for fiat in res.data {
        fiat.insert(&pg, update).await?;
    }

    Ok(())
}
