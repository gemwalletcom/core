use lazy_static::lazy_static;
use primitives::{asset_constants::*, Asset, AssetId, AssetType, Chain};

const USDT_NAME: &str = "Tether";
const USDT_SYMBOL: &str = "USDT";
const USDC_NAME: &str = "USDC";
const USDC_SYMBOL: &str = "USDC";
const WBTC_SYMBOL: &str = "WBTC";
const WBTC_NAME: &str = "Wrapped BTC";
const DAI_SYMBOL: &str = "DAI";
const WETH_NAME: &str = "Wrapped Ether";
const WETH_SYMBOL: &str = "WETH";
const CBBTC_NAME: &str = "Coinbase BTC";
const CBBTC_SYMBOL: &str = "cbBTC";

// TODO: merge into primitives::asset_constants
pub const ETHEREUM_USDC_TOKEN_ID: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
pub const ETHEREUM_USDT_TOKEN_ID: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
pub const ETHEREUM_WBTC_TOKEN_ID: &str = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599";
pub const ETHEREUM_DAI_TOKEN_ID: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
pub const ETHEREUM_METH_TOKEN_ID: &str = "0xd5f7838f5c461feff7fe49ea5ebaf7728bb0adfa";
pub const SMARTCHAIN_USDT_TOKEN_ID: &str = "0x55d398326f99059fF775485246999027B3197955";
pub const SMARTCHAIN_USDC_TOKEN_ID: &str = "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d";
pub const AVALANCHE_USDT_TOKEN_ID: &str = "0x9702230A8Ea53601f5cD2dc00fDBc13d4dF4A8c7";
pub const AVALANCHE_USDC_TOKEN_ID: &str = "0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E";
pub const BASE_USDC_TOKEN_ID: &str = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";
pub const BASE_CBBTC_TOKEN_ID: &str = "0xcbB7C0000aB88B473b1f5aFd9ef808440eed33Bf";

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
        name: WBTC_NAME.to_owned(),
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
    pub static ref ETHEREUM_WETH: Asset = Asset {
        id: WETH_ETH_ASSET_ID.into(),
        name: WETH_NAME.into(),
        symbol: WETH_SYMBOL.into(),
        decimals: 18,
        asset_type: AssetType::ERC20,
    };
    // arbitrum
    pub static ref ARBITRUM_WETH: Asset = Asset {
        id: WETH_ARB_ASSET_ID.into(),
        name: WETH_NAME.into(),
        symbol: WETH_SYMBOL.into(),
        decimals: 18,
        asset_type: AssetType::ERC20,
    };
    pub static ref ARBITRUM_USDC: Asset = Asset {
        id: USDC_ARB_ASSET_ID.into(),
        name: USDC_NAME.into(),
        symbol: USDC_SYMBOL.into(),
        decimals: 6,
        asset_type: AssetType::ERC20,
    };
    pub static ref ARBITRUM_USDT: Asset = Asset {
        id: USDT_ARB_ASSET_ID.into(),
        name: USDT_NAME.into(),
        symbol: USDT_SYMBOL.into(),
        decimals: 6,
        asset_type: AssetType::ERC20,
    };
    // base
    pub static ref BASE_WETH: Asset = Asset {
        id: WETH_BASE_ASSET_ID.into(),
        name: WETH_NAME.into(),
        symbol: WETH_SYMBOL.into(),
        decimals: 18,
        asset_type: AssetType::ERC20,
    };
    pub static ref BASE_USDC: Asset = Asset {
        id: AssetId::from_token(Chain::Base, BASE_USDC_TOKEN_ID),
        name: USDT_NAME.to_owned(),
        symbol: USDT_SYMBOL.to_owned(),
        decimals: 6,
        asset_type: AssetType::ERC20,
    };
    pub static ref BASE_CBBTC: Asset = Asset {
        id: AssetId::from_token(Chain::Base, BASE_CBBTC_TOKEN_ID),
        name: CBBTC_NAME.to_owned(),
        symbol: CBBTC_SYMBOL.to_owned(),
        decimals: 8,
        asset_type: AssetType::ERC20,
    };
    // blast
    pub static ref BLAST_WETH: Asset = Asset {
        id: WETH_BLAST_ASSET_ID.into(),
        name: WETH_NAME.into(),
        symbol: WETH_SYMBOL.into(),
        decimals: 18,
        asset_type: AssetType::ERC20,
    };
    // linea
    pub static ref LINEA_WETH: Asset = Asset {
        id: WETH_LINEA_ASSET_ID.into(),
        name: WETH_NAME.into(),
        symbol: WETH_SYMBOL.into(),
        decimals: 18,
        asset_type: AssetType::ERC20,
    };
    pub static ref LINEA_USDT: Asset = Asset {
        id: USDT_LINEA_ASSET_ID.into(),
        name: USDT_NAME.into(),
        symbol: USDT_SYMBOL.into(),
        decimals: 6,
        asset_type: AssetType::ERC20,
    };
    // optimism
    pub static ref OPTIMISM_WETH: Asset = Asset {
        id: WETH_OP_ASSET_ID.into(),
        name: WETH_NAME.into(),
        symbol: WETH_SYMBOL.into(),
        decimals: 18,
        asset_type: AssetType::ERC20,
    };
    pub static ref OPTIMISM_USDC: Asset = Asset {
        id: USDC_OP_ASSET_ID.into(),
        name: USDC_NAME.into(),
        symbol: USDC_SYMBOL.into(),
        decimals: 6,
        asset_type: AssetType::ERC20,
    };
    pub static ref OPTIMISM_USDT: Asset = Asset {
        id: USDT_OP_ASSET_ID.into(),
        name: USDT_NAME.into(),
        symbol: USDT_SYMBOL.into(),
        decimals: 6,
        asset_type: AssetType::ERC20,
    };
    // polygon
    pub static ref POLYGON_WETH: Asset = Asset {
        id: WETH_POLYGON_ASSET_ID.into(),
        name: WETH_NAME.into(),
        symbol: WETH_SYMBOL.into(),
        decimals: 18,
        asset_type: AssetType::ERC20,
    };
    pub static ref POLYGON_USDC: Asset = Asset {
        id: USDC_POLYGON_ASSET_ID.into(),
        name: USDC_NAME.into(),
        symbol: USDC_SYMBOL.into(),
        decimals: 6,
        asset_type: AssetType::ERC20,
    };
    pub static ref POLYGON_USDT: Asset = Asset {
        id: USDT_POLYGON_ASSET_ID.into(),
        name: USDT_NAME.into(),
        symbol: USDT_SYMBOL.into(),
        decimals: 6,
        asset_type: AssetType::ERC20,
    };
    // zksync
    pub static ref ZKSYNC_WETH: Asset = Asset {
        id: WETH_ZKSYNC_ASSET_ID.into(),
        name: WETH_NAME.into(),
        symbol: WETH_SYMBOL.into(),
        decimals: 18,
        asset_type: AssetType::ERC20,
    };
    pub static ref ZKSYNC_USDT: Asset = Asset {
        id: USDT_ZKSYNC_ASSET_ID.into(),
        name: USDT_NAME.into(),
        symbol: USDT_SYMBOL.into(),
        decimals: 6,
        asset_type: AssetType::ERC20,
    };
    // world
    pub static ref WORLD_WETH: Asset = Asset {
        id: WETH_WORLD_ASSET_ID.into(),
        name: WETH_NAME.into(),
        symbol: WETH_SYMBOL.into(),
        decimals: 18,
        asset_type: AssetType::ERC20,
    };
    // smartchain
    pub static ref SMARTCHAIN_USDT: Asset = Asset {
        id: AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDT_TOKEN_ID),
        name: USDT_NAME.to_owned(),
        symbol: USDT_SYMBOL.to_owned(),
        decimals: 18,
        asset_type: AssetType::BEP20,
    };
    pub static ref SMARTCHAIN_USDC: Asset = Asset {
        id: AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDC_TOKEN_ID),
        name: USDC_NAME.to_owned(),
        symbol: USDC_SYMBOL.to_owned(),
        decimals: 18,
        asset_type: AssetType::BEP20,
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
