use primitives::asset_score::AssetRank;
use std::error::Error;
use storage::{AssetUpdate, DatabaseClient};

pub struct AssetRankVerifiedUpdater {
    database: DatabaseClient,
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

impl AssetRankVerifiedUpdater {
    pub fn new(database_url: &str) -> Self {
        AssetRankVerifiedUpdater {
            database: DatabaseClient::new(database_url),
        }
    }

    pub async fn update_suspicious_assets(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let assets = self.database.assets().get_assets_all()?;

        let mut updated_count = 0;

        for asset in assets {
            let basic = asset.as_basic_primitive();
            if is_suspicious(basic.score.rank, &basic.asset.name, &basic.asset.symbol) {
                let asset_ids = vec![asset.id.to_string()];
                let updates = vec![AssetUpdate::Rank(AssetRank::Fraudulent.threshold()), AssetUpdate::IsEnabled(false)];

                if self.database.assets().update_assets(asset_ids, updates).is_ok() {
                    updated_count += 1;
                }
            }
        }

        Ok(updated_count)
    }
}

fn is_suspicious(rank: i32, name: &str, symbol: &str) -> bool {
    let suspicious_assets = [
        SuspiciousAsset::new("Tether".to_string(), "USDT".to_string()),
        SuspiciousAsset::new("Tether USD".to_string(), "USDT".to_string()),
        SuspiciousAsset::new("Tether USD".to_string(), "$USD₮".to_string()),
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
