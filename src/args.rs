use clap::Parser;

use crate::cmc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// CoinMarketCap API domain
    #[arg(long, default_value = "sandbox-api.coinmarketcap.com")]
    cmc_domain: String,

    /// CoinMarketCap API key
    #[arg(long, default_value = "b54bcf4d-1bca-4e8e-9a24-22ff2c3d462c")]
    cmc_api_key: String,

    /// PostgreSQL database path
    #[arg(long, default_value = "host=/var/run/postgresql dbname=markets")]
    pub db: String,
}

impl Args {
    pub fn cmc_api(&self) -> cmc::API {
        cmc::API::new(&self.cmc_domain, &self.cmc_api_key)
    }
}
