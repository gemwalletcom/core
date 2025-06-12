mod etherscan;
mod gasflow;

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;

use gem_evm::jsonrpc::EthereumRpc;
use gemstone::{
    ethereum::model::GemEthereumFeeHistory,
    network::{alien_provider::NativeProvider, JsonRpcClient, JsonRpcResult},
};
use primitives::Chain;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The number of blocks to fetch
    #[clap(short, long, default_value_t = 10)]
    blocks: u64,
}

async fn run(args: Cli) -> Result<()> {
    let native_provider = Arc::new(NativeProvider::new());
    let client = JsonRpcClient::new_with_chain(native_provider, Chain::Ethereum);
    let call = EthereumRpc::FeeHistory {
        blocks: args.blocks,
        reward_percentiles: vec![25, 50, 75],
    };

    let resp: JsonRpcResult<GemEthereumFeeHistory> = client.call(&call).await?;
    let fee_history_data = resp.take()?;

    println!("fee_history_data: {:#?}", fee_history_data);
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
