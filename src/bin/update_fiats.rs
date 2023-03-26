use clap::Parser;

use trdr::args::Args;
use trdr::error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    let db = args.db_connect().await?;
    let cmc = args.cmc_api();

    let (fiats, metals) = cmc.update_fiats(&db, true).await?;
    println!("INFO: Imported {} fiats, {} metals.", fiats, metals);

    Ok(())
}
