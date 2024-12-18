use lazy_static::lazy_static;
use primitives::{Asset, AssetId, AssetType, Chain};

const USDT_NAME: &str = "Tether";
const USDT_SYMBOL: &str = "USDT";
const USDC_NAME: &str = "USDC";
const USDC_SYMBOL: &str = "USDC";
const WBTC_SYMBOL: &str = "WBTC";
const DAI_SYMBOL: &str = "DAI";

pub const ETHEREUM_USDC_TOKEN_ID: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
pub const ETHEREUM_USDT_TOKEN_ID: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
pub const ETHEREUM_WBTC_TOKEN_ID: &str = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599";
pub const ETHEREUM_DAI_TOKEN_ID: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
pub const SMARTCHAIN_USDT_TOKEN_ID: &str = "0x55d398326f99059fF775485246999027B3197955";
pub const SMARTCHAIN_USDC_TOKEN_ID: &str = "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d";
pub const AVALANCHE_USDT_TOKEN_ID: &str = "0x9702230A8Ea53601f5cD2dc00fDBc13d4dF4A8c7";
pub const AVALANCHE_USDC_TOKEN_ID: &str = "0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E";

lazy_static! {
    // ethereum
    pub static ref ETHEREUM_USDT: Asset = Asset {
        id: AssetId::from_token(Chain::Ethereum, ETHEREUM_USDT_TOKEN_ID),
        name: USDT_NAME.to_owned(),
        symbol: USDT_SYMBOL.to_owned(),
        decimals: 6,
        asset_type: AssetType::ERC20,
    };
    pub static ref ETHEREUM_USDC: Asset = Asset {
        id: AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID),
        name: USDC_SYMBOL.to_owned(),
        symbol: USDC_SYMBOL.to_owned(),
        decimals: 6,
        asset_type: AssetType::ERC20,
    };
    pub static ref ETHEREUM_WBTC: Asset = Asset {
        id: AssetId::from_token(Chain::Ethereum, ETHEREUM_WBTC_TOKEN_ID),
        name: "Wrapped BTC".to_owned(),
        symbol: WBTC_SYMBOL.to_owned(),
        decimals: 8,
        asset_type: AssetType::ERC20,
    };
    pub static ref ETHEREUM_DAI: Asset = Asset {
        id: AssetId::from_token(Chain::Ethereum, ETHEREUM_DAI_TOKEN_ID),
        name: DAI_SYMBOL.to_owned(),
        symbol: DAI_SYMBOL.to_owned(),
        decimals: 18,
        asset_type: AssetType::ERC20,
    };
    // smartchain
    pub static ref SMARTCHAIN_USDT: Asset = Asset {
        id: AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDT_TOKEN_ID),
        name: USDT_NAME.to_owned(),
        symbol: USDT_SYMBOL.to_owned(),
        decimals: 18,
        asset_type: AssetType::ERC20,
    };
    pub static ref SMARTCHAIN_USDC: Asset = Asset {
        id: AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDC_TOKEN_ID),
        name: USDC_NAME.to_owned(),
        symbol: USDC_SYMBOL.to_owned(),
        decimals: 18,
        asset_type: AssetType::ERC20,
    };
    // avalanche
    pub static ref AVALANCHE_USDT: Asset = Asset {
        id: AssetId::from_token(Chain::AvalancheC, AVALANCHE_USDT_TOKEN_ID),
        name: USDT_NAME.to_owned(),
        symbol: USDT_SYMBOL.to_owned(),
        decimals: 6,
        asset_type: AssetType::ERC20,
    };
    pub static ref AVALANCHE_USDC: Asset = Asset {
        id: AssetId::from_token(Chain::AvalancheC, AVALANCHE_USDC_TOKEN_ID),
        name: USDC_NAME.to_owned(),
        symbol: USDC_SYMBOL.to_owned(),
        decimals: 6,
        asset_type: AssetType::ERC20,
    };
}
