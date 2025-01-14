use lazy_static::lazy_static;
use primitives::{AssetId, Chain};

use crate::swapper::{
    asset::{
        ARBITRUM_USDC, ARBITRUM_USDT, ARBITRUM_WETH, AVALANCHE_USDC, AVALANCHE_USDT, BASE_USDC, BASE_WETH, ETHEREUM_METH, ETHEREUM_USDC, ETHEREUM_USDT,
        ETHEREUM_WETH, LINEA_WETH, MANTLE_USDC, MANTLE_USDT, OPTIMISM_USDC, OPTIMISM_USDT, OPTIMISM_WETH, POLYGON_USDC, POLYGON_USDT, SEI_USDC, SEI_USDT,
        SMARTCHAIN_USDC, SMARTCHAIN_USDT,
    },
    SwapChainAsset, SwapperError,
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
    pub sei: StargateEndpoint,
    //pub metis: StargateEndpoint,
    //pub scroll: StargateEndpoint,
    pub mantle: StargateEndpoint,
    //pub kava: StargateEndpoint,
    //pub aurora: StargateEndpoint,
    //pub core: StargateEndpoint,
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
                StargatePool {
                    asset: ETHEREUM_METH.id.clone(),
                    address: "0xd5f7838f5c461feff7fe49ea5ebaf7728bb0adfa".to_string(),
                },
            ],
            endpoint_id: 30101,
            chain_asset: SwapChainAsset::Assets(
                Chain::Ethereum,
                vec![AssetId::from_chain(Chain::Ethereum), ETHEREUM_USDC.id.clone(), ETHEREUM_USDT.id.clone(), ETHEREUM_METH.id.clone(),]
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
            chain_asset: SwapChainAsset::Assets(
                Chain::Optimism,
                vec![OPTIMISM_USDC.id.clone(), OPTIMISM_USDT.id.clone(),]
            ),
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
            chain_asset: SwapChainAsset::Assets(
                Chain::Arbitrum,
                vec![ARBITRUM_USDC.id.clone(), ARBITRUM_USDT.id.clone(),]
            ),
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
        sei: StargateEndpoint {
            id: Chain::Sei,
            pools: vec![
                StargatePool {
                    asset: AssetId::from_chain(Chain::Sei),
                    address: "0x5c386D85b1B82FD9Db681b9176C8a4248bb6345B".to_string(),
                },
                StargatePool {
                    asset: SEI_USDC.id.clone(),
                    address: "0x45d417612e177672958dC0537C45a8f8d754Ac2E".to_string(),
                },
                StargatePool {
                    asset: SEI_USDT.id.clone(),
                    address: "0x0dB9afb4C33be43a0a0e396Fd1383B4ea97aB10a".to_string(),
                },
            ],
            endpoint_id: 30280,
            chain_asset: SwapChainAsset::Assets(Chain::Sei, vec![
                SEI_USDC.id.clone(),
                SEI_USDT.id.clone(),
            ]),
        },

        //metis: StargateEndpoint {
        //    id: Chain::Metis,
        //    pools: vec![
        //        StargatePool {
        //            asset: AssetId::from_chain(Chain::Metis),
        //            address: "0x36ed193dc7160D3858EC250e69D12B03Ca087D08".to_string(),
        //        },
        //        StargatePool {
        //            asset: AssetId::from_token(Chain::Metis, "0x0"),
        //            address: "0x4dCBFC0249e8d5032F89D6461218a9D2eFff5125".to_string(),
        //        },
        //    ],
        //    endpoint_id: 30151,
        //    chain_asset: SwapChainAsset::Assets(Chain::Metis, vec![]),
        //},
        //scroll: StargateEndpoint {
        //    id: Chain::Scroll,
        //    pools: vec![
        //        StargatePool {
        //            asset: AssetId::from_chain(Chain::Scroll),
        //            address: "0xC2b638Cb5042c1B3c5d5C969361fB50569840583".to_string(),
        //        },
        //        StargatePool {
        //            asset: AssetId::from_token(Chain::Scroll, "0x0"),
        //            address: "0x3Fc69CC4A842838bCDC9499178740226062b14E4".to_string(),
        //        },
        //    ],
        //    endpoint_id: 30214,
        //    chain_asset: SwapChainAsset::Assets(Chain::Scroll, vec![]),
        //},
        mantle: StargateEndpoint {
            id: Chain::Mantle,
            pools: vec![
                StargatePool {
                    asset: AssetId::from_chain(Chain::Mantle),
                    address: "0x4c1d3Fc3fC3c177c3b633427c2F769276c547463".to_string(),
                },
                StargatePool {
                    asset: MANTLE_USDC.id.clone(),
                    address: "0xAc290Ad4e0c891FDc295ca4F0a6214cf6dC6acDC".to_string(),
                },
                StargatePool {
                    asset: MANTLE_USDT.id.clone(),
                    address: "0xB715B85682B731dB9D5063187C450095c91C57FC".to_string(),
                },
            ],
            endpoint_id: 30181,
            chain_asset: SwapChainAsset::Assets(Chain::Mantle, vec![
                AssetId::from_chain(Chain::Mantle),
                MANTLE_USDC.id.clone(),
                MANTLE_USDT.id.clone(),
            ]),
        },
        //kava: StargateEndpoint {
        //    id: Chain::Kava,
        //    pools: vec![StargatePool {
        //        asset: AssetId::from_token(Chain::Kava, "0x0"),
        //        address: "0x41A5b0470D96656Fb3e8f68A218b39AdBca3420b".to_string(),
        //    },],
        //    endpoint_id: 30177,
        //    chain_asset: SwapChainAsset::Assets(Chain::Kava, vec![]),
        //},
        //aurora: StargateEndpoint {
        //    id: Chain::Aurora,
        //    pools: vec![StargatePool {
        //        asset: AssetId::from_token(Chain::Aurora, "0x0"),
        //        address: "0x81F6138153d473E8c5EcebD3DC8Cd4903506B075".to_string(),
        //    },],
        //    endpoint_id: 30211,
        //    chain_asset: SwapChainAsset::Assets(Chain::Aurora, vec![]),
        //},
        //core: StargateEndpoint {
        //    id: Chain::Core,
        //    pools: vec![
        //        StargatePool {
        //            asset: AssetId::from_token(Chain::Core, "0x0"),
        //            address: "0x2F6F07CDcf3588944Bf4C42aC74ff24bF56e7590".to_string(),
        //        },
        //        StargatePool {
        //            asset: AssetId::from_token(Chain::Core, "0x0"),
        //            address: "0x45f1A95A4D3f3836523F5c83673c797f4d4d263B".to_string(),
        //        },
        //    ],
        //    endpoint_id: 30153,
        //    chain_asset: SwapChainAsset::Assets(Chain::Core, vec![]),
        //},
    };
}
