mod client;
mod model;
mod provider;

use std::sync::LazyLock;

use crate::models::SwapperChainAsset;
use primitives::{
    AssetId, Chain,
    asset_constants::{COSMOS_USDC_TOKEN_ID, INJECTIVE_USDC_TOKEN_ID, OSMOSIS_USDC_TOKEN_ID, OSMOSIS_USDT_TOKEN_ID, SEI_USDC_TOKEN_ID},
};

pub use provider::Squid;

const SQUID_COSMOS_MULTICALL: &str = "osmo1n6ney9tsf55etz9nrmzyd8wa7e64qd3s06a74fqs30ka8pps6cvqtsycr6";

static SUPPORTED_CHAINS: LazyLock<Vec<SwapperChainAsset>> = LazyLock::new(|| {
    vec![
        SwapperChainAsset::Assets(Chain::Cosmos, vec![AssetId::from_token(Chain::Cosmos, COSMOS_USDC_TOKEN_ID)]),
        SwapperChainAsset::Assets(
            Chain::Osmosis,
            vec![
                AssetId::from_token(Chain::Osmosis, OSMOSIS_USDC_TOKEN_ID),
                AssetId::from_token(Chain::Osmosis, OSMOSIS_USDT_TOKEN_ID),
            ],
        ),
        SwapperChainAsset::Assets(Chain::Celestia, vec![]),
        SwapperChainAsset::Assets(Chain::Injective, vec![AssetId::from_token(Chain::Injective, INJECTIVE_USDC_TOKEN_ID)]),
        SwapperChainAsset::Assets(Chain::Sei, vec![AssetId::from_token(Chain::Sei, SEI_USDC_TOKEN_ID)]),
        SwapperChainAsset::Assets(Chain::Noble, vec![]),
    ]
});
