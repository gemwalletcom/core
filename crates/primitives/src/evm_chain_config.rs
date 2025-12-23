use crate::{AssetType, EVMChain, chain_evm::ChainStack};
use crate::macros::{evm_chain_configs, with_evm_chain_list};

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
