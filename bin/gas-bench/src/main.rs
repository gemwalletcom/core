mod client;
mod etherscan;
mod gasflow;

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;

use crate::client::GemstoneClient;
use crate::etherscan::EtherscanClient;
use crate::gasflow::GasflowClient;
use gemstone::network::alien_provider::NativeProvider;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The number of blocks to fetch
    #[clap(short, long, default_value_t = 10)]
    blocks: u64,

    /// The reward percentiles to fetch
    #[clap(short, long, value_delimiter = ',', default_value = "25,50,75")]
    reward_percentiles: Vec<u64>,

    /// The Etherscan API key
    #[clap(long, env = "ETHERSCAN_API_KEY")]
    etherscan_api_key: String,
}

async fn run(args: Cli) -> Result<()> {
    let native_provider = Arc::new(NativeProvider::new());
    let gemstone_client = GemstoneClient::new(native_provider);
    // Prepare futures for concurrent execution
    let fee_history_future = gemstone_client.fetch_and_calculate_gemstone_fees(args.blocks, args.reward_percentiles.clone());

    let etherscan_future = async {
        let client = EtherscanClient::new(args.etherscan_api_key.clone());
        client.fetch_gas_oracle().await
    };

    let gasflow_future = async {
        let client = GasflowClient::new();
        client.fetch_prediction().await
    };

    // Execute calls in parallel
    let (fee_history_call_result, etherscan_call_result, gasflow_call_result) = tokio::join!(fee_history_future, etherscan_future, gasflow_future);

    // Process Gemstone fee history result
    match fee_history_call_result {
        Ok(gemstone_data) => {
            println!("Gemstone fee data: {:#?}", gemstone_data);
        }
        Err(e) => eprintln!("Error fetching or calculating Gemstone fee data: {:?}", e),
    }

    // Process Etherscan result
    match etherscan_call_result {
        Ok(etherscan_data) => {
            println!("Etherscan fee data: {:#?}", etherscan_data.result.fee_data());
        }
        Err(e) => eprintln!("\nError fetching Etherscan data: {:?}", e),
    }

    // Process Gasflow result
    match gasflow_call_result {
        Ok(gasflow_data) => {
            println!("Gasflow fee data: {:#?}", gasflow_data.fee_data());
        }
        Err(e) => eprintln!("\nError fetching Gasflow data: {:?}", e),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    if let Err(e) = run(args).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    Ok(())
}
