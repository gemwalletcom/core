use gem_solana::model::{AccountData, ValueResult};
use gem_solana::SolanaClient;

use crate::pyth::decoder::decode_price_account;

use super::model::Price;

pub struct PythClient {
    client: SolanaClient,
}

impl PythClient {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            client: SolanaClient::new(rpc_url),
        }
    }

    pub async fn get_asset_prices(&self, price_ids: Vec<String>) -> Result<Vec<Price>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .client
            .get_account_info_batch(price_ids, "base64", 100)
            .await?
            .into_iter()
            .collect::<Vec<ValueResult<AccountData>>>()
            .into_iter()
            .filter_map(|x| decode_price_account(&x.value.data[0]).ok())
            .collect())
    }

    pub async fn get_price(&self, price_id: &str) -> Result<Price, Box<dyn std::error::Error + Send + Sync>> {
        let data: ValueResult<AccountData> = self.client.get_account_info(price_id, "base64").await?;
        decode_price_account(&data.value.data[0])
    }
}
