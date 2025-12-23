macro_rules! define_evm_chain {
    ($(
        $variant:ident { $($field:tt)* }
    ),+ $(,)?) => {
        #[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString, PartialEq, Eq, Hash)]
        #[typeshare(swift = "Equatable, Hashable, CaseIterable, Sendable")]
        #[serde(rename_all = "lowercase")]
        #[strum(serialize_all = "lowercase")]
        pub enum EVMChain {
            $($variant,)+
        }
    };
}

macro_rules! evm_chain_configs {
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

include!("chain_evm_list.rs");

pub(crate) use define_evm_chain;
pub(crate) use evm_chain_configs;
pub(crate) use with_evm_chain_list;
