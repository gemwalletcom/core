use alloy_primitives::{Address, address};
use primitives::{AssetId, Chain};

#[derive(Debug, Clone, Copy)]
pub struct YoAsset {
    pub chain: Chain,
    pub asset_token: Address,
    pub yo_token: Address,
}

impl YoAsset {
    pub fn asset_id(&self) -> AssetId {
        AssetId::from_token(self.chain, &self.asset_token.to_string())
    }
}

pub const YO_USDC: YoAsset = YoAsset {
    chain: Chain::Base,
    asset_token: address!("0x833589fcd6edb6e08f4c7c32d4f71b54bda02913"),
    yo_token: address!("0x0000000f2eb9f69274678c76222b35eec7588a65"),
};

pub const YO_USDT: YoAsset = YoAsset {
    chain: Chain::Ethereum,
    asset_token: address!("0xdac17f958d2ee523a2206206994597c13d831ec7"),
    yo_token: address!("0xb9a7da9e90d3b428083bae04b860faa6325b721e"),
};

pub fn supported_assets() -> &'static [YoAsset] {
    &[YO_USDC, YO_USDT]
}
