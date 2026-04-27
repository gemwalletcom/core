use std::collections::HashSet;
use std::error::Error;

use gem_tracing::info_with_fields;
use primitives::AssetId;
use storage::{AssetsUsageRanksRepository, Database, PricesRepository};
use streamer::{StreamProducer, StreamProducerQueue};

const MAX_ASSETS_PER_RUN: usize = 1;

pub struct MissingPricesPublisher {
    database: Database,
    stream_producer: StreamProducer,
}

impl MissingPricesPublisher {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let ranks: Vec<(AssetId, i32)> = self
            .database
            .assets_usage_ranks()?
            .get_all_usage_ranks()?
            .into_iter()
            .map(|row| (row.asset_id.0, row.usage_rank))
            .collect();
        let priced: HashSet<AssetId> = self.database.prices()?.get_prices_assets()?.into_iter().map(|row| row.asset_id.0).collect();
        let asset_ids: Vec<AssetId> = missing_assets(ranks, &priced).into_iter().take(MAX_ASSETS_PER_RUN).collect();
        let count = asset_ids.len();
        self.stream_producer.publish_fetch_prices_assets(asset_ids.clone()).await?;
        for asset_id in &asset_ids {
            info_with_fields!("publish missing prices", asset_id = asset_id.to_string());
        }
        Ok(count)
    }
}

fn missing_assets(ranks: Vec<(AssetId, i32)>, priced: &HashSet<AssetId>) -> Vec<AssetId> {
    let mut candidates: Vec<(AssetId, i32)> = ranks.into_iter().filter(|(id, _)| !priced.contains(id)).collect();
    candidates.sort_by_key(|(_, rank)| std::cmp::Reverse(*rank));
    candidates.into_iter().map(|(id, _)| id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_missing_assets() {
        let priced = Chain::Ethereum.as_asset_id();
        let unpriced_high = Chain::Solana.as_asset_id();
        let unpriced_low = Chain::Bitcoin.as_asset_id();
        let ranks = vec![(unpriced_high.clone(), 80), (priced.clone(), 100), (unpriced_low.clone(), 20)];
        let priced: HashSet<AssetId> = [priced].into_iter().collect();

        assert_eq!(missing_assets(ranks, &priced), vec![unpriced_high, unpriced_low]);
    }
}
