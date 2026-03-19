use primitives::{AssetId, Chain};

const WSOL_TOKEN_ADDRESS: &str = "So11111111111111111111111111111111111111112";

pub fn map_id_to_asset_id(id: &str) -> AssetId {
    match id {
        WSOL_TOKEN_ADDRESS => AssetId::from(Chain::Solana, None),
        _ => AssetId::from(Chain::Solana, Some(id.to_string())),
    }
}
