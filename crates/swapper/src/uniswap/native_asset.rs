use primitives::{AssetId, Chain};

pub fn requires_native_wrapping(asset_id: &AssetId) -> bool {
    asset_id.is_native() && !is_native_erc20(asset_id.chain)
}

pub fn is_native_erc20(chain: Chain) -> bool {
    chain == Chain::Celo
}
