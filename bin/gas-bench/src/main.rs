mod client;
mod etherscan;
mod gasflow;

use anyhow::Result;
use clap::Parser;
use prettytable::{format, Cell, Row, Table};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::time::interval;

use crate::{
    client::{GemstoneClient, GemstoneFeeData},
    etherscan::EtherscanClient,
    gasflow::GasflowClient,
};
use gemstone::{ethereum::model::GemFeePriority, network::alien_provider::NativeProvider};

#[derive(Debug, Clone)]
struct SourceFeeDetail {
    source_name: String,
    base_fee: String,
    gas_used_ratio: Option<String>,
    slow_fee: String,
    normal_fee: String,
    fast_fee: String,
}

#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about,
    long_about = "A CLI tool to benchmark Ethereum gas fees from multiple sources.\n\
It periodically fetches gas fee data from Gemstone (local Ethereum node), Etherscan API, and Gasflow API.\n\
The data is then grouped by block number and displayed in a comparative table, showing slow, normal, and fast priority fees along with suggested base fees from each source.\n\
This allows for easy visual comparison and benchmarking of gas fee estimations."
)]
struct Cli {
    /// Enable debug logging
    #[arg(long, short, action = clap::ArgAction::SetTrue)]
    debug: bool,

    /// The number of blocks to fetch
    #[clap(short, long, default_value_t = 6)]
    blocks: u64,

    /// The reward percentiles to fetch
    #[clap(short, long, value_delimiter = ',', default_value = "20,50,70")]
    reward_percentiles: Vec<u64>,

    /// The Etherscan API key
    #[clap(long, env = "ETHERSCAN_API_KEY")]
    etherscan_api_key: String,
}

async fn run(args: Cli) -> Result<()> {
    let mut ticker = interval(Duration::from_secs(6));
    let native_provider = Arc::new(NativeProvider::new().set_debug(args.debug));

    let mut last_printed_block_opt: Option<u64> = None;
    let mut block_data: HashMap<u64, Vec<SourceFeeDetail>> = HashMap::new();
    println!(
        "gas-bench: with history blocks: {}, reward percentiles: {:?}",
        args.blocks, args.reward_percentiles
    );

    loop {
        ticker.tick().await;
        println!("gas-bench: fetching new gas fee data...");

        let gemstone_client_clone = GemstoneClient::new(native_provider.clone());
        let reward_percentiles_clone = args.reward_percentiles.clone();
        let etherscan_api_key_clone = args.etherscan_api_key.clone();

        let fee_history_future = gemstone_client_clone.fetch_and_calculate_gemstone_fees(args.blocks, reward_percentiles_clone);

        let etherscan_future = async move {
            let client = EtherscanClient::new(etherscan_api_key_clone);
            client.fetch_gas_oracle().await
        };

        let gasflow_future = async {
            let client = GasflowClient::new();
            client.fetch_prediction().await
        };

        let (gemstone_res, etherscan_res, gasflow_res) = tokio::join!(fee_history_future, etherscan_future, gasflow_future);

        if args.debug {
            eprintln!("gas-bench: processing new fetch cycle, block_data currently has {} entries.", block_data.len());
        }

        let process_fee_data = |source_name: &str, data: &GemstoneFeeData| -> SourceFeeDetail {
            let mut slow = "N/A".to_string();
            let mut normal = "N/A".to_string();
            let mut fast = "N/A".to_string();
            for fee_record in &data.priority_fees {
                match fee_record.priority {
                    GemFeePriority::Slow => slow = fee_record.value.clone(),
                    GemFeePriority::Normal => normal = fee_record.value.clone(),
                    GemFeePriority::Fast => fast = fee_record.value.clone(),
                }
            }
            SourceFeeDetail {
                source_name: source_name.to_string(),
                base_fee: data.suggest_base_fee.clone(),
                gas_used_ratio: data.gas_used_ratio.clone(),
                slow_fee: slow,
                normal_fee: normal,
                fast_fee: fast,
            }
        };

        if let Ok(data) = gemstone_res {
            let entry = block_data.entry(data.latest_block).or_default();
            if !entry.iter().any(|d| d.source_name == "Gemstone") {
                entry.push(process_fee_data("Gemstone", &data));
            }
        } else if let Err(e) = gemstone_res {
            if args.debug {
                eprintln!("gas-bench: Error fetching Gemstone data: {:?}", e);
            }
        }

        if let Ok(data) = etherscan_res {
            let fee_data = data.result.fee_data();
            let entry = block_data.entry(fee_data.latest_block).or_default();
            if !entry.iter().any(|d| d.source_name == "Etherscan") {
                entry.push(process_fee_data("Etherscan", &fee_data));
            }
        } else if let Err(e) = etherscan_res {
            if args.debug {
                eprintln!("Error fetching Etherscan data: {:?}", e);
            }
        }

        if let Ok(data) = gasflow_res {
            let fee_data = data.fee_data();
            let entry = block_data.entry(fee_data.latest_block).or_default();
            if !entry.iter().any(|d| d.source_name == "Gasflow") {
                entry.push(process_fee_data("Gasflow", &fee_data));
            }
        } else if let Err(e) = gasflow_res {
            if args.debug {
                eprintln!("Error fetching Gasflow data: {:?}", e);
            }
        }

        if args.debug {
            eprintln!("Debug: Aggregated block_data summary:");
            let mut sorted_debug_keys: Vec<_> = block_data.keys().collect();
            sorted_debug_keys.sort();
            for block_num in sorted_debug_keys {
                if let Some(details) = block_data.get(block_num) {
                    let sources: Vec<&str> = details.iter().map(|d| d.source_name.as_str()).collect();
                    eprintln!("  Block {}: {:?}", block_num, sources);
                }
            }
        }

        let mut sorted_blocks_in_map: Vec<u64> = block_data.keys().cloned().collect();
        sorted_blocks_in_map.sort_unstable();

        let block_to_print_this_iteration: Option<u64> = match last_printed_block_opt {
            Some(last_printed) => sorted_blocks_in_map
                .into_iter()
                .find(|&block_num| block_num > last_printed && block_data.get(&block_num).is_some_and(|details| details.len() >= 2)),
            None => sorted_blocks_in_map
                .iter()
                .find(|&&b_num| block_data.get(&b_num).is_some_and(|details| details.len() >= 2))
                .cloned(),
        };

        if args.debug {
            eprintln!("Debug: last_printed_block_opt: {:?}", last_printed_block_opt);
            eprintln!("Debug: block_to_print_this_iteration: {:?}", block_to_print_this_iteration);
        }

        if let Some(current_block_to_print) = block_to_print_this_iteration {
            if args.debug {
                eprintln!("Debug: Attempting to print table for block: {}", current_block_to_print);
            }
            if let Some(details_for_block) = block_data.get(&current_block_to_print) {
                if details_for_block.len() >= 2 {
                    println!("\n--- Block: {} ---", current_block_to_print);
                    let mut table = Table::new();
                    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
                    table.add_row(Row::new(vec![
                        Cell::new("Source"),
                        Cell::new("Base Fee (Gwei)"),
                        Cell::new("Used Gas (%)"),
                        Cell::new("Slow (Gwei)"),
                        Cell::new("Normal (Gwei)"),
                        Cell::new("Fast (Gwei)"),
                    ]));

                    for detail in details_for_block {
                        table.add_row(Row::new(vec![
                            Cell::new(&detail.source_name),
                            Cell::new(&detail.base_fee),
                            Cell::new(&detail.gas_used_ratio.clone().unwrap_or_else(|| "N/A".to_string())),
                            Cell::new(&detail.slow_fee),
                            Cell::new(&detail.normal_fee),
                            Cell::new(&detail.fast_fee),
                        ]));
                    }
                    table.printstd();
                    last_printed_block_opt = Some(current_block_to_print);
                }
            }
        }
    }
    // Ok(()) // Loop is infinite, this is not reached
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    if args.debug {
        eprintln!("gas-bench: debug mode enabled by CLI flag.");
    }

    if let Err(e) = run(args).await {
        eprintln!("gas-bench: run error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
