use crate::{AssetType, Chain};
use crate::macros::{define_evm_chain, evm_chain_configs, with_evm_chain_list};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChainStack {
    Native,
    Optimism,
    ZkSync,
}

with_evm_chain_list!(define_evm_chain);

#[derive(Debug)]
pub(crate) struct EvmChainConfig {
    pub chain: EVMChain,
    pub chain_id: &'static str,
    pub rpc_urls: &'static [&'static str],
    pub native_name: &'static str,
    pub native_symbol: &'static str,
    pub native_decimals: i32,
    pub default_asset_type: AssetType,
    pub slip44: i64,
    pub block_time_ms: u32,
    pub rank: i32,
    pub swap_supported: bool,
    pub chain_stack: ChainStack,
    pub min_priority_fee: u64,
    pub is_ethereum_layer2: bool,
    pub weth_contract: Option<&'static str>,
}

with_evm_chain_list!(evm_chain_configs);

impl EVMChain {
    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }

    fn config(&self) -> &'static EvmChainConfig {
        evm_chain_config(*self)
    }

    pub fn chain_id(&self) -> &'static str {
        self.config().chain_id
    }

    pub fn rpc_urls(&self) -> &'static [&'static str] {
        self.config().rpc_urls
    }

    pub fn rpc_url(&self) -> &'static str {
        self.config().rpc_urls.first().copied().unwrap_or_default()
    }

    pub fn native_name(&self) -> &'static str {
        self.config().native_name
    }

    pub fn native_symbol(&self) -> &'static str {
        self.config().native_symbol
    }

    pub fn native_decimals(&self) -> i32 {
        self.config().native_decimals
    }

    pub fn default_asset_type(&self) -> AssetType {
        self.config().default_asset_type.clone()
    }

    pub fn slip44(&self) -> i64 {
        self.config().slip44
    }

    pub fn block_time(&self) -> u32 {
        self.config().block_time_ms
    }

    pub fn rank(&self) -> i32 {
        self.config().rank
    }

    pub fn swap_supported(&self) -> bool {
        self.config().swap_supported
    }

    pub fn min_priority_fee(&self) -> u64 {
        self.config().min_priority_fee
    }

    pub fn chain_stack(&self) -> ChainStack {
        self.config().chain_stack
    }

    pub fn is_ethereum_layer2(&self) -> bool {
        self.config().is_ethereum_layer2
    }

    // https://docs.optimism.io/stack/getting-started
    pub fn is_opstack(&self) -> bool {
        self.chain_stack() == ChainStack::Optimism
    }

    // https://docs.zksync.io/zk-stack/running/quickstart
    pub fn is_zkstack(&self) -> bool {
        self.chain_stack() == ChainStack::ZkSync
    }

    pub fn weth_contract(&self) -> Option<&'static str> {
        self.config().weth_contract
    }

    pub fn from_chain(chain: Chain) -> Option<Self> {
        EVMChain::from_str(chain.as_ref()).ok()
    }

    pub fn to_chain(&self) -> Chain {
        Chain::from_str(self.as_ref()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Chain, EVMChain};

    #[test]
    fn test_from_chain() {
        assert_eq!(EVMChain::from_chain(Chain::Ethereum), Some(EVMChain::Ethereum));
        assert_eq!(EVMChain::from_chain(Chain::Bitcoin), None);
    }
}
