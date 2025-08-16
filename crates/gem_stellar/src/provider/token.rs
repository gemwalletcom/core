use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::{Asset, AssetId, AssetType};

use crate::rpc::client::StellarClient;

const STELLAR_TOKEN_DECIMALS: i32 = 7;

#[async_trait]
impl<C: Client> ChainToken for StellarClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let parts: Vec<&str> = token_id.split('-').collect();
        if parts.len() != 2 {
            return Err("Invalid token ID format. Expected: SYMBOL-ISSUER".into());
        }
        
        let symbol = parts[0];
        let issuer = parts[1];
        
        let assets = self.get_assets_by_issuer(issuer).await?;
        
        let asset = assets._embedded.records
            .iter()
            .find(|a| a.asset_code == symbol)
            .ok_or_else(|| format!("Asset not found: {}", token_id))?;
        
        Ok(Asset {
            id: AssetId::from(self.chain, Some(token_id.clone())),
            chain: self.chain,
            token_id: Some(token_id),
            name: asset.asset_code.clone(),
            symbol: asset.asset_code.clone(),
            decimals: STELLAR_TOKEN_DECIMALS,
            asset_type: AssetType::TOKEN,
        })
    }
    
    fn get_is_token_address(&self, token_id: &str) -> bool {
        token_id.len() > 32 && token_id.contains('-')
    }
}