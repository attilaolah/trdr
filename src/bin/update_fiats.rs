use clap::Parser;
use tokio_postgres::NoTls;

use trdr::args::Args;
use trdr::error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    let cmc = args.cmc_api();
    let (pg, conn) = tokio_postgres::connect(&args.db, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("PG ERR: {}", e);
        }
    });

    cmc.update_fiats(&pg, true).await
}
