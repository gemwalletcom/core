use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use crate::XRP_DEFAULT_ASSET_DECIMALS;
use crate::rpc::client::XRPClient;
use gem_client::Client;
use primitives::{Asset, AssetId, AssetType};

#[async_trait]
impl<C: Client> ChainToken for XRPClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let objects = self.get_account_objects(&token_id).await?;

        if let Some(asset) = objects.account_objects.unwrap_or_default().first() {
            let currency = &asset.low_limit.currency;
            let currency_bytes = hex::decode(currency.trim_end_matches('0')).map_err(|_| "Invalid currency hex")?;
            let symbol = String::from_utf8(currency_bytes.into_iter().filter(|&b| b != 0).collect()).unwrap_or_else(|_| currency.clone());

            Ok(Asset {
                id: AssetId::from_token(self.chain, &token_id),
                chain: self.chain,
                token_id: Some(token_id.clone()),
                name: symbol.clone(),
                symbol,
                decimals: XRP_DEFAULT_ASSET_DECIMALS as i32,
                asset_type: AssetType::TOKEN,
            })
        } else {
            Err("Token not found".into())
        }
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        token_id.len() == 34 && token_id.starts_with('r')
    }
}
