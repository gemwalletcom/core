use super::fees::CapitalCostConfig;
use crate::ether_conv::EtherConv;
use alloy_primitives::map::HashSet;
use num_bigint::BigInt;
use primitives::{AssetId, Chain, asset_constants::*};
use std::{collections::HashMap, vec};

pub const ACROSS_CONFIG_STORE: &str = "0x3B03509645713718B78951126E0A6de6f10043f5";
pub const ACROSS_HUBPOOL: &str = "0xc186fA914353c44b2E33eBE05f21846F1048bEda";
pub const MULTICALL_HANDLER: &str = "0x924a9f036260DdD5808007E1AA95f08eD08aA569";

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
        let chain_id: u32 = chain.network_id().parse().unwrap();
        match chain {
            Chain::Ethereum => Some(Self {
                chain_id,
                spoke_pool: "0x5c7BCd6E7De5423a257D81B442095A1a6ced35C5",
            }),
            Chain::Arbitrum => Some(Self {
                chain_id,
                spoke_pool: "0xe35e9842fceaca96570b734083f4a58e8f7c5f2a",
            }),
            Chain::Base => Some(Self {
                chain_id,
                spoke_pool: "0x09aea4b2242abC8bb4BB78D537A67a245A7bEC64",
            }),
            Chain::Blast => Some(Self {
                chain_id,
                spoke_pool: "0x2D509190Ed0172ba588407D4c2df918F955Cc6E1",
            }),
            Chain::Linea => Some(Self {
                chain_id,
                spoke_pool: "0x7E63A5f1a8F0B4d0934B2f2327DAED3F6bb2ee75",
            }),
            Chain::Optimism => Some(Self {
                chain_id,
                spoke_pool: "0x6f26Bf09B1C792e3228e5467807a900A503c0281",
            }),
            Chain::Polygon => Some(Self {
                chain_id,
                spoke_pool: "0x9295ee1d8C5b022Be115A2AD3c30C72E34e7F096",
            }),
            Chain::World => Some(Self {
                chain_id,
                spoke_pool: "0x09aea4b2242abC8bb4BB78D537A67a245A7bEC64",
            }),
            Chain::ZkSync => Some(Self {
                chain_id,
                spoke_pool: "0xE0B015E54d54fc84a6cB9B666099c46adE9335FF",
            }),
            Chain::Ink => Some(Self {
                chain_id,
                spoke_pool: "0xeF684C38F94F48775959ECf2012D7E864ffb9dd4",
            }),
            Chain::Unichain => Some(Self {
                chain_id,
                spoke_pool: "0x09aea4b2242abC8bb4BB78D537A67a245A7bEC64",
            }),
            Chain::SmartChain => Some(Self {
                chain_id,
                spoke_pool: "0x4e8E101924eDE233C13e2D8622DC8aED2872d505",
            }),
            Chain::Hyperliquid => Some(Self {
                chain_id,
                spoke_pool: "0x35E63eA3eb0fb7A3bc543C71FB66412e1F6B0E04",
            }),
            Chain::Plasma => Some(Self {
                chain_id,
                spoke_pool: "0x50039fAEfebef707cFD94D6d462fE6D10B39207a",
            }),
            _ => None,
        }
    }

    pub fn multicall_handler(&self) -> String {
        match self.chain_id {
            // Linea
            59144 => "0x1015c58894961F4F7Dd7D68ba033e28Ed3ee1cDB".into(),
            // zkSync
            324 => "0x863859ef502F0Ee9676626ED5B418037252eFeb2".into(),
            // SmartChain
            56 => "0xAC537C12fE8f544D712d71ED4376a502EEa944d7".into(),
            // HyperEvm | Plasma
            999 | 9745 => "0x5E7840E06fAcCb6d1c3b5F5E0d1d3d07F2829bba".into(),
            _ => MULTICALL_HANDLER.into(),
        }
    }

    pub fn supported_assets() -> HashMap<Chain, Vec<AssetId>> {
        HashMap::from([
            (
                Chain::Ethereum,
                vec![USDC_ETH_ASSET_ID.into(), USDT_ETH_ASSET_ID.into(), WETH_ETH_ASSET_ID.into()],
            ),
            (Chain::Optimism, vec![USDT_OP_ASSET_ID.into(), USDC_OP_ASSET_ID.into(), WETH_OP_ASSET_ID.into()]),
            (
                Chain::Polygon,
                vec![USDC_POLYGON_ASSET_ID.into(), USDT_POLYGON_ASSET_ID.into(), WETH_POLYGON_ASSET_ID.into()],
            ),
            (
                Chain::Arbitrum,
                vec![USDT_ARB_ASSET_ID.into(), USDC_ARB_ASSET_ID.into(), WETH_ARB_ASSET_ID.into()],
            ),
            (Chain::Base, vec![WETH_BASE_ASSET_ID.into(), USDC_BASE_ASSET_ID.into()]),
            (Chain::Hyperliquid, vec![USDC_HYPEREVM_ASSET_ID.into(), USDT_HYPEREVM_ASSET_ID.into()]),
            (Chain::Linea, vec![USDT_LINEA_ASSET_ID.into(), WETH_LINEA_ASSET_ID.into()]),
            (Chain::ZkSync, vec![WETH_ZKSYNC_ASSET_ID.into(), USDT_ZKSYNC_ASSET_ID.into()]),
            (Chain::World, vec![WETH_WORLD_ASSET_ID.into()]),
            (Chain::Blast, vec![WETH_BLAST_ASSET_ID.into()]),
            (Chain::Ink, vec![WETH_INK_ASSET_ID.into(), USDT_INK_ASSET_ID.into()]),
            (Chain::Unichain, vec![WETH_UNICHAIN_ASSET_ID.into(), USDC_UNICHAIN_ASSET_ID.into()]),
            (Chain::SmartChain, vec![ETH_SMARTCHAIN_ASSET_ID.into()]),
            (Chain::Plasma, vec![USDT_PLASMA_ASSET_ID.into()]),
        ])
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
                    WETH_ARB_ASSET_ID.into(),
                    WETH_BASE_ASSET_ID.into(),
                    WETH_BLAST_ASSET_ID.into(),
                    WETH_ETH_ASSET_ID.into(),
                    WETH_LINEA_ASSET_ID.into(),
                    WETH_OP_ASSET_ID.into(),
                    WETH_POLYGON_ASSET_ID.into(),
                    WETH_ZKSYNC_ASSET_ID.into(),
                    WETH_WORLD_ASSET_ID.into(),
                    WETH_INK_ASSET_ID.into(),
                    WETH_UNICHAIN_ASSET_ID.into(),
                    ETH_SMARTCHAIN_ASSET_ID.into(),
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
                    USDC_ARB_ASSET_ID.into(),
                    USDC_BASE_ASSET_ID.into(),
                    USDC_ETH_ASSET_ID.into(),
                    USDC_OP_ASSET_ID.into(),
                    USDC_POLYGON_ASSET_ID.into(),
                    USDC_UNICHAIN_ASSET_ID.into(),
                    USDC_HYPEREVM_ASSET_ID.into(),
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
                set: HashSet::from_iter([USDC_ETH_ASSET_ID.into(), USDC_SMARTCHAIN_ASSET_ID.into()]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: EtherConv::parse_ether("0.0001"),
                    cutoff: EtherConv::parse_ether("1500000"),
                    decimals: 6,
                },
                set: HashSet::from_iter([
                    USDT_ARB_ASSET_ID.into(),
                    USDT_ETH_ASSET_ID.into(),
                    USDT_LINEA_ASSET_ID.into(),
                    USDT_OP_ASSET_ID.into(),
                    USDT_POLYGON_ASSET_ID.into(),
                    USDT_ZKSYNC_ASSET_ID.into(),
                    USDT_INK_ASSET_ID.into(),
                    USDT_HYPEREVM_ASSET_ID.into(),
                    USDT_PLASMA_ASSET_ID.into(),
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
                set: HashSet::from_iter([USDT_ETH_ASSET_ID.into(), USDT_SMARTCHAIN_ASSET_ID.into()]),
            },
            AssetMapping {
                capital_cost: CapitalCostConfig {
                    lower_bound: EtherConv::parse_ether("0.0001"),
                    upper_bound: EtherConv::parse_ether("0.0001"),
                    cutoff: EtherConv::parse_ether("1500000"),
                    decimals: 18,
                },
                set: HashSet::from_iter([
                    DAI_ARB_ASSET_ID.into(),
                    DAI_BASE_ASSET_ID.into(),
                    DAI_ETH_ASSET_ID.into(),
                    DAI_LINEA_ASSET_ID.into(),
                    DAI_OP_ASSET_ID.into(),
                    DAI_POLYGON_ASSET_ID.into(),
                    DAI_ZKSYNC_ASSET_ID.into(),
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
                    USDC_E_ARB_ASSET_ID.into(),
                    USDC_E_BASE_ASSET_ID.into(),
                    USDC_E_ETH_ASSET_ID.into(),
                    USDC_E_LINEA_ASSET_ID.into(),
                    USDC_E_OP_ASSET_ID.into(),
                    USDC_E_POLYGON_ASSET_ID.into(),
                    USDC_E_WORLD_ASSET_ID.into(),
                    USDC_E_ZKSYNC_ASSET_ID.into(),
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
                    WBTC_ARB_ASSET_ID.into(),
                    WBTC_BLAST_ASSET_ID.into(),
                    WBTC_ETH_ASSET_ID.into(),
                    WBTC_LINEA_ASSET_ID.into(),
                    WBTC_OP_ASSET_ID.into(),
                    WBTC_POLYGON_ASSET_ID.into(),
                    WBTC_WORLD_ASSET_ID.into(),
                    WBTC_ZKSYNC_ASSET_ID.into(),
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
                    ACX_ARB_ASSET_ID.into(),
                    ACX_ETH_ASSET_ID.into(),
                    ACX_OP_ASSET_ID.into(),
                    ACX_POLYGON_ASSET_ID.into(),
                ]),
            },
        ]
    }
}
