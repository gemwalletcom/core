use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::{AssetBalance, AssetId};

use crate::{provider::balances_mapper, rpc::client::TronClient};

#[async_trait]
impl<C: Client> ChainBalances for TronClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&address).await?;
        balances_mapper::map_coin_balance(&account)
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let mut results = Vec::new();

        for token_id in token_ids {
            // Convert base58 addresses to hex like iOS implementation
            let owner_bytes = bs58::decode(&address)
                .into_vec()
                .map_err(|e| format!("Invalid owner address {}: {}", address, e))?;
            let owner_hex = hex::encode(&owner_bytes);

            // Format parameter as 64-character hex string (32 bytes)
            let parameter = format!("{:0>64}", owner_hex.trim_start_matches("41"));

            let balance_hex = self.trigger_constant_contract(&token_id, "balanceOf(address)", &parameter).await?;
            let asset_id = AssetId::from(self.get_chain(), Some(token_id));
            let balance = balances_mapper::map_token_balance(&balance_hex, asset_id)?;
            results.push(balance);
        }

        Ok(results)
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(Some(AssetBalance::new_staking(
            self.get_chain().as_asset_id(),
            "0".to_string(),
            "0".to_string(),
            "0".to_string(),
        )))
    }
}
