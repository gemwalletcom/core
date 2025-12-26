use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};
use typeshare::typeshare;

use crate::Chain;
use crate::chain_config::StakeChainConfig;

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
    Monad,
    Tron,
    Aptos,
    HyperCore,
}

impl StakeChain {
    fn config(&self) -> &'static StakeChainConfig {
        let chain = self.chain();
        let config = chain.config();
        config.stake.as_ref().unwrap_or_else(|| panic!("Missing stake config for {chain}"))
    }

    pub fn chain(&self) -> Chain {
        Chain::from_str(self.as_ref()).unwrap()
    }

    /// Get the lock time in seconds
    pub fn get_lock_time(&self) -> u64 {
        self.config().lock_time
    }

    /// Get the minimum stake amount
    pub fn get_min_stake_amount(&self) -> u64 {
        self.config().min_stake_amount
    }

    /// Get if chain support ability to change amount on unstake
    pub fn get_change_amount_on_unstake(&self) -> bool {
        self.config().change_amount_on_unstake
    }

    /// Get if chain support redelegate
    pub fn get_can_redelegate(&self) -> bool {
        self.config().can_redelegate
    }

    pub fn get_can_withdraw(&self) -> bool {
        self.config().can_withdraw
    }

    pub fn get_can_claim_rewards(&self) -> bool {
        self.config().can_claim_rewards
    }

    pub fn get_reserved_for_fees(&self) -> u64 {
        self.config().reserved_for_fees
    }
}
