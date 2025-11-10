use primitives::asset_score::AssetRank;
use std::error::Error;
use storage::Database;
use storage::AssetUpdate;

pub struct AssetRankUpdater {
    database: Database,
}

struct SuspiciousAsset {
    name: String,
    symbol: String,
}

impl SuspiciousAsset {
    fn new(name: String, symbol: String) -> Self {
        SuspiciousAsset { name, symbol }
    }
}

impl AssetRankUpdater {
    pub fn new(database: Database) -> Self {
        AssetRankUpdater {
            database,
        }
    }

    pub async fn update_suspicious_assets(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let assets = self.database.client()?.assets().get_assets_all()?;
        let asset_ids: Vec<String> = assets
            .into_iter()
            .filter(|x| is_suspicious(x.score.rank, &x.asset.name, &x.asset.symbol))
            .map(|x| x.asset.id.to_string())
            .collect();

        let updates = vec![AssetUpdate::Rank(AssetRank::Fraudulent.threshold()), AssetUpdate::IsEnabled(false)];
        Ok(self.database.client()?.assets().update_assets(asset_ids, updates)?)
    }
}

fn is_suspicious(rank: i32, name: &str, symbol: &str) -> bool {
    let suspicious_assets = [
        SuspiciousAsset::new("Tether".to_string(), "USDT".to_string()),
        SuspiciousAsset::new("Tether USD".to_string(), "USDT".to_string()),
        SuspiciousAsset::new("Tether USD".to_string(), "$USD₮".to_string()),
        SuspiciousAsset::new("USD Coin".to_string(), "USDC".to_string()),
    ];
    rank <= 15 && suspicious_assets.iter().any(|x| x.name == name && x.symbol == symbol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_suspicious() {
        assert!(is_suspicious(10, "Tether", "USDT"));

        assert!(!is_suspicious(25, "Tether", "USDT"));
        assert!(!is_suspicious(10, "Bitcoin", "BTC"));
    }
}
