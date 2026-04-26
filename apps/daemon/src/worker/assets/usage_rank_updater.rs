use primitives::AssetId;
use std::collections::HashMap;
use std::error::Error;
use storage::{AssetUsageRankRow, AssetsUsageRanksRepository, Database, TransactionsRepository};

pub struct UsageRankUpdater {
    database: Database,
}

impl UsageRankUpdater {
    pub fn new(database: Database) -> Self {
        UsageRankUpdater { database }
    }

    pub async fn update_usage_ranks(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let now = chrono::Utc::now().naive_utc();
        let thirty_days_ago = now - chrono::Duration::days(30);

        let counts_1h = self.database.transactions()?.get_asset_usage_counts(now - chrono::Duration::hours(1))?;
        let counts_24h = self.database.transactions()?.get_asset_usage_counts(now - chrono::Duration::days(1))?;
        let counts_7d = self.database.transactions()?.get_asset_usage_counts(now - chrono::Duration::days(7))?;
        let counts_30d = self.database.transactions()?.get_asset_usage_counts(thirty_days_ago)?;

        let usage_ranks = calculate_usage_ranks(&counts_1h, &counts_24h, &counts_7d, &counts_30d);
        let rows: Vec<AssetUsageRankRow> = usage_ranks
            .into_iter()
            .map(|(asset_id, usage_rank)| AssetUsageRankRow {
                asset_id: asset_id.into(),
                usage_rank,
            })
            .collect();

        self.database.assets_usage_ranks()?.delete_usage_ranks_before(thirty_days_ago)?;
        Ok(self.database.assets_usage_ranks()?.upsert_usage_ranks(rows)?)
    }
}

fn calculate_usage_ranks(counts_1h: &[(AssetId, i64)], counts_24h: &[(AssetId, i64)], counts_7d: &[(AssetId, i64)], counts_30d: &[(AssetId, i64)]) -> Vec<(AssetId, i32)> {
    let mut raw_scores: HashMap<AssetId, i64> = HashMap::new();

    for (asset_id, count) in counts_1h {
        *raw_scores.entry(asset_id.clone()).or_insert(0) += count * 250;
    }
    for (asset_id, count) in counts_24h {
        *raw_scores.entry(asset_id.clone()).or_insert(0) += count * 100;
    }
    for (asset_id, count) in counts_7d {
        *raw_scores.entry(asset_id.clone()).or_insert(0) += count * 10;
    }
    for (asset_id, count) in counts_30d {
        *raw_scores.entry(asset_id.clone()).or_insert(0) += count;
    }

    if raw_scores.is_empty() {
        return vec![];
    }

    let mut scores: Vec<(AssetId, i64)> = raw_scores.into_iter().collect();
    scores.sort_by_key(|a| a.1);

    let total = scores.len() as f64;
    scores
        .into_iter()
        .enumerate()
        .map(|(position, (asset_id, _))| {
            let percentile = ((position as f64 + 1.0) / total * 100.0).round() as i32;
            (asset_id, percentile.min(100))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_calculate_usage_ranks_empty() {
        let result = calculate_usage_ranks(&[], &[], &[], &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_calculate_usage_ranks_single() {
        let counts_1h = vec![(Chain::Bitcoin.as_asset_id(), 10)];
        let result = calculate_usage_ranks(&counts_1h, &[], &[], &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, Chain::Bitcoin.as_asset_id());
        assert_eq!(result[0].1, 100);
    }

    #[test]
    fn test_calculate_usage_ranks_multiple() {
        let counts_1h = vec![(Chain::Bitcoin.as_asset_id(), 100), (Chain::Ethereum.as_asset_id(), 10), (Chain::Solana.as_asset_id(), 1)];
        let result = calculate_usage_ranks(&counts_1h, &[], &[], &[]);
        assert_eq!(result.len(), 3);

        let asset1_rank = result.iter().find(|(id, _)| *id == Chain::Bitcoin.as_asset_id()).map(|(_, r)| *r).unwrap();
        let asset2_rank = result.iter().find(|(id, _)| *id == Chain::Ethereum.as_asset_id()).map(|(_, r)| *r).unwrap();
        let asset3_rank = result.iter().find(|(id, _)| *id == Chain::Solana.as_asset_id()).map(|(_, r)| *r).unwrap();

        assert_eq!(asset3_rank, 33);
        assert_eq!(asset2_rank, 67);
        assert_eq!(asset1_rank, 100);
    }

    #[test]
    fn test_calculate_usage_ranks_weighted() {
        let counts_1h = vec![(Chain::Bitcoin.as_asset_id(), 2)];
        let counts_24h = vec![(Chain::Ethereum.as_asset_id(), 25)];
        let counts_7d = vec![(Chain::Solana.as_asset_id(), 300)];
        let counts_30d = vec![(Chain::SmartChain.as_asset_id(), 4000)];
        let result = calculate_usage_ranks(&counts_1h, &counts_24h, &counts_7d, &counts_30d);

        let asset1_rank = result.iter().find(|(id, _)| *id == Chain::Bitcoin.as_asset_id()).map(|(_, r)| *r).unwrap();
        let asset2_rank = result.iter().find(|(id, _)| *id == Chain::Ethereum.as_asset_id()).map(|(_, r)| *r).unwrap();
        let asset3_rank = result.iter().find(|(id, _)| *id == Chain::Solana.as_asset_id()).map(|(_, r)| *r).unwrap();
        let asset4_rank = result.iter().find(|(id, _)| *id == Chain::SmartChain.as_asset_id()).map(|(_, r)| *r).unwrap();

        // Scores: asset1=500, asset2=2500, asset3=3000, asset4=4000
        assert_eq!(asset1_rank, 25);
        assert_eq!(asset2_rank, 50);
        assert_eq!(asset3_rank, 75);
        assert_eq!(asset4_rank, 100);
    }
}
