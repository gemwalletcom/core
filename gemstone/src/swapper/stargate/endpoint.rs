use lazy_static::lazy_static;
use primitives::{AssetId, Chain};

use crate::swapper::{
    asset::{
        ARBITRUM_USDC, ARBITRUM_USDT, AVALANCHE_USDC, AVALANCHE_USDT, BASE_USDC, ETHEREUM_USDC, ETHEREUM_USDT, OPTIMISM_USDC, OPTIMISM_USDT, POLYGON_USDC,
        POLYGON_USDT, SMARTCHAIN_USDC, SMARTCHAIN_USDT,
    },
    SwapChainAsset,
};

#[derive(Clone, Debug, PartialEq)]
pub struct StargatePool {
    pub asset: AssetId,
    pub address: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StargateEndpoint {
    pub id: Chain,
    pub pools: Vec<StargatePool>,
    pub endpoint_id: u32,
    pub chain_asset: SwapChainAsset,
}

#[derive(Clone, Debug)]
pub struct StargateRoutes {
    pub ethereum: StargateEndpoint,
    pub base: StargateEndpoint,
    pub optimism: StargateEndpoint,
    pub arbitrum: StargateEndpoint,
    pub polygon: StargateEndpoint,
    pub avalanche: StargateEndpoint,
    pub linea: StargateEndpoint,
    pub smartchain: StargateEndpoint,
}

lazy_static! {
    pub static ref STARGATE_ROUTES: StargateRoutes = StargateRoutes {
        ethereum: StargateEndpoint {
            id: Chain::Ethereum,
            pools: vec![
                StargatePool {
                    asset: AssetId::from_chain(Chain::Ethereum),
                    address: "0x77b2043768d28E9C9aB44E1aBfC95944bcE57931".to_string(),
                },
                StargatePool {
                    asset: ETHEREUM_USDC.id.clone(),
                    address: "0xc026395860Db2d07ee33e05fE50ed7bD583189C7".to_string(),
                },
                StargatePool {
                    asset: ETHEREUM_USDT.id.clone(),
                    address: "0x933597a323Eb81cAe705C5bC29985172fd5A3973".to_string(),
                },
            ],
            endpoint_id: 30101,
            chain_asset: SwapChainAsset::Assets(
                Chain::Ethereum,
                vec![AssetId::from_chain(Chain::Ethereum), ETHEREUM_USDC.id.clone(), ETHEREUM_USDT.id.clone(),]
            ),
        },
        base: StargateEndpoint {
            id: Chain::Base,
            pools: vec![
                StargatePool {
                    asset: AssetId::from_chain(Chain::Base),
                    address: "0xdc181Bd607330aeeBEF6ea62e03e5e1Fb4B6F7C7".to_string(),
                },
                StargatePool {
                    asset: BASE_USDC.id.clone(),
                    address: "0x27a16dc786820B16E5c9028b75B99F6f604b5d26".to_string(),
                },
            ],
            endpoint_id: 30184,
            chain_asset: SwapChainAsset::Assets(Chain::Base, vec![BASE_USDC.id.clone(),]),
        },
        optimism: StargateEndpoint {
            id: Chain::Optimism,
            pools: vec![
                StargatePool {
                    asset: AssetId::from_chain(Chain::Optimism),
                    address: "0xe8CDF27AcD73a434D661C84887215F7598e7d0d3".to_string(),
                },
                StargatePool {
                    asset: OPTIMISM_USDC.id.clone(),
                    address: "0xcE8CcA271Ebc0533920C83d39F417ED6A0abB7D0".to_string(),
                },
                StargatePool {
                    asset: OPTIMISM_USDT.id.clone(),
                    address: "0x19cFCE47eD54a88614648DC3f19A5980097007dD".to_string(),
                },
            ],
            endpoint_id: 30111,
            chain_asset: SwapChainAsset::Assets(Chain::Optimism, vec![OPTIMISM_USDC.id.clone(), OPTIMISM_USDT.id.clone(),]),
        },
        arbitrum: StargateEndpoint {
            id: Chain::Arbitrum,
            pools: vec![
                StargatePool {
                    asset: AssetId::from_chain(Chain::Arbitrum),
                    address: "0xA45B5130f36CDcA45667738e2a258AB09f4A5f7F".to_string(),
                },
                StargatePool {
                    asset: ARBITRUM_USDC.id.clone(),
                    address: "0xe8CDF27AcD73a434D661C84887215F7598e7d0d3".to_string(),
                },
                StargatePool {
                    asset: ARBITRUM_USDT.id.clone(),
                    address: "0xcE8CcA271Ebc0533920C83d39F417ED6A0abB7D0".to_string(),
                },
            ],
            endpoint_id: 30110,
            chain_asset: SwapChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_USDC.id.clone(), ARBITRUM_USDT.id.clone(),]),
        },
        polygon: StargateEndpoint {
            id: Chain::Polygon,
            pools: vec![
                StargatePool {
                    asset: POLYGON_USDC.id.clone(),
                    address: "0x9Aa02D4Fae7F58b8E8f34c66E756cC734DAc7fe4".to_string(),
                },
                StargatePool {
                    asset: POLYGON_USDT.id.clone(),
                    address: "0xd47b03ee6d86Cf251ee7860FB2ACf9f91B9fD4d7".to_string(),
                },
            ],
            endpoint_id: 30109,
            chain_asset: SwapChainAsset::Assets(Chain::Polygon, vec![POLYGON_USDC.id.clone(), POLYGON_USDT.id.clone(),]),
        },
        avalanche: StargateEndpoint {
            id: Chain::AvalancheC,
            pools: vec![
                StargatePool {
                    asset: AVALANCHE_USDC.id.clone(),
                    address: "0x5634c4a5FEd09819E3c46D86A965Dd9447d86e47".to_string(),
                },
                StargatePool {
                    asset: AVALANCHE_USDT.id.clone(),
                    address: "0x12dC9256Acc9895B076f6638D628382881e62CeE".to_string(),
                },
            ],
            endpoint_id: 30106,
            chain_asset: SwapChainAsset::Assets(Chain::AvalancheC, vec![AVALANCHE_USDC.id.clone(), AVALANCHE_USDT.id.clone(),]),
        },
        linea: StargateEndpoint {
            id: Chain::Linea,
            pools: vec![StargatePool {
                asset: AssetId::from_chain(Chain::Linea),
                address: "0x81F6138153d473E8c5EcebD3DC8Cd4903506B075".to_string(),
            },],
            endpoint_id: 30183,
            chain_asset: SwapChainAsset::Assets(Chain::Linea, vec![AssetId::from_chain(Chain::Linea),]),
        },
        smartchain: StargateEndpoint {
            id: Chain::SmartChain,
            pools: vec![
                StargatePool {
                    asset: SMARTCHAIN_USDC.id.clone(),
                    address: "0x962Bd449E630b0d928f308Ce63f1A21F02576057".to_string(),
                },
                StargatePool {
                    asset: SMARTCHAIN_USDT.id.clone(),
                    address: "0x138EB30f73BC423c6455C53df6D89CB01d9eBc63".to_string(),
                },
            ],
            endpoint_id: 30102,
            chain_asset: SwapChainAsset::Assets(Chain::SmartChain, vec![SMARTCHAIN_USDC.id.clone(), SMARTCHAIN_USDT.id.clone(),]),
        },
    };
}
