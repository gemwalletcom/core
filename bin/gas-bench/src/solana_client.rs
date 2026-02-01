use std::error::Error;
use std::sync::Arc;

use gem_jsonrpc::client::JsonRpcClient;
use gem_solana::models::jito::{FeeStats, JitoTipEstimates, calculate_fee_stats, estimate_jito_tips};
use gem_solana::models::prioritization_fee::SolanaPrioritizationFee;
use gemstone::alien::{AlienProvider, new_alien_client, reqwest_provider::NativeProvider};
use primitives::Chain;
use serde_json::json;

pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const JUPITER_PROGRAM: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
pub const ORCA_WHIRLPOOL: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

const MIN_SLOW_FEE: u64 = 1_000;
const MIN_NORMAL_FEE: u64 = 10_000;
const MIN_FAST_FEE: u64 = 100_000;

#[derive(Debug)]
pub struct SolanaFeeData {
    pub slot: u64,
    pub priority_fees: PriorityFees,
    pub jito_tips: JitoTipEstimates,
    pub raw_fees: FeeStats,
    pub account_fees: AccountFeeStats,
}

#[derive(Debug)]
pub struct PriorityFees {
    pub slow: u64,
    pub normal: u64,
    pub fast: u64,
}

#[derive(Debug, Default)]
pub struct AccountFeeStats {
    pub jupiter: Option<FeeStats>,
    pub orca: Option<FeeStats>,
    pub usdc: Option<FeeStats>,
}

pub struct SolanaGasClient {
    native_provider: Arc<NativeProvider>,
}

impl SolanaGasClient {
    pub fn new(native_provider: Arc<NativeProvider>) -> Self {
        Self { native_provider }
    }

    pub async fn fetch_fee_data(&self) -> Result<SolanaFeeData, Box<dyn Error + Send + Sync>> {
        let endpoint = self.native_provider.get_endpoint(Chain::Solana)?;
        let alien_client = new_alien_client(endpoint, self.native_provider.clone());
        let client: JsonRpcClient<_> = JsonRpcClient::new(alien_client);

        let slot: u64 = client.call("getSlot", json!([])).await?;

        let global_fees: Vec<SolanaPrioritizationFee> = client.call("getRecentPrioritizationFees", json!([])).await?;

        let mut account_fees = AccountFeeStats::default();

        for (account, name) in [(JUPITER_PROGRAM, "jupiter"), (ORCA_WHIRLPOOL, "orca"), (USDC_MINT, "usdc")] {
            let fees: Vec<SolanaPrioritizationFee> = client.call("getRecentPrioritizationFees", json!([[account]])).await?;

            if !fees.is_empty() {
                let values: Vec<i64> = fees.iter().map(|f| f.prioritization_fee).collect();
                let stats = calculate_fee_stats(&values);
                match name {
                    "jupiter" => account_fees.jupiter = Some(stats),
                    "orca" => account_fees.orca = Some(stats),
                    "usdc" => account_fees.usdc = Some(stats),
                    _ => {}
                }
            }
        }

        let global_values: Vec<i64> = global_fees.iter().map(|f| f.prioritization_fee).collect();
        let raw_fees = calculate_fee_stats(&global_values);

        let effective_stats = get_best_fee_stats(&raw_fees, &account_fees);
        let priority_fees = calculate_priority_fees(&effective_stats);
        let jito_tips = estimate_jito_tips(&effective_stats);

        Ok(SolanaFeeData {
            slot,
            priority_fees,
            jito_tips,
            raw_fees,
            account_fees,
        })
    }
}

fn get_best_fee_stats(global: &FeeStats, accounts: &AccountFeeStats) -> FeeStats {
    accounts
        .jupiter
        .as_ref()
        .filter(|a| a.count > 0 && a.avg > 0)
        .or_else(|| accounts.orca.as_ref().filter(|a| a.count > 0 && a.avg > 0))
        .or_else(|| accounts.usdc.as_ref().filter(|a| a.count > 0 && a.avg > 0))
        .cloned()
        .unwrap_or_else(|| global.clone())
}

fn calculate_priority_fees(stats: &FeeStats) -> PriorityFees {
    PriorityFees {
        slow: (stats.median as u64).max(MIN_SLOW_FEE),
        normal: (stats.p75 as u64).max(MIN_NORMAL_FEE),
        fast: (stats.p90 as u64).max(MIN_FAST_FEE),
    }
}
