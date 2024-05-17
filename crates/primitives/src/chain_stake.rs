use serde::{Deserialize, Serialize};
use strum::EnumIter;
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum StakeChain {
    Cosmos,
    Osmosis,
    Injective,
    Sei,
    Celestia,
    Solana,
    Sui,
    SmartChain,
}

impl StakeChain {
    /// Get the lock time in seconds
    pub fn get_lock_time(&self) -> u64 {
        match self {
            StakeChain::Cosmos | StakeChain::Injective | StakeChain::Sei | StakeChain::Celestia => {
                1_814_400
            }
            StakeChain::Solana => 259200,
            StakeChain::Sui => 86400,
            StakeChain::Osmosis => 1_036_800,
            StakeChain::SmartChain => 604800,
        }
    }

    /// Get the minimum stake amount
    pub fn get_min_stake_amount(&self) -> u64 {
        match self {
            StakeChain::Cosmos
            | StakeChain::Injective
            | StakeChain::Sei
            | StakeChain::Celestia
            | StakeChain::Osmosis => 0,
            StakeChain::Solana => 10_000_000, // 0.01 SOL
            StakeChain::Sui => 1_000_000_000, // 1 SUI
            StakeChain::SmartChain => 1_000_000_000_000_000_000, // 1 BNB
        }
    }
}
