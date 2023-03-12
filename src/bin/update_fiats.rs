use clap::Parser;
use reqwest;
use serde::Deserialize;

const API_HEADER: &str = "X-CMC_PRO_API_KEY";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// CoinMarketCap API domain
    #[arg(short, long, default_value = "sandbox-api.coinmarketcap.com")]
    api_domain: String,

    /// CoinMarketCap API key
    #[arg(
        short = 'k',
        long,
        default_value = "b54bcf4d-1bca-4e8e-9a24-22ff2c3d462c"
    )]
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct Status {
    error_code: i64,
    error_message: Option<String>,
    credit_count: i64,
    timestamp: String,
    elapsed: i64,
    notice: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Fiat {
    id: i64,
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

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = Args::parse();
    let client = reqwest::Client::new();

    let res: Response = client
        .get(format!("https://{}/v1/fiat/map", args.api_domain))
        .query(&[("include_metals", "true")])
        .header(API_HEADER, args.api_key)
        .send()
        .await?
        .json()
        .await?;

    println!("JSON: {:#?}", res);

    Ok(())
}
