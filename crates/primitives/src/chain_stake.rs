use serde::{Deserialize, Serialize};
use strum::EnumIter;
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
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
    Tron,
}

impl StakeChain {
    /// Get the lock time in seconds
    pub fn get_lock_time(&self) -> u64 {
        match self {
            Self::Cosmos | Self::Injective | Self::Sei | Self::Celestia => 1_814_400,
            Self::Solana => 259200,
            Self::Sui => 86400,
            Self::Osmosis | Self::Tron => 1_209_600,
            Self::SmartChain => 604800,
        }
    }

    /// Get the minimum stake amount
    pub fn get_min_stake_amount(&self) -> u64 {
        match self {
            Self::Cosmos | Self::Injective | Self::Sei | Self::Celestia | Self::Osmosis => 0,
            Self::Solana => 10_000_000,                    // 0.01 SOL
            Self::Sui => 1_000_000_000,                    // 1 SUI
            Self::SmartChain => 1_000_000_000_000_000_000, // 1 BNB
            Self::Tron => 1_000_000,                       // 1 TRX
        }
    }

    /// Get if chain support ability to change amount on unstake
    pub fn get_change_amount_on_unstake(&self) -> bool {
        match self {
            Self::Cosmos | Self::Osmosis | Self::Injective | Self::Sei | Self::Celestia | Self::Solana | Self::SmartChain | Self::Tron => true,
            Self::Sui => false,
        }
    }

    /// Get if chain support redelegate
    pub fn get_can_redelegate(&self) -> bool {
        match self {
            Self::Cosmos | Self::Osmosis | Self::Injective | Self::Sei | Self::Celestia | Self::SmartChain | Self::Tron => true,
            Self::Sui | Self::Solana => false,
        }
    }

    pub fn get_can_withdraw(&self) -> bool {
        match self {
            Self::Solana | Self::Tron | Self::SmartChain => true,
            Self::Cosmos | Self::Osmosis | Self::Injective | Self::Sei | Self::Celestia | Self::Sui => false,
        }
    }

    pub fn get_can_claim_rewards(&self) -> bool {
        match self {
            Self::Cosmos | Self::Osmosis | Self::Injective | Self::Sei | Self::Celestia | Self::Tron => true,
            Self::SmartChain | Self::Sui | Self::Solana => false,
        }
    }
}
