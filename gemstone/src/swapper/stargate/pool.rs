use primitives::Chain;

use crate::swapper::{
    asset::{
        ARBITRUM_USDC, ARBITRUM_USDT, ARBITRUM_WETH, AVALANCHE_USDC, AVALANCHE_USDT, BASE_USDC, BASE_WETH, ETHEREUM_USDC, ETHEREUM_USDT, ETHEREUM_WETH,
        LINEA_WETH, OPTIMISM_USDC, OPTIMISM_USDT, OPTIMISM_WETH, POLYGON_USDC, POLYGON_USDT, SMARTCHAIN_USDC, SMARTCHAIN_USDT,
    },
    SwapChainAsset, SwapperError,
};

#[derive(Debug, PartialEq)]
pub enum StargatePool {
    // ETH
    ETH_ETH,
    ETH_USDC,
    ETH_USDT,

    // BASE
    BASE_ETH,
    BASE_USDC,
    // Note: BASE has no USDT pool

    // OPTIMISM
    OPTIMISM_ETH,
    OPTIMISM_USDC,
    OPTIMISM_USDT,

    // ARBITRUM
    ARBITRUM_ETH,
    ARBITRUM_USDC,
    ARBITRUM_USDT,

    // POLYGON
    POLYGON_USDC,
    POLYGON_USDT,
    // Note: POLYGON has no native ETH pool

    // AVALANCHE
    AVALANCHE_USDC,
    AVALANCHE_USDT,
    // Note: AVALANCHE has no native ETH pool

    // METIS
    METIS_ETH,
    METIS_USDT,
    // Note: METIS has no USDC pool

    // BNB
    BNB_USDC,
    BNB_USDT,
    // Note: BNB has no native ETH pool

    // SCROLL
    SCROLL_ETH,
    SCROLL_USDC,

    // LINEA
    LINEA_ETH,

    // MANTLE
    MANTLE_ETH,
    MANTLE_USDC,
    MANTLE_USDT,
}

impl StargatePool {
    pub fn to_stargate_contract_address(&self) -> Result<String, SwapperError> {
        let address = match self {
            // Ethereum
            StargatePool::ETH_ETH => Some("0x77b2043768d28E9C9aB44E1aBfC95944bcE57931".to_string()),
            StargatePool::ETH_USDC => Some("0xc026395860Db2d07ee33e05fE50ed7bD583189C7".to_string()),
            StargatePool::ETH_USDT => Some("0x933597a323Eb81cAe705C5bC29985172fd5A3973".to_string()),

            // Base
            StargatePool::BASE_ETH => Some("0xdc181Bd607330aeeBEF6ea62e03e5e1Fb4B6F7C7".to_string()),
            StargatePool::BASE_USDC => Some("0x27a16dc786820B16E5c9028b75B99F6f604b5d26".to_string()),

            // Optimism
            StargatePool::OPTIMISM_ETH => Some("0xe8CDF27AcD73a434D661C84887215F7598e7d0d3".to_string()),
            StargatePool::OPTIMISM_USDC => Some("0xcE8CcA271Ebc0533920C83d39F417ED6A0abB7D0".to_string()),
            StargatePool::OPTIMISM_USDT => Some("0x19cFCE47eD54a88614648DC3f19A5980097007dD".to_string()),

            // Arbitrum
            StargatePool::ARBITRUM_ETH => Some("0xA45B5130f36CDcA45667738e2a258AB09f4A5f7F".to_string()),
            StargatePool::ARBITRUM_USDC => Some("0xe8CDF27AcD73a434D661C84887215F7598e7d0d3".to_string()),
            StargatePool::ARBITRUM_USDT => Some("0xcE8CcA271Ebc0533920C83d39F417ED6A0abB7D0".to_string()),

            // Polygon
            StargatePool::POLYGON_USDC => Some("0x9Aa02D4Fae7F58b8E8f34c66E756cC734DAc7fe4".to_string()),
            StargatePool::POLYGON_USDT => Some("0xd47b03ee6d86Cf251ee7860FB2ACf9f91B9fD4d7".to_string()),

            // Avalanche
            StargatePool::AVALANCHE_USDC => Some("0x5634c4a5FEd09819E3c46D86A965Dd9447d86e47".to_string()),
            StargatePool::AVALANCHE_USDT => Some("0x12dC9256Acc9895B076f6638D628382881e62CeE".to_string()),

            // Metis
            StargatePool::METIS_ETH => Some("0x36ed193dc7160D3858EC250e69D12B03Ca087D08".to_string()),
            StargatePool::METIS_USDT => Some("0x4dCBFC0249e8d5032F89D6461218a9D2eFff5125".to_string()),

            // BNB Chain
            StargatePool::BNB_USDC => Some("0x962Bd449E630b0d928f308Ce63f1A21F02576057".to_string()),
            StargatePool::BNB_USDT => Some("0x138EB30f73BC423c6455C53df6D89CB01d9eBc63".to_string()),

            // Scroll
            StargatePool::SCROLL_ETH => Some("0xC2b638Cb5042c1B3c5d5C969361fB50569840583".to_string()),
            StargatePool::SCROLL_USDC => Some("0x3Fc69CC4A842838bCDC9499178740226062b14E4".to_string()),

            // Linea
            StargatePool::LINEA_ETH => Some("0x81F6138153d473E8c5EcebD3DC8Cd4903506B075".to_string()),

            // Mantle
            StargatePool::MANTLE_ETH => Some("0x4c1d3Fc3fC3c177c3b633427c2F769276c547463".to_string()),
            StargatePool::MANTLE_USDC => Some("0xAc290Ad4e0c891FDc295ca4F0a6214cf6dC6acDC".to_string()),
            StargatePool::MANTLE_USDT => Some("0xB715B85682B731dB9D5063187C450095c91C57FC".to_string()),
        };
        Ok(address.ok_or(SwapperError::NotImplemented)?)
    }

    pub fn endpoint_id(&self) -> u32 {
        match self {
            // Ethereum (all pools)
            StargatePool::ETH_ETH | StargatePool::ETH_USDC | StargatePool::ETH_USDT => 30101,

            // Base (all pools)
            StargatePool::BASE_ETH | StargatePool::BASE_USDC => 30184,

            // Optimism (all pools)
            StargatePool::OPTIMISM_ETH | StargatePool::OPTIMISM_USDC | StargatePool::OPTIMISM_USDT => 30111,

            // Arbitrum (all pools)
            StargatePool::ARBITRUM_ETH | StargatePool::ARBITRUM_USDC | StargatePool::ARBITRUM_USDT => 30110,

            // Polygon (all pools)
            StargatePool::POLYGON_USDC | StargatePool::POLYGON_USDT => 30109,

            // Avalanche (all pools)
            StargatePool::AVALANCHE_USDC | StargatePool::AVALANCHE_USDT => 30106,

            // Metis (all pools)
            StargatePool::METIS_ETH | StargatePool::METIS_USDT => 30151,

            // BNB (all pools)
            StargatePool::BNB_USDC | StargatePool::BNB_USDT => 30102,

            // Scroll (all pools)
            StargatePool::SCROLL_ETH | StargatePool::SCROLL_USDC => 30214,

            // Linea (all pools)
            StargatePool::LINEA_ETH => 30183,

            // Mantle (all pools)
            StargatePool::MANTLE_ETH | StargatePool::MANTLE_USDC | StargatePool::MANTLE_USDT => 30181,
        }
    }

    pub fn from_chain(chain: &Chain) -> Vec<Self> {
        match chain {
            Chain::Ethereum => vec![Self::ETH_ETH, Self::ETH_USDC, Self::ETH_USDT],
            Chain::Base => vec![Self::BASE_ETH, Self::BASE_USDC],
            Chain::Optimism => vec![Self::OPTIMISM_ETH, Self::OPTIMISM_USDC, Self::OPTIMISM_USDT],
            Chain::Arbitrum => vec![Self::ARBITRUM_ETH, Self::ARBITRUM_USDC, Self::ARBITRUM_USDT],
            Chain::Polygon => vec![Self::POLYGON_USDC, Self::POLYGON_USDT],
            Chain::AvalancheC => vec![Self::AVALANCHE_USDC, Self::AVALANCHE_USDT],
            Chain::SmartChain => vec![Self::BNB_USDC, Self::BNB_USDT],
            Chain::Linea => vec![Self::LINEA_ETH],
            Chain::Mantle => vec![Self::MANTLE_ETH, Self::MANTLE_USDC, Self::MANTLE_USDT],
            _ => vec![],
        }
    }

    pub fn to_swap_chain_asset(&self) -> Result<SwapChainAsset, SwapperError> {
        let asset = match self {
            // Ethereum
            StargatePool::ETH_ETH => Some(SwapChainAsset::Assets(Chain::Ethereum, vec![ETHEREUM_WETH.id.clone()])),
            StargatePool::ETH_USDC => Some(SwapChainAsset::Assets(Chain::Ethereum, vec![ETHEREUM_USDC.id.clone()])),
            StargatePool::ETH_USDT => Some(SwapChainAsset::Assets(Chain::Ethereum, vec![ETHEREUM_USDT.id.clone()])),

            // Base
            StargatePool::BASE_ETH => Some(SwapChainAsset::Assets(Chain::Base, vec![BASE_WETH.id.clone()])),
            StargatePool::BASE_USDC => Some(SwapChainAsset::Assets(Chain::Base, vec![BASE_USDC.id.clone()])),

            // Optimism
            StargatePool::OPTIMISM_ETH => Some(SwapChainAsset::Assets(Chain::Optimism, vec![OPTIMISM_WETH.id.clone()])),
            StargatePool::OPTIMISM_USDC => Some(SwapChainAsset::Assets(Chain::Optimism, vec![OPTIMISM_USDC.id.clone()])),
            StargatePool::OPTIMISM_USDT => Some(SwapChainAsset::Assets(Chain::Optimism, vec![OPTIMISM_USDT.id.clone()])),

            // Arbitrum
            StargatePool::ARBITRUM_ETH => Some(SwapChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_WETH.id.clone()])),
            StargatePool::ARBITRUM_USDC => Some(SwapChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_USDC.id.clone()])),
            StargatePool::ARBITRUM_USDT => Some(SwapChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_USDT.id.clone()])),

            // Polygon
            StargatePool::POLYGON_USDC => Some(SwapChainAsset::Assets(Chain::Polygon, vec![POLYGON_USDC.id.clone()])),
            StargatePool::POLYGON_USDT => Some(SwapChainAsset::Assets(Chain::Polygon, vec![POLYGON_USDT.id.clone()])),

            // Avalanche
            StargatePool::AVALANCHE_USDC => Some(SwapChainAsset::Assets(Chain::AvalancheC, vec![AVALANCHE_USDC.id.clone()])),
            StargatePool::AVALANCHE_USDT => Some(SwapChainAsset::Assets(Chain::AvalancheC, vec![AVALANCHE_USDT.id.clone()])),

            // BNB (SmartChain)
            StargatePool::BNB_USDC => Some(SwapChainAsset::Assets(Chain::SmartChain, vec![SMARTCHAIN_USDC.id.clone()])),
            StargatePool::BNB_USDT => Some(SwapChainAsset::Assets(Chain::SmartChain, vec![SMARTCHAIN_USDT.id.clone()])),

            // Linea
            StargatePool::LINEA_ETH => Some(SwapChainAsset::Assets(Chain::Linea, vec![LINEA_WETH.id.clone()])),

            // For chains/assets that are not yet supported in our asset constants
            StargatePool::METIS_ETH
            | StargatePool::METIS_USDT
            | StargatePool::SCROLL_ETH
            | StargatePool::SCROLL_USDC
            | StargatePool::MANTLE_ETH
            | StargatePool::MANTLE_USDC
            | StargatePool::MANTLE_USDT => None,
        };
        Ok(asset.ok_or(SwapperError::NotImplemented)?)
    }
}
