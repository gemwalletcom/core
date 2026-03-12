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
    asset_token: address!("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"),
    yo_token: address!("0x0000000f2eB9f69274678c76222B35eEc7588a65"),
};

pub const YO_USDT: YoAsset = YoAsset {
    chain: Chain::Ethereum,
    asset_token: address!("0xdAC17F958D2ee523a2206206994597C13D831ec7"),
    yo_token: address!("0xb9a7da9e90D3B428083BAe04b860faA6325b721e"),
};

pub fn supported_assets() -> &'static [YoAsset] {
    &[YO_USDC, YO_USDT]
}
