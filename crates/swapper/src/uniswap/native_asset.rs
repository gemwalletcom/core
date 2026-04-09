use primitives::{AssetId, Chain};

pub fn uses_msg_value(asset_id: &AssetId) -> bool {
    asset_id.is_native() && !is_tokenized_native(asset_id.chain)
}

pub fn is_tokenized_native(chain: Chain) -> bool {
    [Chain::Celo].contains(&chain)
}
