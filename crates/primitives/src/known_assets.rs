use std::sync::LazyLock;

use crate::{Asset, AssetId, AssetType, Chain, asset_constants::*};

const USDT_NAME: &str = "Tether";
const USDT_SYMBOL: &str = "USDT";
const USDC_NAME: &str = "USDC";
const USDC_SYMBOL: &str = "USDC";
const WBTC_SYMBOL: &str = "WBTC";
const WBTC_NAME: &str = "Wrapped BTC";
const DAI_NAME: &str = "Dai Stablecoin";
const DAI_SYMBOL: &str = "DAI";
const WETH_NAME: &str = "Wrapped Ether";
const WETH_SYMBOL: &str = "WETH";
const CBBTC_NAME: &str = "Coinbase BTC";
const CBBTC_SYMBOL: &str = "cbBTC";
const USDS_NAME: &str = "USDS Stablecoin";
const USDS_SYMBOL: &str = "USDS";

fn token_asset(chain: Chain, token_id: &str, name: &str, symbol: &str, decimals: i32, asset_type: AssetType) -> Asset {
    Asset::new(AssetId::from_token(chain, token_id), name.to_string(), symbol.to_string(), decimals, asset_type)
}

pub static ETHEREUM_USDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID, USDT_NAME, USDT_SYMBOL, 6, AssetType::ERC20));
pub static ETHEREUM_USDC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID, USDC_NAME, USDC_SYMBOL, 6, AssetType::ERC20));
pub static ETHEREUM_WBTC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Ethereum, ETHEREUM_WBTC_TOKEN_ID, WBTC_NAME, WBTC_SYMBOL, 8, AssetType::ERC20));
pub static ETHEREUM_DAI: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Ethereum, ETHEREUM_DAI_TOKEN_ID, DAI_NAME, DAI_SYMBOL, 18, AssetType::ERC20));
pub static ETHEREUM_WETH: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Ethereum, ETHEREUM_WETH_TOKEN_ID, WETH_NAME, WETH_SYMBOL, 18, AssetType::ERC20));
pub static ETHEREUM_USDS: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Ethereum, ETHEREUM_USDS_TOKEN_ID, USDS_NAME, USDS_SYMBOL, 18, AssetType::ERC20));
pub static ETHEREUM_STETH: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Ethereum, ETHEREUM_STETH_TOKEN_ID, "stETH", "stETH", 18, AssetType::ERC20));
pub static ETHEREUM_CBBTC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Ethereum, ETHEREUM_CBBTC_TOKEN_ID, CBBTC_NAME, CBBTC_SYMBOL, 8, AssetType::ERC20));
pub static ETHEREUM_FLIP: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Ethereum, ETHEREUM_FLIP_TOKEN_ID, "Chainflip", "FLIP", 18, AssetType::ERC20));

pub static ARBITRUM_WETH: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Arbitrum, ARBITRUM_WETH_TOKEN_ID, WETH_NAME, WETH_SYMBOL, 18, AssetType::ERC20));
pub static ARBITRUM_USDC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Arbitrum, ARBITRUM_USDC_TOKEN_ID, USDC_NAME, USDC_SYMBOL, 6, AssetType::ERC20));
pub static ARBITRUM_USDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Arbitrum, ARBITRUM_USDT_TOKEN_ID, USDT_NAME, USDT_SYMBOL, 6, AssetType::ERC20));

pub static BASE_WETH: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Base, BASE_WETH_TOKEN_ID, WETH_NAME, WETH_SYMBOL, 18, AssetType::ERC20));
pub static BASE_USDC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Base, BASE_USDC_TOKEN_ID, USDC_NAME, USDC_SYMBOL, 6, AssetType::ERC20));
pub static BASE_CBBTC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Base, BASE_CBBTC_TOKEN_ID, CBBTC_NAME, CBBTC_SYMBOL, 8, AssetType::ERC20));
pub static BASE_USDS: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Base, BASE_USDS_TOKEN_ID, USDS_NAME, USDS_SYMBOL, 18, AssetType::ERC20));
pub static BASE_WBTC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Base, BASE_WBTC_TOKEN_ID, WBTC_NAME, WBTC_SYMBOL, 8, AssetType::ERC20));

pub static BLAST_WETH: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Blast, BLAST_WETH_TOKEN_ID, WETH_NAME, WETH_SYMBOL, 18, AssetType::ERC20));

pub static LINEA_WETH: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Linea, LINEA_WETH_TOKEN_ID, WETH_NAME, WETH_SYMBOL, 18, AssetType::ERC20));
pub static LINEA_USDC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Linea, LINEA_USDC_E_TOKEN_ID, USDC_NAME, USDC_SYMBOL, 6, AssetType::ERC20));
pub static LINEA_USDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Linea, LINEA_USDT_TOKEN_ID, USDT_NAME, USDT_SYMBOL, 6, AssetType::ERC20));

pub static OPTIMISM_WETH: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Optimism, OPTIMISM_WETH_TOKEN_ID, WETH_NAME, WETH_SYMBOL, 18, AssetType::ERC20));
pub static OPTIMISM_USDC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Optimism, OPTIMISM_USDC_TOKEN_ID, USDC_NAME, USDC_SYMBOL, 6, AssetType::ERC20));
pub static OPTIMISM_USDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Optimism, OPTIMISM_USDT_TOKEN_ID, USDT_NAME, USDT_SYMBOL, 6, AssetType::ERC20));

pub static POLYGON_WETH: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Polygon, POLYGON_WETH_TOKEN_ID, WETH_NAME, WETH_SYMBOL, 18, AssetType::ERC20));
pub static POLYGON_USDC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Polygon, POLYGON_USDC_TOKEN_ID, USDC_NAME, USDC_SYMBOL, 6, AssetType::ERC20));
pub static POLYGON_USDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Polygon, POLYGON_USDT_TOKEN_ID, USDT_NAME, USDT_SYMBOL, 6, AssetType::ERC20));

pub static ZKSYNC_WETH: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::ZkSync, ZKSYNC_WETH_TOKEN_ID, WETH_NAME, WETH_SYMBOL, 18, AssetType::ERC20));
pub static ZKSYNC_USDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::ZkSync, ZKSYNC_USDT_TOKEN_ID, USDT_NAME, USDT_SYMBOL, 6, AssetType::ERC20));

pub static WORLD_WETH: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::World, WORLD_WETH_TOKEN_ID, WETH_NAME, WETH_SYMBOL, 18, AssetType::ERC20));

pub static SMARTCHAIN_ETH: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::SmartChain, SMARTCHAIN_ETH_TOKEN_ID, "Binance-Peg Ethereum", "ETH", 18, AssetType::ERC20));
pub static SMARTCHAIN_USDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::SmartChain, SMARTCHAIN_USDT_TOKEN_ID, USDT_NAME, USDT_SYMBOL, 18, AssetType::BEP20));
pub static SMARTCHAIN_USDC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::SmartChain, SMARTCHAIN_USDC_TOKEN_ID, USDC_NAME, USDC_SYMBOL, 18, AssetType::BEP20));
pub static SMARTCHAIN_WBTC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::SmartChain, SMARTCHAIN_WBTC_TOKEN_ID, WBTC_NAME, WBTC_SYMBOL, 8, AssetType::BEP20));

pub static AVALANCHE_USDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::AvalancheC, AVALANCHE_USDT_TOKEN_ID, USDT_NAME, USDT_SYMBOL, 6, AssetType::ERC20));
pub static AVALANCHE_USDC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::AvalancheC, AVALANCHE_USDC_TOKEN_ID, USDC_NAME, USDC_SYMBOL, 6, AssetType::ERC20));

pub static INK_WETH: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Ink, INK_WETH_TOKEN_ID, WETH_NAME, WETH_SYMBOL, 18, AssetType::ERC20));
pub static INK_USDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Ink, INK_USDT_TOKEN_ID, USDT_NAME, USDT_SYMBOL, 6, AssetType::ERC20));

pub static UNICHAIN_WETH: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Unichain, UNICHAIN_WETH_TOKEN_ID, WETH_NAME, WETH_SYMBOL, 18, AssetType::ERC20));
pub static UNICHAIN_USDC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Unichain, UNICHAIN_USDC_TOKEN_ID, USDC_NAME, USDC_SYMBOL, 6, AssetType::ERC20));
pub static UNICHAIN_DAI: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Unichain, UNICHAIN_DAI_TOKEN_ID, DAI_NAME, DAI_SYMBOL, 18, AssetType::ERC20));

pub static MONAD_MON: LazyLock<Asset> = LazyLock::new(|| Asset::from_chain(Chain::Monad));
pub static MONAD_USDC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Monad, MONAD_USDC_TOKEN_ID, USDC_NAME, USDC_SYMBOL, 6, AssetType::ERC20));
pub static MONAD_USDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Monad, MONAD_USDT_TOKEN_ID, USDT_NAME, USDT_SYMBOL, 6, AssetType::ERC20));

pub static HYPERCORE_HYPE: LazyLock<Asset> = LazyLock::new(|| Asset::from_chain(Chain::HyperCore));
pub static HYPERCORE_SPOT_HYPE: LazyLock<Asset> =
    LazyLock::new(|| Asset::new(HYPERCORE_SPOT_HYPE_ASSET_ID.clone(), "Hyperliquid".to_string(), "HYPE".to_string(), 8, AssetType::TOKEN));
pub static HYPERCORE_SPOT_USDC: LazyLock<Asset> =
    LazyLock::new(|| Asset::new(HYPERCORE_SPOT_USDC_ASSET_ID.clone(), USDC_NAME.to_string(), USDC_SYMBOL.to_string(), 8, AssetType::TOKEN));
pub static HYPERCORE_SPOT_UBTC: LazyLock<Asset> =
    LazyLock::new(|| Asset::new(HYPERCORE_SPOT_UBTC_ASSET_ID.clone(), "Bitcoin".to_string(), "UBTC".to_string(), 10, AssetType::TOKEN));

pub static HYPEREVM_HYPE: LazyLock<Asset> = LazyLock::new(|| Asset::from_chain(Chain::Hyperliquid));
pub static HYPEREVM_USDC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Hyperliquid, HYPEREVM_USDC_TOKEN_ID, USDC_NAME, USDC_SYMBOL, 6, AssetType::ERC20));
pub static HYPEREVM_USDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Hyperliquid, HYPEREVM_USDT_TOKEN_ID, USDT_NAME, USDT_SYMBOL, 6, AssetType::ERC20));

pub static PLASMA_USDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Plasma, PLASMA_USDT_TOKEN_ID, USDT_NAME, USDT_SYMBOL, 6, AssetType::ERC20));

pub static SOLANA_USDC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Solana, SOLANA_USDC_TOKEN_ID, USDC_NAME, USDC_SYMBOL, 6, AssetType::SPL));
pub static SOLANA_USDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Solana, SOLANA_USDT_TOKEN_ID, USDT_NAME, USDT_SYMBOL, 6, AssetType::SPL));
pub static SOLANA_USDS: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Solana, SOLANA_USDS_TOKEN_ID, USDS_NAME, USDS_SYMBOL, 6, AssetType::SPL));
pub static SOLANA_WBTC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Solana, SOLANA_WBTC_TOKEN_ID, WBTC_NAME, WBTC_SYMBOL, 8, AssetType::SPL));
pub static SOLANA_CBBTC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Solana, SOLANA_CBBTC_TOKEN_ID, CBBTC_NAME, CBBTC_SYMBOL, 8, AssetType::SPL));
pub static SOLANA_JITO_SOL: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Solana, SOLANA_JITO_SOL_TOKEN_ID, "Jito Staked SOL", "JitoSOL", 9, AssetType::SPL));

pub static SUI_USDC: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Sui, SUI_USDC_TOKEN_ID, USDC_NAME, USDC_SYMBOL, 6, AssetType::TOKEN));
pub static SUI_SBUSDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Sui, SUI_SBUSDT_TOKEN_ID, "Sui Bridged USDT", "sbUSDT", 6, AssetType::TOKEN));
pub static SUI_WAL: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Sui, SUI_WAL_TOKEN_ID, "Walrus", "WAL", 9, AssetType::TOKEN));

pub static THORCHAIN_TCY: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Thorchain, THORCHAIN_TCY_TOKEN_ID, "TCY", "TCY", 8, AssetType::TOKEN));

pub static TRON_USDT: LazyLock<Asset> = LazyLock::new(|| token_asset(Chain::Tron, TRON_USDT_TOKEN_ID, USDT_NAME, USDT_SYMBOL, 6, AssetType::TRC20));
