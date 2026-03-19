use super::fees::CapitalCostConfig;
use crate::ether_conv::EtherConv;
use alloy_primitives::map::HashSet;
use num_bigint::BigInt;
use primitives::{AssetId, Chain, asset_constants::*, contract_constants::*};
use std::{collections::HashMap, vec};

/// https://docs.across.to/developer-docs/developers/contract-addresses
pub struct AcrossDeployment {
    pub chain_id: u32,
    pub spoke_pool: &'static str,
}

#[derive(Debug)]
pub struct AssetMapping {
    pub capital_cost: CapitalCostConfig,
    pub set: HashSet<AssetId>,
}

impl AcrossDeployment {
    pub fn deployment_by_chain(chain: &Chain) -> Option<Self> {
        let chain_id: u32 = chain.network_id().parse().ok()?;
        let spoke_pool = match chain {
            Chain::Ethereum => ETHEREUM_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Arbitrum => ARBITRUM_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Base => BASE_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Blast => BLAST_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Linea => LINEA_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Optimism => OPTIMISM_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Polygon => POLYGON_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::World => WORLD_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::ZkSync => ZKSYNC_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Ink => INK_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Unichain => UNICHAIN_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Monad => MONAD_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::SmartChain => SMARTCHAIN_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Hyperliquid => HYPEREVM_ACROSS_SPOKE_POOL_CONTRACT,
            Chain::Plasma => PLASMA_ACROSS_SPOKE_POOL_CONTRACT,
            _ => return None,
        };
        Some(Self { chain_id, spoke_pool })
    }

    pub fn multicall_handler(&self) -> String {
        match self.chain_id {
            // Linea
            59144 => LINEA_ACROSS_MULTICALL_HANDLER_CONTRACT.into(),
            // zkSync
            324 => ZKSYNC_ACROSS_MULTICALL_HANDLER_CONTRACT.into(),
            // SmartChain
            56 => SMARTCHAIN_ACROSS_MULTICALL_HANDLER_CONTRACT.into(),
            // Monad
            143 => MONAD_ACROSS_MULTICALL_HANDLER_CONTRACT.into(),
            // HyperEvm | Plasma
            999 => HYPEREVM_ACROSS_MULTICALL_HANDLER_CONTRACT.into(),
            9745 => PLASMA_ACROSS_MULTICALL_HANDLER_CONTRACT.into(),
            _ => ETHEREUM_ACROSS_MULTICALL_HANDLER_CONTRACT.into(),
        }
    }

    pub fn supported_assets() -> HashMap<Chain, Vec<AssetId>> {
        HashMap::from([
            (
                Chain::Ethereum,
                vec![ETHEREUM_USDC_ASSET_ID.clone(), ETHEREUM_USDT_ASSET_ID.clone(), ETHEREUM_WETH_ASSET_ID.clone()],
            ),
            (
                Chain::Optimism,
                vec![OPTIMISM_USDT_ASSET_ID.clone(), OPTIMISM_USDC_ASSET_ID.clone(), OPTIMISM_WETH_ASSET_ID.clone()],
            ),
            (
                Chain::Polygon,
                vec![POLYGON_USDC_ASSET_ID.clone(), POLYGON_USDT_ASSET_ID.clone(), POLYGON_WETH_ASSET_ID.clone()],
            ),
            (
                Chain::Arbitrum,
                vec![ARBITRUM_USDT_ASSET_ID.clone(), ARBITRUM_USDC_ASSET_ID.clone(), ARBITRUM_WETH_ASSET_ID.clone()],
            ),
            (Chain::Base, vec![BASE_WETH_ASSET_ID.clone(), BASE_USDC_ASSET_ID.clone()]),
            (Chain::Hyperliquid, vec![HYPEREVM_USDC_ASSET_ID.clone(), HYPEREVM_USDT_ASSET_ID.clone()]),
            (Chain::Linea, vec![LINEA_USDT_ASSET_ID.clone(), LINEA_WETH_ASSET_ID.clone()]),
            (Chain::ZkSync, vec![ZKSYNC_WETH_ASSET_ID.clone(), ZKSYNC_USDT_ASSET_ID.clone()]),
            (Chain::World, vec![WORLD_WETH_ASSET_ID.clone()]),
            (Chain::Blast, vec![BLAST_WETH_ASSET_ID.clone()]),
            (Chain::Ink, vec![INK_WETH_ASSET_ID.clone(), INK_USDT_ASSET_ID.clone()]),
            (Chain::Unichain, vec![UNICHAIN_WETH_ASSET_ID.clone(), UNICHAIN_USDC_ASSET_ID.clone()]),
            (Chain::Monad, vec![MONAD_USDC_ASSET_ID.clone(), MONAD_USDT_ASSET_ID.clone()]),
            (Chain::SmartChain, vec![SMARTCHAIN_ETH_ASSET_ID.clone()]),
            (Chain::Plasma, vec![PLASMA_USDT_ASSET_ID.clone()]),
        ])
    }

    pub fn deposit_addresses() -> Vec<String> {
        let mut addresses: HashSet<String> = HashSet::default();
        for chain in Chain::all() {
            if let Some(deployment) = Self::deployment_by_chain(&chain) {
                addresses.insert(deployment.spoke_pool.to_string());
            }
        }
        addresses.into_iter().collect()
    }

    pub fn send_addresses() -> Vec<String> {
        let mut addresses: HashSet<String> = HashSet::default();
        for chain in Chain::all() {
            if let Some(deployment) = Self::deployment_by_chain(&chain) {
                addresses.insert(deployment.spoke_pool.to_string());
                addresses.insert(deployment.multicall_handler());
            }
        }
        addresses.into_iter().collect()
    }

    pub fn asset_mappings() -> Vec<AssetMapping> {
        vec![
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: EtherConv::parse_ether("0.000075"),
                    cutoff: EtherConv::parse_ether("0.3"),
                    decimals: 18,
                },
                set: HashSet::from_iter([
                    ARBITRUM_WETH_ASSET_ID.clone(),
                    BASE_WETH_ASSET_ID.clone(),
                    BLAST_WETH_ASSET_ID.clone(),
                    ETHEREUM_WETH_ASSET_ID.clone(),
                    LINEA_WETH_ASSET_ID.clone(),
                    OPTIMISM_WETH_ASSET_ID.clone(),
                    POLYGON_WETH_ASSET_ID.clone(),
                    ZKSYNC_WETH_ASSET_ID.clone(),
                    WORLD_WETH_ASSET_ID.clone(),
                    INK_WETH_ASSET_ID.clone(),
                    UNICHAIN_WETH_ASSET_ID.clone(),
                    SMARTCHAIN_ETH_ASSET_ID.clone(),
                ]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: BigInt::from(0),
                    cutoff: EtherConv::parse_ether("100000"),
                    decimals: 6,
                },
                set: HashSet::from_iter([
                    ARBITRUM_USDC_ASSET_ID.clone(),
                    BASE_USDC_ASSET_ID.clone(),
                    ETHEREUM_USDC_ASSET_ID.clone(),
                    OPTIMISM_USDC_ASSET_ID.clone(),
                    POLYGON_USDC_ASSET_ID.clone(),
                    UNICHAIN_USDC_ASSET_ID.clone(),
                    HYPEREVM_USDC_ASSET_ID.clone(),
                    MONAD_USDC_ASSET_ID.clone(),
                ]),
            },
            // USDC on BSC decimals are 18
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: BigInt::from(0),
                    cutoff: EtherConv::parse_ether("100000"),
                    decimals: 18,
                },
                set: HashSet::from_iter([ETHEREUM_USDC_ASSET_ID.clone(), SMARTCHAIN_USDC_ASSET_ID.clone()]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: EtherConv::parse_ether("0.0001"),
                    cutoff: EtherConv::parse_ether("1500000"),
                    decimals: 6,
                },
                set: HashSet::from_iter([
                    ARBITRUM_USDT_ASSET_ID.clone(),
                    ETHEREUM_USDT_ASSET_ID.clone(),
                    LINEA_USDT_ASSET_ID.clone(),
                    OPTIMISM_USDT_ASSET_ID.clone(),
                    POLYGON_USDT_ASSET_ID.clone(),
                    ZKSYNC_USDT_ASSET_ID.clone(),
                    INK_USDT_ASSET_ID.clone(),
                    HYPEREVM_USDT_ASSET_ID.clone(),
                    PLASMA_USDT_ASSET_ID.clone(),
                    MONAD_USDT_ASSET_ID.clone(),
                ]),
            },
            // USDT on BSC decimals are 18
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: EtherConv::parse_ether("0.0001"),
                    cutoff: EtherConv::parse_ether("1500000"),
                    decimals: 18,
                },
                set: HashSet::from_iter([ETHEREUM_USDT_ASSET_ID.clone(), SMARTCHAIN_USDT_ASSET_ID.clone()]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: EtherConv::parse_ether("0.0001"),
                    cutoff: EtherConv::parse_ether("1500000"),
                    decimals: 18,
                },
                set: HashSet::from_iter([
                    ARBITRUM_DAI_ASSET_ID.clone(),
                    BASE_DAI_ASSET_ID.clone(),
                    ETHEREUM_DAI_ASSET_ID.clone(),
                    LINEA_DAI_ASSET_ID.clone(),
                    OPTIMISM_DAI_ASSET_ID.clone(),
                    POLYGON_DAI_ASSET_ID.clone(),
                    ZKSYNC_DAI_ASSET_ID.clone(),
                ]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: BigInt::from(0),
                    cutoff: EtherConv::parse_ether("100000"),
                    decimals: 6,
                },
                set: HashSet::from_iter([
                    ARBITRUM_USDC_E_ASSET_ID.clone(),
                    BASE_USDC_E_ASSET_ID.clone(),
                    ETHEREUM_USDC_E_ASSET_ID.clone(),
                    LINEA_USDC_E_ASSET_ID.clone(),
                    OPTIMISM_USDC_E_ASSET_ID.clone(),
                    POLYGON_USDC_E_ASSET_ID.clone(),
                    WORLD_USDC_E_ASSET_ID.clone(),
                    ZKSYNC_USDC_E_ASSET_ID.clone(),
                ]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0003"),
                    upper_bound: EtherConv::parse_ether("0.0025"),
                    cutoff: EtherConv::parse_ether("10"),
                    decimals: 8,
                },
                set: HashSet::from_iter([
                    ARBITRUM_WBTC_ASSET_ID.clone(),
                    BLAST_WBTC_ASSET_ID.clone(),
                    ETHEREUM_WBTC_ASSET_ID.clone(),
                    LINEA_WBTC_ASSET_ID.clone(),
                    OPTIMISM_WBTC_ASSET_ID.clone(),
                    POLYGON_WBTC_ASSET_ID.clone(),
                    WORLD_WBTC_ASSET_ID.clone(),
                    ZKSYNC_WBTC_ASSET_ID.clone(),
                ]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: EtherConv::parse_ether("0.001"),
                    cutoff: EtherConv::parse_ether("1000000"),
                    decimals: 18,
                },
                set: HashSet::from_iter([
                    ARBITRUM_ACX_ASSET_ID.clone(),
                    ETHEREUM_ACX_ASSET_ID.clone(),
                    OPTIMISM_ACX_ASSET_ID.clone(),
                    POLYGON_ACX_ASSET_ID.clone(),
                ]),
            },
        ]
    }
}
