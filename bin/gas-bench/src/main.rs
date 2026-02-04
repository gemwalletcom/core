mod client;
mod etherscan;
mod gasflow;
mod helius;
mod jito;
mod solana_client;

use clap::{Parser, ValueEnum};
use prettytable::{Cell, Row, Table, format};
use std::error::Error;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::time::interval;

use crate::jito::{format_micro_lamports, lamports_to_sol, priority_fee_to_lamports};
use crate::{
    client::{GemstoneClient, GemstoneFeeData},
    etherscan::EtherscanClient,
    gasflow::GasflowClient,
    helius::{HeliusClient, HeliusPriorityFees},
    jito::{JitoClient, JitoTipFloor},
    solana_client::{JUPITER_PROGRAM, SolanaFeeData, SolanaGasClient},
};
use gem_jsonrpc::native_provider::NativeProvider;
use gem_evm::ether_conv::EtherConv;
use primitives::fee::FeePriority;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ChainMode {
    Ethereum,
    Solana,
}

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
    long_about = "A CLI tool to benchmark gas/priority fees from multiple sources.\n\
It periodically fetches fee data and displays comparative tables.\n\n\
For Ethereum: fetches from Gemstone (local node), Etherscan API, and Gasflow API.\n\
For Solana: fetches priority fees via RPC and compares with Jito tip floor API."
)]
struct Cli {
    /// Chain to benchmark
    #[clap(short, long, value_enum, default_value_t = ChainMode::Ethereum)]
    chain: ChainMode,

    /// Enable debug logging
    #[arg(long, short, action = clap::ArgAction::SetTrue)]
    debug: bool,

    /// The number of blocks to fetch (Ethereum only)
    #[clap(short, long, default_value_t = 4)]
    blocks: u64,

    /// The reward percentiles to fetch (Ethereum only)
    #[clap(short, long, value_delimiter = ',', default_value = "20,40,60")]
    reward_percentiles: Vec<u64>,

    /// The minimum priority fee in wei (Ethereum only, default: 0.01 Gwei)
    #[clap(short, long, default_value_t = 10000000)]
    min_priority_fee: u64,

    /// The Etherscan API key (Ethereum only)
    #[clap(long, env = "ETHERSCAN_API_KEY")]
    etherscan_api_key: Option<String>,

    /// Compute units for priority fee calculation (Solana only)
    #[clap(long, default_value_t = 200_000)]
    compute_units: u64,

    /// Skip Jito API comparison (Solana only)
    #[clap(long, action = clap::ArgAction::SetTrue)]
    skip_jito: bool,

    /// Helius API key for getPriorityFeeEstimate (Solana only)
    #[clap(long, env = "HELIUS_API_KEY")]
    helius_api_key: Option<String>,
}

async fn run_ethereum(args: Cli) -> Result<(), Box<dyn Error + Send + Sync>> {
    let etherscan_api_key = args.etherscan_api_key.ok_or("Etherscan API key is required for Ethereum mode")?;

    let mut ticker = interval(Duration::from_secs(6));
    let native_provider = Arc::new(NativeProvider::new().set_debug(args.debug));

    let mut last_printed_block_opt: Option<u64> = None;
    let mut block_data: HashMap<u64, Vec<SourceFeeDetail>> = HashMap::new();
    println!(
        "gas-bench [Ethereum]: with history blocks: {}, reward percentiles: {:?}",
        args.blocks, args.reward_percentiles
    );

    loop {
        ticker.tick().await;
        if args.debug {
            eprintln!("gas-bench: fetching new gas fee data...");
        }

        let gemstone_client_clone = GemstoneClient::new(native_provider.clone());
        let reward_percentiles_clone = args.reward_percentiles.clone();
        let etherscan_api_key_clone = etherscan_api_key.clone();

        let fee_history_future = gemstone_client_clone.fetch_base_priority_fees(args.blocks, reward_percentiles_clone, args.min_priority_fee);

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
                    FeePriority::Slow => slow = EtherConv::to_gwei(&fee_record.value),
                    FeePriority::Normal => normal = EtherConv::to_gwei(&fee_record.value),
                    FeePriority::Fast => fast = EtherConv::to_gwei(&fee_record.value),
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
        } else if let Err(e) = gemstone_res
            && args.debug
        {
            eprintln!("gas-bench: Error fetching Gemstone data: {e:?}");
        }

        if let Ok(data) = etherscan_res {
            let fee_data = data.result.fee_data();
            let entry = block_data.entry(fee_data.latest_block).or_default();
            if !entry.iter().any(|d| d.source_name == "Etherscan") {
                entry.push(process_fee_data("Etherscan", &fee_data));
            }
        } else if let Err(e) = etherscan_res
            && args.debug
        {
            eprintln!("Error fetching Etherscan data: {e:?}");
        }

        if let Ok(data) = gasflow_res {
            let fee_data = data.fee_data();
            let entry = block_data.entry(fee_data.latest_block).or_default();
            if !entry.iter().any(|d| d.source_name == "Gasflow") {
                entry.push(process_fee_data("Gasflow", &fee_data));
            }
        } else if let Err(e) = gasflow_res
            && args.debug
        {
            eprintln!("Error fetching Gasflow data: {e:?}");
        }

        if args.debug {
            eprintln!("Debug: Aggregated block_data summary:");
            let mut sorted_debug_keys: Vec<_> = block_data.keys().collect();
            sorted_debug_keys.sort();
            for block_num in sorted_debug_keys {
                if let Some(details) = block_data.get(block_num) {
                    let sources: Vec<&str> = details.iter().map(|d| d.source_name.as_str()).collect();
                    eprintln!("  Block {block_num}: {sources:?}");
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
            eprintln!("Debug: last_printed_block_opt: {last_printed_block_opt:?}");
            eprintln!("Debug: block_to_print_this_iteration: {block_to_print_this_iteration:?}");
        }

        if let Some(current_block_to_print) = block_to_print_this_iteration {
            if args.debug {
                eprintln!("Debug: Attempting to print table for block: {current_block_to_print}");
            }
            if let Some(details_for_block) = block_data.get(&current_block_to_print)
                && details_for_block.len() >= 2
            {
                println!("\n--- Block: {current_block_to_print} ---");
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

async fn run_solana(args: Cli) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut ticker = interval(Duration::from_secs(6));
    let native_provider = Arc::new(NativeProvider::new().set_debug(args.debug));
    let solana_client = SolanaGasClient::new(native_provider);
    let jito_client = if args.skip_jito { None } else { Some(JitoClient::new()) };
    let helius_client = args.helius_api_key.as_ref().map(|key| HeliusClient::new(key));

    let mut last_printed_slot: Option<u64> = None;

    println!("gas-bench [Solana]: monitoring priority fees and Jito tips");
    println!("  Compute units for fee calculation: {}", args.compute_units);
    println!("  Jito comparison: {}", if jito_client.is_some() { "enabled" } else { "disabled" });
    println!("  Helius comparison: {}", if helius_client.is_some() { "enabled" } else { "disabled (set HELIUS_API_KEY)" });
    println!();

    loop {
        ticker.tick().await;

        if args.debug {
            eprintln!("gas-bench: fetching Solana fee data...");
        }

        let solana_future = solana_client.fetch_fee_data();
        let jito_future = async {
            match &jito_client {
                Some(client) => Some(client.fetch_tip_floor().await),
                None => None,
            }
        };
        let helius_future = async {
            match &helius_client {
                Some(client) => Some(client.fetch_priority_fee_estimate(Some(vec![JUPITER_PROGRAM.to_string()])).await),
                None => None,
            }
        };

        let (solana_res, jito_res, helius_res) = tokio::join!(solana_future, jito_future, helius_future);

        match solana_res {
            Ok(fee_data) => {
                if last_printed_slot.is_some_and(|s| s >= fee_data.slot) {
                    continue;
                }

                print_solana_fee_data(&fee_data, &jito_res, &helius_res, args.compute_units);
                last_printed_slot = Some(fee_data.slot);
            }
            Err(e) => {
                if args.debug {
                    eprintln!("gas-bench: Error fetching Solana data: {e:?}");
                }
            }
        }
    }
}

fn print_solana_fee_data(
    fee_data: &SolanaFeeData,
    jito_res: &Option<Result<JitoTipFloor, Box<dyn Error + Send + Sync>>>,
    helius_res: &Option<Result<HeliusPriorityFees, Box<dyn Error + Send + Sync>>>,
    compute_units: u64,
) {
    println!("\n--- Slot: {} ---", fee_data.slot);

    let accounts = [
        ("Jupiter", &fee_data.account_fees.jupiter),
        ("Orca", &fee_data.account_fees.orca),
        ("USDC", &fee_data.account_fees.usdc),
    ];
    let active_accounts: Vec<&str> = accounts
        .iter()
        .filter(|(_, data)| data.as_ref().is_some_and(|d| d.count > 0))
        .map(|(name, _)| *name)
        .collect();

    let jito_available = jito_res.as_ref().is_some_and(|r| r.is_ok());
    let helius_data = helius_res.as_ref().and_then(|r| r.as_ref().ok());

    if !active_accounts.is_empty() {
        print!("Sampling: {} (avg: {} µL/CU)", active_accounts.join(", "), fee_data.raw_fees.avg);
    }
    if let Some(helius) = helius_data {
        print!(
            " | Helius: slow={} normal={} fast={}",
            format_micro_lamports(helius.low),
            format_micro_lamports(helius.medium),
            format_micro_lamports(helius.high)
        );
    }
    println!();

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    let mut header = vec![Cell::new("Level"), Cell::new("Priority (70%)"), Cell::new("Jito Tip (30%)"), Cell::new("Total")];
    if jito_available {
        header.push(Cell::new("Jito Floor"));
    }
    table.add_row(Row::new(header));

    let levels = [
        ("Slow", fee_data.priority_fees.slow, fee_data.jito_tips.slow),
        ("Normal", fee_data.priority_fees.normal, fee_data.jito_tips.normal),
        ("Fast", fee_data.priority_fees.fast, fee_data.jito_tips.fast),
    ];

    let jito_data = jito_res.as_ref().and_then(|r| r.as_ref().ok());

    for (level, priority_fee, jito_tip) in levels.iter() {
        let priority_lamports = priority_fee_to_lamports(*priority_fee, compute_units);
        let total_lamports = priority_lamports + jito_tip;
        let priority_display = format!("{} ({})", format_micro_lamports(*priority_fee), lamports_to_sol(priority_lamports));
        let jito_tip_display = lamports_to_sol(*jito_tip);
        let total_display = lamports_to_sol(total_lamports);

        let mut row = vec![Cell::new(level), Cell::new(&priority_display), Cell::new(&jito_tip_display), Cell::new(&total_display)];

        if let Some(jito) = jito_data {
            let jito_floor = match *level {
                "Slow" => jito.p25_lamports,
                "Normal" => jito.p50_lamports,
                "Fast" => jito.p75_lamports,
                _ => 0,
            };
            let jito_floor_display = lamports_to_sol(jito_floor);
            row.push(Cell::new(&jito_floor_display));
        }

        table.add_row(Row::new(row));
    }

    table.printstd();

    if let Some(Err(e)) = jito_res {
        println!("  (Jito API error: {})", e);
    }
    if let Some(Err(e)) = helius_res {
        println!("  (Helius API error: {})", e);
    }
}

async fn run(args: Cli) -> Result<(), Box<dyn Error + Send + Sync>> {
    match args.chain {
        ChainMode::Ethereum => run_ethereum(args).await,
        ChainMode::Solana => run_solana(args).await,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = Cli::parse();
    if args.debug {
        eprintln!("gas-bench: debug mode enabled by CLI flag.");
    }

    if let Err(e) = run(args).await {
        eprintln!("gas-bench: run error: {e}");
        std::process::exit(1);
    }

    Ok(())
}
