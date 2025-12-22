use crate::{AssetType, EVMChain, chain_evm::ChainStack};

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

macro_rules! evm_chain_list {
    ($(
        $variant:ident {
            chain_id: $chain_id:expr,
            rpc_urls: [$($rpc_url:expr),* $(,)?],
            native_name: $native_name:expr,
            native_symbol: $native_symbol:expr,
            native_decimals: $native_decimals:expr,
            default_asset_type: $default_asset_type:expr,
            slip44: $slip44:expr,
            block_time_ms: $block_time_ms:expr,
            rank: $rank:expr,
            swap_supported: $swap_supported:expr,
            chain_stack: $chain_stack:expr,
            min_priority_fee: $min_priority_fee:expr,
            is_ethereum_layer2: $is_ethereum_layer2:expr,
            weth_contract: $weth_contract:expr,
        }
    ),+ $(,)?) => {
        static EVM_CHAIN_CONFIGS: &[EvmChainConfig] = &[
            $(
                EvmChainConfig {
                    chain: EVMChain::$variant,
                    chain_id: $chain_id,
                    rpc_urls: &[$($rpc_url),*],
                    native_name: $native_name,
                    native_symbol: $native_symbol,
                    native_decimals: $native_decimals,
                    default_asset_type: $default_asset_type,
                    slip44: $slip44,
                    block_time_ms: $block_time_ms,
                    rank: $rank,
                    swap_supported: $swap_supported,
                    chain_stack: $chain_stack,
                    min_priority_fee: $min_priority_fee,
                    is_ethereum_layer2: $is_ethereum_layer2,
                    weth_contract: $weth_contract,
                },
            )+
        ];

        pub(crate) fn evm_chain_config(chain: EVMChain) -> &'static EvmChainConfig {
            EVM_CHAIN_CONFIGS
                .iter()
                .find(|config| config.chain == chain)
                .unwrap_or_else(|| panic!("Missing EVM chain config for {:?}", chain))
        }
    };
}

include!("evm_chain_list.rs");
