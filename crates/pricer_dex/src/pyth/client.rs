use gem_chain_rpc::SolanaClient;
use gem_solana::jsonrpc::{AccountData, ValueResult};

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

    pub async fn get_price(&self, price_id: &str) -> Result<Price, Box<dyn std::error::Error + Send + Sync>> {
        let data: ValueResult<AccountData> = self.client.get_account_info(price_id, "base64").await?;
        decode_price_account(&data.value.data[0])
    }
}
