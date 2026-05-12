use crate::models::{AssetRow, NewAssetRow};
use chrono::Utc;
use primitives::{Asset, Chain};

impl AssetRow {
    pub fn mock() -> Self {
        let asset = NewAssetRow::from_primitive_default(Asset::from_chain(Chain::Bitcoin));
        Self {
            id: asset.id,
            chain: asset.chain,
            token_id: asset.token_id,
            name: asset.name,
            symbol: asset.symbol,
            asset_type: asset.asset_type,
            decimals: asset.decimals,
            rank: asset.rank,
            is_enabled: asset.is_enabled,
            is_buyable: asset.is_buyable,
            is_sellable: asset.is_sellable,
            is_swappable: asset.is_swappable,
            is_stakeable: asset.is_stakeable,
            staking_apr: asset.staking_apr,
            is_earnable: asset.is_earnable,
            earn_apr: asset.earn_apr,
            has_image: asset.has_image,
            has_price: asset.has_price,
            circulating_supply: asset.circulating_supply,
            total_supply: asset.total_supply,
            max_supply: asset.max_supply,
            updated_at: Utc::now().naive_utc(),
        }
    }
}
