use async_trait::async_trait;
use chain_traits::ChainBalances;
use futures::future::join_all;
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
        let parameter = balances_mapper::format_address_parameter(&address)?;
        let futures: Vec<_> = token_ids
            .into_iter()
            .map(|token_id| {
                let parameter = parameter.clone();
                async move {
                    let balance_hex = self.trigger_constant_contract(&token_id, "balanceOf(address)", &parameter).await?;
                    let asset_id = AssetId::from(self.get_chain(), Some(token_id));
                    balances_mapper::map_token_balance(&balance_hex, asset_id)
                }
            })
            .collect();
        join_all(futures).await.into_iter().collect::<Result<Vec<_>, _>>()
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
