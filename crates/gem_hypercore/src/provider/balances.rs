use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use number_formatter::BigNumberFormatter;
use primitives::{AssetBalance, Balance};

use crate::rpc::client::HyperCoreClient;

#[async_trait]
impl<C: Client> ChainBalances for HyperCoreClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let total = self
            .get_spot_balances(&address)
            .await?
            .balances
            .into_iter()
            .find(|x| x.token == 150)
            .ok_or("not found")?
            .total;
        let available: String = BigNumberFormatter::value_from_amount(&total, 18)?;
        Ok(AssetBalance::new(self.chain.as_asset_id(), available))
    }

    async fn get_balance_tokens(&self, _address: String, _token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let balance = self.get_stake_balance(&address).await?;
        let available = BigNumberFormatter::value_from_amount(&balance.delegated, 18)?;
        let pending = BigNumberFormatter::value_from_amount(&balance.total_pending_withdrawal, 18)?;

        Ok(Some(AssetBalance::new_balance(
            self.chain.as_asset_id(),
            Balance::stake_balance(available, pending, None),
        )))
    }
}
