use primitives::{AssetId, Chain, contract_constants::SOLANA_WRAPPED_SOL_TOKEN_ADDRESS};

pub fn map_id_to_asset_id(id: &str) -> AssetId {
    match id {
        SOLANA_WRAPPED_SOL_TOKEN_ADDRESS => AssetId::from(Chain::Solana, None),
        _ => AssetId::from(Chain::Solana, Some(id.to_string())),
    }
}
