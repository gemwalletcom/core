use std::str::FromStr;

use crate::Chain;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};
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
    Ethereum,
    Solana,
    Sui,
    SmartChain,
    Tron,
    Aptos,
    HyperCore,
}

impl StakeChain {
    pub fn chain(&self) -> Chain {
        Chain::from_str(self.as_ref()).unwrap()
    }

    /// Get the lock time in seconds
    pub fn get_lock_time(&self) -> u64 {
        match self {
            Self::Cosmos | Self::Injective | Self::Sei | Self::Celestia => 1_814_400,
            Self::Ethereum => 1_209_600, // ~14 days, Exit Queue is dynamic
            Self::Solana => 259200,
            Self::Sui => 86400,
            Self::Osmosis | Self::Tron => 1_209_600,
            Self::SmartChain => 604800,
            Self::Aptos => 2_592_000, // 30 days
            Self::HyperCore => 604800,
        }
    }

    /// Get the minimum stake amount
    pub fn get_min_stake_amount(&self) -> u64 {
        match self {
            Self::Cosmos | Self::Injective | Self::Sei | Self::Celestia | Self::Osmosis => 0,
            Self::Ethereum => 100_000_000_000_000_000,     // 0.1 ETH
            Self::Solana => 10_000_000,                    // 0.01 SOL
            Self::Sui => 1_000_000_000,                    // 1 SUI
            Self::SmartChain => 1_000_000_000_000_000_000, // 1 BNB
            Self::Tron => 1_000_000,                       // 1 TRX
            Self::Aptos => 1_100_000_000,                  // 11 APT
            Self::HyperCore => 10000000000000000,          // 0.01 HYPE
        }
    }

    /// Get if chain support ability to change amount on unstake
    pub fn get_change_amount_on_unstake(&self) -> bool {
        match self {
            Self::Cosmos
            | Self::Osmosis
            | Self::Injective
            | Self::Sei
            | Self::Celestia
            | Self::Ethereum
            | Self::SmartChain
            | Self::Tron
            | Self::Aptos
            | Self::HyperCore => true,
            Self::Sui | Self::Solana => false,
        }
    }

    /// Get if chain support redelegate
    pub fn get_can_redelegate(&self) -> bool {
        match self {
            Self::Cosmos | Self::Osmosis | Self::Injective | Self::Sei | Self::Celestia | Self::SmartChain | Self::Tron => true,
            Self::Ethereum | Self::Sui | Self::Solana | Self::Aptos | Self::HyperCore => false,
        }
    }

    pub fn get_can_withdraw(&self) -> bool {
        match self {
            Self::Ethereum | Self::Solana | Self::Tron | Self::SmartChain | Self::Aptos => true,
            Self::Cosmos | Self::Osmosis | Self::Injective | Self::Sei | Self::Celestia | Self::Sui | Self::HyperCore => false,
        }
    }

    pub fn get_can_claim_rewards(&self) -> bool {
        match self {
            Self::Cosmos | Self::Osmosis | Self::Injective | Self::Sei | Self::Celestia | Self::Tron => true,
            Self::Ethereum | Self::SmartChain | Self::Sui | Self::Solana | Self::Aptos | Self::HyperCore => false,
        }
    }

    pub fn get_reserved_for_fees(&self) -> u64 {
        match self {
            Self::Cosmos => 25_000,                    // 0.025 ATOM
            Self::Osmosis => 10_000,                   // 0.01 OSMO
            Self::Injective => 10_000_000_000_000_000, // 0.01 INJ
            Self::Sei => 100_000,                      // 0.1 SEI
            Self::Celestia => 100_000,                 // 0.1 TIA
            Self::Ethereum => 5_000_000_000_000_000,   // 0.005 ETH
            Self::Solana => 5_000_000,                 // 0.005 SOL
            Self::Sui => 100_000_000,                  // 0.1 SUI
            Self::SmartChain => 250_000_000_000_000,   // 0.00025 BNB
            Self::Tron => 10_000_000,                  // 10 TRX
            Self::Aptos => 1_000_000,                  // 0.01 APT
            Self::HyperCore => 0,                      // 0 HLC (TODO: update value)
        }
    }
}
