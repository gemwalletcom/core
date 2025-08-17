use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::{Asset, AssetId, AssetType};

use crate::rpc::client::AlgorandClient;

#[async_trait]
impl<C: Client> ChainToken for AlgorandClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let asset = self.get_asset(&token_id).await?.asset;

        Ok(Asset {
            id: AssetId {
                chain: self.get_chain(),
                token_id: Some(token_id.clone()),
            },
            chain: self.get_chain(),
            token_id: Some(token_id),
            name: asset.params.name,
            symbol: asset.params.unit_name,
            decimals: asset.params.decimals as i32,
            asset_type: AssetType::ASA,
        })
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        if token_id.len() > 4 && token_id.parse::<u64>().is_ok() {
            return true;
        }
        false
    }
}
