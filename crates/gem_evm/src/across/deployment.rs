use crate::constants::*;
use alloy_primitives::map::HashSet;
use primitives::{AssetId, Chain};
use std::collections::HashMap;

pub struct AcrossDeployment {
    pub chain_id: u32,
    pub hub_pool: &'static str, // only for mainnet
    pub spoke_pool: &'static str,
}

impl AcrossDeployment {
    pub fn deployed_chains() -> Vec<Chain> {
        vec![
            Chain::Ethereum,
            Chain::Arbitrum,
            Chain::Base,
            Chain::Blast,
            Chain::Linea,
            Chain::Optimism,
            Chain::Polygon,
            Chain::World,
            Chain::ZkSync,
        ]
    }
    pub fn deployment_by_chain(chain: &Chain) -> Option<Self> {
        match chain {
            Chain::Ethereum => Some(Self {
                chain_id: 1,
                hub_pool: "0xc186fA914353c44b2E33eBE05f21846F1048bEda",
                spoke_pool: "0x5c7BCd6E7De5423a257D81B442095A1a6ced35C5",
            }),
            Chain::Arbitrum => Some(Self {
                chain_id: 42161,
                hub_pool: "",
                spoke_pool: "0xe35e9842fceaca96570b734083f4a58e8f7c5f2a",
            }),
            Chain::Base => Some(Self {
                chain_id: 8453,
                hub_pool: "",
                spoke_pool: "0x09aea4b2242abC8bb4BB78D537A67a245A7bEC64",
            }),
            Chain::Blast => Some(Self {
                chain_id: 81457,
                hub_pool: "",
                spoke_pool: "0x2D509190Ed0172ba588407D4c2df918F955Cc6E1",
            }),
            Chain::Linea => Some(Self {
                chain_id: 59144,
                hub_pool: "",
                spoke_pool: "0x7E63A5f1a8F0B4d0934B2f2327DAED3F6bb2ee75",
            }),
            Chain::Optimism => Some(Self {
                chain_id: 10,
                hub_pool: "",
                spoke_pool: "0x6f26Bf09B1C792e3228e5467807a900A503c0281",
            }),
            Chain::Polygon => Some(Self {
                chain_id: 137,
                hub_pool: "",
                spoke_pool: "0x9295ee1d8C5b022Be115A2AD3c30C72E34e7F096",
            }),
            Chain::World => Some(Self {
                chain_id: 480,
                hub_pool: "",
                spoke_pool: "0x09aea4b2242abC8bb4BB78D537A67a245A7bEC64",
            }),
            Chain::ZkSync => Some(Self {
                chain_id: 324,
                hub_pool: "",
                spoke_pool: "0xE0B015E54d54fc84a6cB9B666099c46adE9335FF",
            }),
            _ => None,
        }
    }

    pub fn supported_assets() -> HashMap<Chain, Vec<AssetId>> {
        HashMap::from([
            (
                Chain::Ethereum,
                vec![
                    ACX_ETH.into(),
                    DAI_ETH.into(),
                    USDC_ETH.into(),
                    USDT_ETH.into(),
                    USDC_E_ETH.into(),
                    WBTC_ETH.into(),
                    WETH_ETH.into(),
                ],
            ),
            (
                Chain::Optimism,
                vec![
                    ACX_OP.into(),
                    DAI_OP.into(),
                    USDT_OP.into(),
                    USDC_OP.into(),
                    USDC_E_OP.into(),
                    WBTC_OP.into(),
                    WETH_OP.into(),
                ],
            ),
            (
                Chain::Polygon,
                vec![
                    ACX_POLYGON.into(),
                    DAI_POLYGON.into(),
                    USDC_POLYGON.into(),
                    USDC_E_POLYGON.into(),
                    USDT_POLYGON.into(),
                    WBTC_POLYGON.into(),
                    WETH_POLYGON.into(),
                ],
            ),
            (
                Chain::Arbitrum,
                vec![
                    ACX_ARB.into(),
                    DAI_ARB.into(),
                    USDT_ARB.into(),
                    USDC_ARB.into(),
                    USDC_E_ARB.into(),
                    WBTC_ARB.into(),
                    WETH_ARB.into(),
                ],
            ),
            (Chain::Base, vec![WETH_BASE.into(), USDC_BASE.into(), USDC_E_BASE.into()]),
            (
                Chain::Linea,
                vec![DAI_LINEA.into(), USDC_E_LINEA.into(), USDT_LINEA.into(), WBTC_LINEA.into(), WETH_LINEA.into()],
            ),
            (
                Chain::ZkSync,
                vec![
                    DAI_ZKSYNC.into(),
                    WBTC_ZKSYNC.into(),
                    WETH_ZKSYNC.into(),
                    USDC_E_ZKSYNC.into(),
                    USDT_ZKSYNC.into(),
                ],
            ),
            (Chain::World, vec![WBTC_WORLD.into(), WETH_WORLD.into(), USDC_E_WORLD.into()]),
            (Chain::Blast, vec![WBTC_BLAST.into(), WETH_BLAST.into()]),
        ])
    }

    pub fn asset_mappings() -> Vec<HashSet<AssetId>> {
        vec![
            HashSet::from_iter([
                WETH_ARB.into(),
                WETH_BASE.into(),
                WETH_BLAST.into(),
                WETH_ETH.into(),
                WETH_LINEA.into(),
                WETH_OP.into(),
                WETH_POLYGON.into(),
                WETH_ZKSYNC.into(),
                WETH_WORLD.into(),
            ]),
            HashSet::from_iter([USDC_ARB.into(), USDC_BASE.into(), USDC_ETH.into(), USDC_OP.into(), USDC_POLYGON.into()]),
            HashSet::from_iter([
                USDT_ARB.into(),
                USDT_ETH.into(),
                USDT_LINEA.into(),
                USDT_OP.into(),
                USDT_POLYGON.into(),
                USDT_ZKSYNC.into(),
            ]),
            HashSet::from_iter([
                DAI_ARB.into(),
                DAI_BASE.into(),
                DAI_ETH.into(),
                DAI_LINEA.into(),
                DAI_OP.into(),
                DAI_POLYGON.into(),
                DAI_ZKSYNC.into(),
            ]),
            HashSet::from_iter([
                USDC_E_ARB.into(),
                USDC_E_BASE.into(),
                USDC_E_ETH.into(),
                USDC_E_LINEA.into(),
                USDC_E_OP.into(),
                USDC_E_POLYGON.into(),
                USDC_E_WORLD.into(),
                USDC_E_ZKSYNC.into(),
            ]),
            HashSet::from_iter([ACX_ARB.into(), ACX_ETH.into(), ACX_OP.into(), ACX_POLYGON.into()]),
        ]
    }
}
