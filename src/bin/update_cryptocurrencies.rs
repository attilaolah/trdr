use clap::Parser;

use trdr::args::Args;
use trdr::error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    let db = args.db_connect().await?;
    let cmc = args.cmc_api();

    cmc.update_cryptocurrencies(&db).await
}
