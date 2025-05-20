use lazy_static::lazy_static;
use primitives::{asset_constants::*, Asset, AssetId, AssetType, Chain};

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

// TODO: merge into primitives::asset_constants
// Ethereum
pub const ETHEREUM_USDC_TOKEN_ID: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
pub const ETHEREUM_USDT_TOKEN_ID: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
pub const ETHEREUM_WBTC_TOKEN_ID: &str = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599";
pub const ETHEREUM_DAI_TOKEN_ID: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
pub const ETHEREUM_STETH_TOKEN_ID: &str = "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84";
pub const ETHEREUM_CBBTC_TOKEN_ID: &str = "0xcbB7C0000aB88B473b1f5aFd9ef808440eed33Bf";
pub const ETHEREUM_USDS_TOKEN_ID: &str = "0xdC035D45d973E3EC169d2276DDab16f1e407384F";
pub const ETHEREUM_FLIP_TOKEN_ID: &str = "0x826180541412D574cf1336d22c0C0a287822678A";
// SmartChain
pub const SMARTCHAIN_USDT_TOKEN_ID: &str = "0x55d398326f99059fF775485246999027B3197955";
pub const SMARTCHAIN_USDC_TOKEN_ID: &str = "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d";
pub const SMARTCHAIN_WBTC_TOKEN_ID: &str = "0x0555E30da8f98308EdB960aa94C0Db47230d2B9c";
// Avalanche
pub const AVALANCHE_USDT_TOKEN_ID: &str = "0x9702230A8Ea53601f5cD2dc00fDBc13d4dF4A8c7";
pub const AVALANCHE_USDC_TOKEN_ID: &str = "0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E";
// Base
pub const BASE_USDC_TOKEN_ID: &str = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";
pub const BASE_USDS_TOKEN_ID: &str = "0x820C137fa70C8691f0e44Dc420a5e53c168921Dc";
pub const BASE_CBBTC_TOKEN_ID: &str = "0xcbB7C0000aB88B473b1f5aFd9ef808440eed33Bf";
pub const BASE_WBTC_TOKEN_ID: &str = "0x0555E30da8f98308EdB960aa94C0Db47230d2B9c";
// Solana
pub const SOLANA_USDC_TOKEN_ID: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const SOLANA_USDT_TOKEN_ID: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
pub const SOLANA_USDS_TOKEN_ID: &str = "USDSwr9ApdHk5bvJKMjzff41FfuX8bSxdKcR81vTwcA";
pub const SOLANA_WBTC_TOKEN_ID: &str = "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh";
pub const SOLANA_CBBTC_TOKEN_ID: &str = "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij";
pub const SOLANA_JITO_SOL_TOKEN_ID: &str = "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn";
// SUI
pub const SUI_USDC_TOKEN_ID: &str = "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC";
pub const SUI_WAL_TOKEN_ID: &str = "0x356a26eb9e012a68958082340d4c4116e7f55615cf27affcff209cf0ae544f59::wal::WAL";
// SUI bridged
pub const SUI_SBUSDT_TOKEN_ID: &str = "0x375f70cf2ae4c00bf37117d0c85a2c71545e6ee05c4a5c7d282cd66a4504b068::usdt::USDT";
// Thorchain
pub const THORCHAIN_TCY_TOKEN_ID: &str = "tcy";

lazy_static! {
    // ethereum
    pub static ref ETHEREUM_USDT: Asset = Asset::new(
        AssetId::from_token(Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID),
        USDT_NAME.to_owned(),
        USDT_SYMBOL.to_owned(),
        6,
        AssetType::ERC20,
    );
    pub static ref ETHEREUM_USDC: Asset = Asset::new(
        AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID),
        USDC_NAME.to_owned(),
        USDC_SYMBOL.to_owned(),
        6,
        AssetType::ERC20,
    );
    pub static ref ETHEREUM_WBTC: Asset = Asset::new(
        AssetId::from_token(Chain::Ethereum, ETHEREUM_WBTC_TOKEN_ID),
        WBTC_NAME.to_owned(),
        WBTC_SYMBOL.to_owned(),
        8,
        AssetType::ERC20,
    );
    pub static ref ETHEREUM_DAI: Asset = Asset::new(
        AssetId::from_token(Chain::Ethereum, ETHEREUM_DAI_TOKEN_ID),
        DAI_NAME.to_owned(),
        DAI_SYMBOL.to_owned(),
        18,
        AssetType::ERC20,
    );
    pub static ref ETHEREUM_WETH: Asset = Asset::new(
        WETH_ETH_ASSET_ID.into(),
        WETH_NAME.into(),
        WETH_SYMBOL.into(),
        18,
        AssetType::ERC20,
    );
    pub static ref ETHEREUM_USDS: Asset = Asset::new(
        AssetId::from_token(Chain::Ethereum, ETHEREUM_USDS_TOKEN_ID),
        USDS_NAME.to_owned(),
        USDS_SYMBOL.to_owned(),
        18,
        AssetType::ERC20,
    );
    pub static ref ETHEREUM_STETH: Asset = Asset::new(
        AssetId::from_token(Chain::Ethereum, ETHEREUM_STETH_TOKEN_ID),
        "stETH".to_owned(),
        "stETH".to_owned(),
        18,
        AssetType::ERC20,
    );
    pub static ref ETHEREUM_CBBTC: Asset = Asset::new(
        AssetId::from_token(Chain::Ethereum, ETHEREUM_CBBTC_TOKEN_ID),
        CBBTC_NAME.to_owned(),
        CBBTC_SYMBOL.to_owned(),
        8,
        AssetType::ERC20,
    );
    pub static ref ETHEREUM_FLIP: Asset = Asset::new(
        AssetId::from_token(Chain::Ethereum, ETHEREUM_FLIP_TOKEN_ID),
        "Chainflip".to_owned(),
        "FLIP".to_owned(),
        18,
        AssetType::ERC20,
    );
    // arbitrum
    pub static ref ARBITRUM_WETH: Asset = Asset::new(
        WETH_ARB_ASSET_ID.into(),
        WETH_NAME.into(),
        WETH_SYMBOL.into(),
        18,
        AssetType::ERC20,
    );
    pub static ref ARBITRUM_USDC: Asset = Asset::new(
        USDC_ARB_ASSET_ID.into(),
        USDC_NAME.into(),
        USDC_SYMBOL.into(),
        6,
        AssetType::ERC20,
    );
    pub static ref ARBITRUM_USDT: Asset = Asset::new(
        USDT_ARB_ASSET_ID.into(),
        USDT_NAME.into(),
        USDT_SYMBOL.into(),
        6,
        AssetType::ERC20,
    );
    // base
    pub static ref BASE_WETH: Asset = Asset::new(
        WETH_BASE_ASSET_ID.into(),
        WETH_NAME.into(),
        WETH_SYMBOL.into(),
        18,
        AssetType::ERC20,
    );
    pub static ref BASE_USDC: Asset = Asset::new(
        AssetId::from_token(Chain::Base, BASE_USDC_TOKEN_ID),
        USDC_NAME.to_owned(),
        USDC_SYMBOL.to_owned(),
        6,
        AssetType::ERC20,
    );
    pub static ref BASE_CBBTC: Asset = Asset::new(
        AssetId::from_token(Chain::Base, BASE_CBBTC_TOKEN_ID),
        CBBTC_NAME.to_owned(),
        CBBTC_SYMBOL.to_owned(),
        8,
        AssetType::ERC20,
    );
    pub static ref BASE_USDS: Asset = Asset::new(
        AssetId::from_token(Chain::Base, BASE_USDS_TOKEN_ID),
        USDS_NAME.to_owned(),
        USDS_SYMBOL.to_owned(),
        18,
        AssetType::ERC20,
    );
    pub static ref BASE_WBTC: Asset = Asset::new(
        AssetId::from_token(Chain::Base, BASE_WBTC_TOKEN_ID),
        WBTC_NAME.to_owned(),
        WBTC_SYMBOL.to_owned(),
        8,
        AssetType::ERC20,
    );
    // blast
    pub static ref BLAST_WETH: Asset = Asset::new(
        WETH_BLAST_ASSET_ID.into(),
        WETH_NAME.into(),
        WETH_SYMBOL.into(),
        18,
        AssetType::ERC20,
    );
    // linea
    pub static ref LINEA_WETH: Asset = Asset::new(
        WETH_LINEA_ASSET_ID.into(),
        WETH_NAME.into(),
        WETH_SYMBOL.into(),
        18,
        AssetType::ERC20,
    );
    pub static ref LINEA_USDC: Asset = Asset::new(
        USDC_E_LINEA_ASSET_ID.into(),
        USDC_NAME.into(),
        USDC_SYMBOL.into(),
        6,
        AssetType::ERC20,
    );
    pub static ref LINEA_USDT: Asset = Asset::new(
        USDT_LINEA_ASSET_ID.into(),
        USDT_NAME.into(),
        USDT_SYMBOL.into(),
        6,
        AssetType::ERC20,
    );
    // optimism
    pub static ref OPTIMISM_WETH: Asset = Asset::new(
        WETH_OP_ASSET_ID.into(),
        WETH_NAME.into(),
        WETH_SYMBOL.into(),
        18,
        AssetType::ERC20,
    );
    pub static ref OPTIMISM_USDC: Asset = Asset::new(
        USDC_OP_ASSET_ID.into(),
        USDC_NAME.into(),
        USDC_SYMBOL.into(),
        6,
        AssetType::ERC20,
    );
    pub static ref OPTIMISM_USDT: Asset = Asset::new(
        USDT_OP_ASSET_ID.into(),
        USDT_NAME.into(),
        USDT_SYMBOL.into(),
        6,
        AssetType::ERC20,
    );
    // polygon
    pub static ref POLYGON_WETH: Asset = Asset::new(
        WETH_POLYGON_ASSET_ID.into(),
        WETH_NAME.into(),
        WETH_SYMBOL.into(),
        18,
        AssetType::ERC20,
    );
    pub static ref POLYGON_USDC: Asset = Asset::new(
        USDC_POLYGON_ASSET_ID.into(),
        USDC_NAME.into(),
        USDC_SYMBOL.into(),
        6,
        AssetType::ERC20,
    );
    pub static ref POLYGON_USDT: Asset = Asset::new(
        USDT_POLYGON_ASSET_ID.into(),
        USDT_NAME.into(),
        USDT_SYMBOL.into(),
        6,
        AssetType::ERC20,
    );
    // zksync
    pub static ref ZKSYNC_WETH: Asset = Asset::new(
        WETH_ZKSYNC_ASSET_ID.into(),
        WETH_NAME.into(),
        WETH_SYMBOL.into(),
        18,
        AssetType::ERC20,
    );
    pub static ref ZKSYNC_USDT: Asset = Asset::new(
        USDT_ZKSYNC_ASSET_ID.into(),
        USDT_NAME.into(),
        USDT_SYMBOL.into(),
        6,
        AssetType::ERC20,
    );
    // world
    pub static ref WORLD_WETH: Asset = Asset::new(
        WETH_WORLD_ASSET_ID.into(),
        WETH_NAME.into(),
        WETH_SYMBOL.into(),
        18,
        AssetType::ERC20,
    );
    // smartchain
    pub static ref SMARTCHAIN_USDT: Asset = Asset::new(
        AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDT_TOKEN_ID),
        USDT_NAME.to_owned(),
        USDT_SYMBOL.to_owned(),
        18,
        AssetType::BEP20,
    );
    pub static ref SMARTCHAIN_USDC: Asset = Asset::new(
        AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDC_TOKEN_ID),
        USDC_NAME.to_owned(),
        USDC_SYMBOL.to_owned(),
        18,
        AssetType::BEP20,
    );
    pub static ref SMARTCHAIN_WBTC: Asset = Asset::new(
        AssetId::from_token(Chain::SmartChain, SMARTCHAIN_WBTC_TOKEN_ID),
        WBTC_NAME.to_owned(),
        WBTC_SYMBOL.to_owned(),
        8,
        AssetType::BEP20,
    );
    // avalanche
    pub static ref AVALANCHE_USDT: Asset = Asset::new(
        AssetId::from_token(Chain::AvalancheC, AVALANCHE_USDT_TOKEN_ID),
        USDT_NAME.to_owned(),
        USDT_SYMBOL.to_owned(),
        6,
        AssetType::ERC20,
    );
    pub static ref AVALANCHE_USDC: Asset = Asset::new(
        AssetId::from_token(Chain::AvalancheC, AVALANCHE_USDC_TOKEN_ID),
        USDC_NAME.to_owned(),
        USDC_SYMBOL.to_owned(),
        6,
        AssetType::ERC20,
    );
    // ink
    pub static ref INK_WETH: Asset = Asset::new(
        WETH_INK_ASSET_ID.into(),
        WETH_NAME.into(),
        WETH_SYMBOL.into(),
        18,
        AssetType::ERC20,
    );
    pub static ref INK_USDT: Asset = Asset::new(
        USDT_INK_ASSET_ID.into(),
        USDT_NAME.to_owned(),
        USDT_SYMBOL.to_owned(),
        6,
        AssetType::ERC20,
    );
    // Unichain
    pub static ref UNICHAIN_WETH: Asset = Asset::new(
        WETH_UNICHAIN_ASSET_ID.into(),
        WETH_NAME.into(),
        WETH_SYMBOL.into(),
        18,
        AssetType::ERC20,
    );
    pub static ref UNICHAIN_USDC: Asset = Asset::new(
        USDC_UNICHAIN_ASSET_ID.into(),
        USDC_NAME.to_owned(),
        USDC_SYMBOL.to_owned(),
        6,
        AssetType::ERC20,
    );
    pub static ref UNICHAIN_DAI: Asset = Asset::new(
        DAI_UNICHAIN_ASSET_ID.into(),
        DAI_NAME.to_owned(),
        DAI_SYMBOL.to_owned(),
        18,
        AssetType::ERC20,
    );
    // Solana
    pub static ref SOLANA_USDC: Asset = Asset::new(
        AssetId::from_token(Chain::Solana, SOLANA_USDC_TOKEN_ID),
        USDC_NAME.to_owned(),
        USDC_SYMBOL.to_owned(),
        6,
        AssetType::SPL,
    );
    pub static ref SOLANA_USDT: Asset = Asset::new(
        AssetId::from_token(Chain::Solana, SOLANA_USDT_TOKEN_ID),
        USDT_NAME.to_owned(),
        USDT_SYMBOL.to_owned(),
        6,
        AssetType::SPL,
    );
    pub static ref SOLANA_USDS: Asset = Asset::new(
        AssetId::from_token(Chain::Solana, SOLANA_USDS_TOKEN_ID),
        USDS_NAME.to_owned(),
        USDS_SYMBOL.to_owned(),
        6,
        AssetType::SPL,
    );
    pub static ref SOLANA_WBTC: Asset = Asset::new(
        AssetId::from_token(Chain::Solana, SOLANA_WBTC_TOKEN_ID),
        WBTC_NAME.to_owned(),
        WBTC_SYMBOL.to_owned(),
        8,
        AssetType::SPL,
    );
    pub static ref SOLANA_CBBTC: Asset = Asset::new(
        AssetId::from_token(Chain::Solana, SOLANA_CBBTC_TOKEN_ID),
        CBBTC_NAME.to_owned(),
        CBBTC_SYMBOL.to_owned(),
        8,
        AssetType::SPL,
    );
    pub static ref SOLANA_JITO_SOL: Asset = Asset::new(
        AssetId::from_token(Chain::Solana, SOLANA_JITO_SOL_TOKEN_ID),
        "Jito Staked SOL".to_owned(),
        "JitoSOL".to_owned(),
        9,
        AssetType::SPL,
    );
    // Sui
    pub static ref SUI_USDC: Asset = Asset::new(
        AssetId::from_token(Chain::Sui, SUI_USDC_TOKEN_ID),
        USDC_NAME.to_owned(),
        USDC_SYMBOL.to_owned(),
        6,
        AssetType::TOKEN,
    );
    pub static ref SUI_SBUSDT: Asset = Asset::new(
        AssetId::from_token(Chain::Sui, SUI_SBUSDT_TOKEN_ID),
        "Sui Bridged USDT".to_owned(),
        "sbUSDT".to_owned(),
        6,
        AssetType::TOKEN,
    );
    pub static ref SUI_WAL: Asset = Asset::new(
        AssetId::from_token(Chain::Sui, SUI_WAL_TOKEN_ID),
        "Walrus".to_owned(),
        "WAL".to_owned(),
        9,
        AssetType::TOKEN,
    );
    // Thorchain
    pub static ref THORCHAIN_TCY: Asset = Asset::new(
        AssetId::from_token(Chain::Thorchain, THORCHAIN_TCY_TOKEN_ID),
        "TCY".to_owned(),
        "TCY".to_owned(),
        8,
        AssetType::TOKEN,
    );

}
