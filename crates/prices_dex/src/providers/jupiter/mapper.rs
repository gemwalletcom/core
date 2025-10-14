use primitives::{AssetId, Chain};

pub fn map_id_to_asset_id(id: &str) -> AssetId {
    match id {
        "So11111111111111111111111111111111111111112" => AssetId::from(Chain::Solana, None),
        _ => AssetId::from(Chain::Solana, Some(id.to_string())),
    }
}
