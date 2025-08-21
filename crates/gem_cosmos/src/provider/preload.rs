use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use std::error::Error;

use gem_client::Client;
use primitives::{FeeRate, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput};

use crate::{
    provider::{preload_mapper::calculate_transaction_fee, state_mapper::calculate_fee_rates},
    rpc::client::CosmosClient,
};

#[async_trait]
impl<C: Client> ChainTransactionLoad for CosmosClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let account = self.get_account_info(&input.sender_address).await?;
        Ok(TransactionLoadMetadata::Cosmos {
            account_number: account.account_number,
            sequence: account.sequence,
            chain_id: self.get_chain().as_chain().network_id().to_string(),
        })
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let account = self.get_account_info(&input.sender_address).await?;
        let fee = calculate_transaction_fee(&input.input_type, self.get_chain(), &input.gas_price);

        Ok(TransactionLoadData {
            fee,
            metadata: TransactionLoadMetadata::Cosmos {
                account_number: account.account_number,
                sequence: account.sequence,
                chain_id: self.get_chain().as_chain().network_id().to_string(),
            },
        })
    }

    async fn get_transaction_fee_rates(&self) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let base_fee = self.get_base_fee();
        let cosmos_chain = self.get_chain();

        Ok(calculate_fee_rates(cosmos_chain, base_fee.into()))
    }
}
