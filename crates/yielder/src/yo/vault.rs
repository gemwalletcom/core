use alloy_primitives::{Address, address};
use primitives::{AssetId, Chain};

#[derive(Debug, Clone, Copy)]
pub struct YoVault {
    pub name: &'static str,
    pub chain: Chain,
    pub yo_token: Address,
    pub asset_token: Address,
    pub asset_decimals: u8,
}

impl YoVault {
    pub const fn new(name: &'static str, chain: Chain, yo_token: Address, asset_token: Address, asset_decimals: u8) -> Self {
        Self {
            name,
            chain,
            yo_token,
            asset_token,
            asset_decimals,
        }
    }

    pub fn asset_id(&self) -> AssetId {
        AssetId::from_token(self.chain, &self.asset_token.to_string())
    }
}

pub const YO_USD: YoVault = YoVault::new(
    "yoUSD",
    Chain::Base,
    address!("0x0000000f2eb9f69274678c76222b35eec7588a65"),
    address!("0x833589fcd6edb6e08f4c7c32d4f71b54bda02913"),
    6,
);

pub const YO_USDT: YoVault = YoVault::new(
    "yoUSDT",
    Chain::Ethereum,
    address!("0xb9a7da9e90d3b428083bae04b860faa6325b721e"),
    address!("0xdac17f958d2ee523a2206206994597c13d831ec7"),
    6,
);

pub fn vaults() -> &'static [YoVault] {
    &[YO_USD, YO_USDT]
}
