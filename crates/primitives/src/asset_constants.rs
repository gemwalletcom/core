use std::sync::LazyLock;

use crate::{AssetId, Chain};

pub const ARBITRUM_ACX_TOKEN_ID: &str = "0x53691596d1BCe8CEa565b84d4915e69e03d9C99d";
pub static ARBITRUM_ACX_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Arbitrum, ARBITRUM_ACX_TOKEN_ID));

pub const ETHEREUM_ACX_TOKEN_ID: &str = "0x44108f0223A3C3028F5Fe7AEC7f9bb2E66beF82F";
pub static ETHEREUM_ACX_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_ACX_TOKEN_ID));

pub const OPTIMISM_ACX_TOKEN_ID: &str = "0xFf733b2A3557a7ed6697007ab5D11B79FdD1b76B";
pub static OPTIMISM_ACX_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Optimism, OPTIMISM_ACX_TOKEN_ID));

pub const POLYGON_ACX_TOKEN_ID: &str = "0xF328b73B6c685831F238c30a23Fc19140CB4D8FC";
pub static POLYGON_ACX_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Polygon, POLYGON_ACX_TOKEN_ID));

pub const ARBITRUM_ARB_TOKEN_ID: &str = "0x912CE59144191C1204E64559FE8253a0e49E6548";
pub static ARBITRUM_ARB_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Arbitrum, ARBITRUM_ARB_TOKEN_ID));

pub const ETHEREUM_ARB_TOKEN_ID: &str = "0x44108f0223A3C3028F5Fe7AEC7f9bb2E66beF82F";
pub static ETHEREUM_ARB_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_ARB_TOKEN_ID));

pub const ARBITRUM_DAI_TOKEN_ID: &str = "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1";
pub static ARBITRUM_DAI_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Arbitrum, ARBITRUM_DAI_TOKEN_ID));

pub const BASE_DAI_TOKEN_ID: &str = "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1";
pub static BASE_DAI_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Base, BASE_DAI_TOKEN_ID));

pub const ETHEREUM_DAI_TOKEN_ID: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
pub static ETHEREUM_DAI_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_DAI_TOKEN_ID));

pub const LINEA_DAI_TOKEN_ID: &str = "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1";
pub static LINEA_DAI_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Linea, LINEA_DAI_TOKEN_ID));

pub const OPTIMISM_DAI_TOKEN_ID: &str = "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1";
pub static OPTIMISM_DAI_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Optimism, OPTIMISM_DAI_TOKEN_ID));

pub const POLYGON_DAI_TOKEN_ID: &str = "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063";
pub static POLYGON_DAI_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Polygon, POLYGON_DAI_TOKEN_ID));

pub const ZKSYNC_DAI_TOKEN_ID: &str = "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1";
pub static ZKSYNC_DAI_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::ZkSync, ZKSYNC_DAI_TOKEN_ID));

pub const UNICHAIN_DAI_TOKEN_ID: &str = "0x20CAb320A855b39F724131C69424240519573f81";
pub static UNICHAIN_DAI_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Unichain, UNICHAIN_DAI_TOKEN_ID));

pub const SMARTCHAIN_ETH_TOKEN_ID: &str = "0x2170Ed0880ac9A755fd29B2688956BD959F933F8";
pub static SMARTCHAIN_ETH_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::SmartChain, SMARTCHAIN_ETH_TOKEN_ID));

pub const ARBITRUM_USDC_TOKEN_ID: &str = "0xaf88d065e77c8cC2239327C5EDb3A432268e5831";
pub static ARBITRUM_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Arbitrum, ARBITRUM_USDC_TOKEN_ID));

pub const BASE_USDC_TOKEN_ID: &str = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";
pub static BASE_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Base, BASE_USDC_TOKEN_ID));

pub const ETHEREUM_USDC_TOKEN_ID: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
pub static ETHEREUM_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID));

pub const OPTIMISM_USDC_TOKEN_ID: &str = "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85";
pub static OPTIMISM_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Optimism, OPTIMISM_USDC_TOKEN_ID));

pub const POLYGON_USDC_TOKEN_ID: &str = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359";
pub static POLYGON_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Polygon, POLYGON_USDC_TOKEN_ID));

pub const GNOSIS_USDC_TOKEN_ID: &str = "0x2a22f9c3b484c3629090FeED35F17Ff8F88f76F0";
pub static GNOSIS_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Gnosis, GNOSIS_USDC_TOKEN_ID));

pub const SOLANA_USDC_TOKEN_ID: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub static SOLANA_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Solana, SOLANA_USDC_TOKEN_ID));

pub const NEAR_USDC_TOKEN_ID: &str = "17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2011e36133a1";
pub static NEAR_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Near, NEAR_USDC_TOKEN_ID));

pub const UNICHAIN_USDC_TOKEN_ID: &str = "0x078D782b760474a361dDA0AF3839290b0EF57AD6";
pub static UNICHAIN_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Unichain, UNICHAIN_USDC_TOKEN_ID));

pub const HYPEREVM_USDC_TOKEN_ID: &str = "0xb88339CB7199b77E23DB6E890353E22632Ba630f";
pub static HYPEREVM_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Hyperliquid, HYPEREVM_USDC_TOKEN_ID));

pub const MONAD_USDC_TOKEN_ID: &str = "0x754704Bc059F8C67012fEd69BC8A327a5aafb603";
pub static MONAD_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Monad, MONAD_USDC_TOKEN_ID));

pub const SMARTCHAIN_USDC_TOKEN_ID: &str = "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d";
pub static SMARTCHAIN_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDC_TOKEN_ID));

pub const AVALANCHE_USDC_TOKEN_ID: &str = "0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E";
pub static AVALANCHE_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::AvalancheC, AVALANCHE_USDC_TOKEN_ID));

pub const SUI_USDC_TOKEN_ID: &str = "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC";
pub static SUI_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Sui, SUI_USDC_TOKEN_ID));

pub const ARBITRUM_USDC_E_TOKEN_ID: &str = "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8";
pub static ARBITRUM_USDC_E_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Arbitrum, ARBITRUM_USDC_E_TOKEN_ID));

pub const BASE_USDC_E_TOKEN_ID: &str = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";
pub static BASE_USDC_E_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Base, BASE_USDC_E_TOKEN_ID));

pub const ETHEREUM_USDC_E_TOKEN_ID: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
pub static ETHEREUM_USDC_E_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_E_TOKEN_ID));

pub const LINEA_USDC_E_TOKEN_ID: &str = "0x176211869cA2b568f2A7D4EE941E073a821EE1ff";
pub static LINEA_USDC_E_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Linea, LINEA_USDC_E_TOKEN_ID));

pub const OPTIMISM_USDC_E_TOKEN_ID: &str = "0x7F5c764cBc14f9669B88837ca1490cCa17c31607";
pub static OPTIMISM_USDC_E_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Optimism, OPTIMISM_USDC_E_TOKEN_ID));

pub const POLYGON_USDC_E_TOKEN_ID: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
pub static POLYGON_USDC_E_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Polygon, POLYGON_USDC_E_TOKEN_ID));

pub const WORLD_USDC_E_TOKEN_ID: &str = "0x79A02482A880bCE3F13e09Da970dC34db4CD24d1";
pub static WORLD_USDC_E_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::World, WORLD_USDC_E_TOKEN_ID));

pub const ZKSYNC_USDC_E_TOKEN_ID: &str = "0x3355df6D4c9C3035724Fd0e3914dE96A5a83aaf4";
pub static ZKSYNC_USDC_E_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::ZkSync, ZKSYNC_USDC_E_TOKEN_ID));

pub const ARBITRUM_USDT_TOKEN_ID: &str = "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9";
pub static ARBITRUM_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Arbitrum, ARBITRUM_USDT_TOKEN_ID));

pub const ETHEREUM_USDT_TOKEN_ID: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
pub static ETHEREUM_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID));

pub const LINEA_USDT_TOKEN_ID: &str = "0xA219439258ca9da29E9Cc4cE5596924745e12B93";
pub static LINEA_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Linea, LINEA_USDT_TOKEN_ID));

pub const OPTIMISM_USDT_TOKEN_ID: &str = "0x94b008aA00579c1307B0EF2c499aD98a8ce58e58";
pub static OPTIMISM_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Optimism, OPTIMISM_USDT_TOKEN_ID));

pub const POLYGON_USDT_TOKEN_ID: &str = "0xc2132D05D31c914a87C6611C10748AEb04B58e8F";
pub static POLYGON_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Polygon, POLYGON_USDT_TOKEN_ID));

pub const ZKSYNC_USDT_TOKEN_ID: &str = "0x493257fD37EDB34451f62EDf8D2a0C418852bA4C";
pub static ZKSYNC_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::ZkSync, ZKSYNC_USDT_TOKEN_ID));

pub const SMARTCHAIN_USDT_TOKEN_ID: &str = "0x55d398326f99059fF775485246999027B3197955";
pub static SMARTCHAIN_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDT_TOKEN_ID));

pub const AVALANCHE_USDT_TOKEN_ID: &str = "0x9702230A8Ea53601f5cD2dc00fDBc13d4dF4A8c7";
pub static AVALANCHE_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::AvalancheC, AVALANCHE_USDT_TOKEN_ID));

pub const SOLANA_USDT_TOKEN_ID: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
pub static SOLANA_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Solana, SOLANA_USDT_TOKEN_ID));

pub const SOLANA_PYUSD_TOKEN_ID: &str = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";
pub static SOLANA_PYUSD_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Solana, SOLANA_PYUSD_TOKEN_ID));

pub const TRON_USDT_TOKEN_ID: &str = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";
pub static TRON_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID));

pub const TON_USDT_TOKEN_ID: &str = "EQcxE6MuTQJkfNGfAARoTKOVt1LZBADiix1KCixRv7NW2id_sDs";
pub static TON_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ton, TON_USDT_TOKEN_ID));

pub const NEAR_USDT_TOKEN_ID: &str = "usdt.tether-token.near";
pub static NEAR_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Near, NEAR_USDT_TOKEN_ID));

pub const INK_USDT_TOKEN_ID: &str = "0x3baD7AD0728f9917d1Bf08af5782dCbD516cDd96";
pub static INK_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ink, INK_USDT_TOKEN_ID));

pub const HYPEREVM_USDT_TOKEN_ID: &str = "0xB8CE59FC3717ada4C02eaDF9682A9e934F625ebb";
pub static HYPEREVM_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Hyperliquid, HYPEREVM_USDT_TOKEN_ID));

pub const PLASMA_USDT_TOKEN_ID: &str = "0xB8CE59FC3717ada4C02eaDF9682A9e934F625ebb";
pub static PLASMA_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Plasma, PLASMA_USDT_TOKEN_ID));

pub const MONAD_USDT_TOKEN_ID: &str = "0xe7cd86e13AC4309349F30B3435a9d337750fC82D";
pub static MONAD_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Monad, MONAD_USDT_TOKEN_ID));

pub const APTOS_USDT_TOKEN_ID: &str = "0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b";
pub static APTOS_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Aptos, APTOS_USDT_TOKEN_ID));

pub const ARBITRUM_WBTC_TOKEN_ID: &str = "0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f";
pub static ARBITRUM_WBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Arbitrum, ARBITRUM_WBTC_TOKEN_ID));

pub const BLAST_WBTC_TOKEN_ID: &str = "0xF7bc58b8D8f97ADC129cfC4c9f45Ce3C0E1D2692";
pub static BLAST_WBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Blast, BLAST_WBTC_TOKEN_ID));

pub const ETHEREUM_WBTC_TOKEN_ID: &str = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599";
pub static ETHEREUM_WBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_WBTC_TOKEN_ID));

pub const LINEA_WBTC_TOKEN_ID: &str = "0x3aAB2285ddcDdaD8edf438C1bAB47e1a9D05a9b4";
pub static LINEA_WBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Linea, LINEA_WBTC_TOKEN_ID));

pub const OPTIMISM_WBTC_TOKEN_ID: &str = "0x68f180fcCe6836688e9084f035309E29Bf0A2095";
pub static OPTIMISM_WBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Optimism, OPTIMISM_WBTC_TOKEN_ID));

pub const POLYGON_WBTC_TOKEN_ID: &str = "0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6";
pub static POLYGON_WBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Polygon, POLYGON_WBTC_TOKEN_ID));

pub const WORLD_WBTC_TOKEN_ID: &str = "0x03C7054BCB39f7b2e5B2c7AcB37583e32D70Cfa3";
pub static WORLD_WBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::World, WORLD_WBTC_TOKEN_ID));

pub const ZKSYNC_WBTC_TOKEN_ID: &str = "0xBBeB516fb02a01611cBBE0453Fe3c580D7281011";
pub static ZKSYNC_WBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::ZkSync, ZKSYNC_WBTC_TOKEN_ID));

pub const ARBITRUM_WETH_TOKEN_ID: &str = "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1";
pub static ARBITRUM_WETH_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Arbitrum, ARBITRUM_WETH_TOKEN_ID));

pub const BLAST_WETH_TOKEN_ID: &str = "0x4300000000000000000000000000000000000004";
pub static BLAST_WETH_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Blast, BLAST_WETH_TOKEN_ID));

pub const ETHEREUM_WETH_TOKEN_ID: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
pub static ETHEREUM_WETH_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_WETH_TOKEN_ID));

pub const LINEA_WETH_TOKEN_ID: &str = "0xe5D7C2a44FfDDf6b295A15c148167daaAf5Cf34f";
pub static LINEA_WETH_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Linea, LINEA_WETH_TOKEN_ID));

pub const BASE_WETH_TOKEN_ID: &str = "0x4200000000000000000000000000000000000006";
pub static BASE_WETH_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Base, BASE_WETH_TOKEN_ID));

pub const OPTIMISM_WETH_TOKEN_ID: &str = "0x4200000000000000000000000000000000000006";
pub static OPTIMISM_WETH_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Optimism, OPTIMISM_WETH_TOKEN_ID));

pub const POLYGON_WETH_TOKEN_ID: &str = "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619";
pub static POLYGON_WETH_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Polygon, POLYGON_WETH_TOKEN_ID));

pub const WORLD_WETH_TOKEN_ID: &str = "0x4200000000000000000000000000000000000006";
pub static WORLD_WETH_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::World, WORLD_WETH_TOKEN_ID));

pub const INK_WETH_TOKEN_ID: &str = "0x4200000000000000000000000000000000000006";
pub static INK_WETH_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ink, INK_WETH_TOKEN_ID));

pub const ZKSYNC_WETH_TOKEN_ID: &str = "0x5AEa5775959fBC2557Cc8789bC1bf90A239D9a91";
pub static ZKSYNC_WETH_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::ZkSync, ZKSYNC_WETH_TOKEN_ID));

pub const UNICHAIN_WETH_TOKEN_ID: &str = "0x4200000000000000000000000000000000000006";
pub static UNICHAIN_WETH_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Unichain, UNICHAIN_WETH_TOKEN_ID));

pub const BASE_CBBTC_TOKEN_ID: &str = "0xcbB7C0000aB88B473b1f5aFd9ef808440eed33Bf";
pub static BASE_CBBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Base, BASE_CBBTC_TOKEN_ID));

pub const ETHEREUM_CBBTC_TOKEN_ID: &str = "0xcbB7C0000aB88B473b1f5aFd9ef808440eed33Bf";
pub static ETHEREUM_CBBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_CBBTC_TOKEN_ID));

pub const ETHEREUM_LINK_TOKEN_ID: &str = "0x514910771AF9Ca656af840dff83E8264EcF986CA";
pub static ETHEREUM_LINK_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_LINK_TOKEN_ID));

pub const ETHEREUM_UNI_TOKEN_ID: &str = "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984";
pub static ETHEREUM_UNI_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_UNI_TOKEN_ID));

pub const ETHEREUM_AAVE_TOKEN_ID: &str = "0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9";
pub static ETHEREUM_AAVE_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_AAVE_TOKEN_ID));

pub const OPTIMISM_OP_TOKEN_ID: &str = "0x4200000000000000000000000000000000000042";
pub static OPTIMISM_OP_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Optimism, OPTIMISM_OP_TOKEN_ID));

pub const BERACHAIN_USDT_TOKEN_ID: &str = "0x779Ded0c9e1022225f8e0630b35a9b54be713736";
pub static BERACHAIN_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Berachain, BERACHAIN_USDT_TOKEN_ID));

pub const GNOSIS_USDT_TOKEN_ID: &str = "0x4ECaBa5870353805a9F068101A40E0f32ed605C6";
pub static GNOSIS_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Gnosis, GNOSIS_USDT_TOKEN_ID));

pub const STELLAR_USDC_TOKEN_ID: &str = "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN::USDC";
pub static STELLAR_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Stellar, STELLAR_USDC_TOKEN_ID));

pub const XLAYER_USDC_TOKEN_ID: &str = "0x74b7f16337b8972027F6196A17a631aC6dE26d22";
pub static XLAYER_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::XLayer, XLAYER_USDC_TOKEN_ID));

pub const XLAYER_USDT_TOKEN_ID: &str = "0x779Ded0c9e1022225f8e0630b35a9b54be713736";
pub static XLAYER_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::XLayer, XLAYER_USDT_TOKEN_ID));

pub const ETHEREUM_STETH_TOKEN_ID: &str = "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84";
pub static ETHEREUM_STETH_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_STETH_TOKEN_ID));

pub const ETHEREUM_USDS_TOKEN_ID: &str = "0xdC035D45d973E3EC169d2276DDab16f1e407384F";
pub static ETHEREUM_USDS_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_USDS_TOKEN_ID));

pub const ETHEREUM_FLIP_TOKEN_ID: &str = "0x826180541412D574cf1336d22c0C0a287822678A";
pub static ETHEREUM_FLIP_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Ethereum, ETHEREUM_FLIP_TOKEN_ID));

pub const BASE_USDS_TOKEN_ID: &str = "0x820C137fa70C8691f0e44Dc420a5e53c168921Dc";
pub static BASE_USDS_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Base, BASE_USDS_TOKEN_ID));

pub const BASE_WBTC_TOKEN_ID: &str = "0x0555E30da8f98308EdB960aa94C0Db47230d2B9c";
pub static BASE_WBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Base, BASE_WBTC_TOKEN_ID));

pub const SMARTCHAIN_WBTC_TOKEN_ID: &str = "0x0555E30da8f98308EdB960aa94C0Db47230d2B9c";
pub static SMARTCHAIN_WBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::SmartChain, SMARTCHAIN_WBTC_TOKEN_ID));

pub const SOLANA_USDS_TOKEN_ID: &str = "USDSwr9ApdHk5bvJKMjzff41FfuX8bSxdKcR81vTwcA";
pub static SOLANA_USDS_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Solana, SOLANA_USDS_TOKEN_ID));

pub const SOLANA_WBTC_TOKEN_ID: &str = "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh";
pub static SOLANA_WBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Solana, SOLANA_WBTC_TOKEN_ID));

pub const SOLANA_CBBTC_TOKEN_ID: &str = "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij";
pub static SOLANA_CBBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Solana, SOLANA_CBBTC_TOKEN_ID));

pub const SOLANA_JITO_SOL_TOKEN_ID: &str = "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn";
pub static SOLANA_JITO_SOL_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Solana, SOLANA_JITO_SOL_TOKEN_ID));

pub const HYPERCORE_SPOT_HYPE_TOKEN_ID: &str = "HYPE::0x0d01dc56dcaaca66ad901c959b4011ec::150";
pub static HYPERCORE_SPOT_HYPE_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::HyperCore, HYPERCORE_SPOT_HYPE_TOKEN_ID));

pub const HYPERCORE_SPOT_USDC_TOKEN_ID: &str = "USDC::0x6d1e7cde53ba9467b783cb7c530ce054::0";
pub static HYPERCORE_SPOT_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::HyperCore, HYPERCORE_SPOT_USDC_TOKEN_ID));

pub const HYPERCORE_SPOT_UBTC_TOKEN_ID: &str = "UBTC::0x8f254b963e8468305d409b33aa137c67::197";
pub static HYPERCORE_SPOT_UBTC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::HyperCore, HYPERCORE_SPOT_UBTC_TOKEN_ID));

pub const HYPERCORE_CORE_HYPE_TOKEN_ID: &str = "HYPE:0x0d01dc56dcaaca66ad901c959b4011ec";

pub const SUI_WAL_TOKEN_ID: &str = "0x356a26eb9e012a68958082340d4c4116e7f55615cf27affcff209cf0ae544f59::wal::WAL";
pub static SUI_WAL_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Sui, SUI_WAL_TOKEN_ID));

pub const SUI_SBUSDT_TOKEN_ID: &str = "0x375f70cf2ae4c00bf37117d0c85a2c71545e6ee05c4a5c7d282cd66a4504b068::usdt::USDT";
pub static SUI_SBUSDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Sui, SUI_SBUSDT_TOKEN_ID));

pub const THORCHAIN_TCY_TOKEN_ID: &str = "tcy";
pub static THORCHAIN_TCY_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Thorchain, THORCHAIN_TCY_TOKEN_ID));

pub const COSMOS_USDC_TOKEN_ID: &str = "ibc/F663521BF1836B00F5F177680F74BFB9A8B5654A694D0D2BC249E03CF2509013";
pub static COSMOS_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Cosmos, COSMOS_USDC_TOKEN_ID));

pub const OSMOSIS_USDC_TOKEN_ID: &str = "ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4";
pub static OSMOSIS_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Osmosis, OSMOSIS_USDC_TOKEN_ID));

pub const OSMOSIS_USDT_TOKEN_ID: &str = "ibc/4ABBEF4C8926DDDB320AE5188CFD63267ABBCEFC0583E4AE05D6E5AA2401DDAB";
pub static OSMOSIS_USDT_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Osmosis, OSMOSIS_USDT_TOKEN_ID));

pub const INJECTIVE_USDC_TOKEN_ID: &str = "ibc/7E1AF94AD246BE522892751046F0C959B768642E5671CC3742264068D49553C0";
pub static INJECTIVE_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Injective, INJECTIVE_USDC_TOKEN_ID));

pub const SEI_USDC_TOKEN_ID: &str = "ibc/CA6FBFAF399474A06263E10D0CE5AEBBE15189D6D4B2DD9ADE61007E68EB9DB0";
pub static SEI_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from_token(Chain::Sei, SEI_USDC_TOKEN_ID));
