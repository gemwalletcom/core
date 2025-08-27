use super::model::Price;

pub struct PythClient {
    _rpc_url: String,
}

impl PythClient {
    pub fn new(_rpc_url: &str) -> Self {
        Self {
            _rpc_url: _rpc_url.to_string(),
        }
    }

    pub async fn get_asset_prices(&self, _price_ids: Vec<String>) -> Result<Vec<Price>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Pyth price fetching not implemented".into())
    }

    pub async fn get_price(&self, _price_id: &str) -> Result<Price, Box<dyn std::error::Error + Send + Sync>> {
        Err("Pyth price fetching not implemented".into())
    }
}
