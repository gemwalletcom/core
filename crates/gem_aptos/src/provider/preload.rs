use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionFee, TransactionLoadData, TransactionLoadInput, TransactionPreload, TransactionPreloadInput};

use super::preload_mapper::map_transaction_preload;
use crate::rpc::client::AptosClient;

#[async_trait]
impl<C: Client> ChainPreload for AptosClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&input.sender_address).await?;
        map_transaction_preload(&account)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let gas_limit = self.calculate_gas_limit(&input).await?;
        let fee = TransactionFee::calculate(gas_limit, &input.gas_price);

        Ok(TransactionLoadData::builder().sequence(input.sequence).fee(fee).build())
    }
}
