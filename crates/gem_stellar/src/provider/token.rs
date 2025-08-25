use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use crate::constants::STELLAR_TOKEN_DECIMALS;
use crate::rpc::client::StellarClient;
use gem_client::Client;
use primitives::{Asset, AssetId, AssetType};

#[async_trait]
impl<C: Client> ChainToken for StellarClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let (issuer, symbol) = if token_id.contains("::") {
            let parts: Vec<&str> = token_id.split("::").collect();
            if parts.len() != 2 {
                return Err("Invalid token ID format. Expected: issuer::symbol".into());
            }
            (parts[0], Some(parts[1]))
        } else {
            (token_id.as_str(), None)
        };

        let assets = self.get_assets_by_issuer(issuer).await?;

        let asset = if let Some(sym) = symbol {
            assets
                ._embedded
                .records
                .iter()
                .find(|a| a.asset_code == sym)
                .ok_or_else(|| format!("Asset not found: {}", token_id))?
        } else {
            assets
                ._embedded
                .records
                .first()
                .ok_or_else(|| format!("No assets found for issuer: {}", issuer))?
        };
        let symbol = asset.asset_code.clone();
        let token_id = AssetId::sub_token_id(&[issuer.to_string(), symbol.clone()]);

        Ok(Asset {
            id: AssetId::from(self.chain, Some(token_id.clone())),
            chain: self.chain,
            token_id: Some(token_id),
            name: symbol.clone(),
            symbol,
            decimals: STELLAR_TOKEN_DECIMALS,
            asset_type: AssetType::TOKEN,
        })
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        token_id.len() >= 56 && token_id.starts_with("G")
    }
}
