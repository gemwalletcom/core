use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use futures::try_join;
use num_bigint::BigInt;
use std::{error::Error, str::FromStr};

use gem_client::Client;
use primitives::{FeeRate, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput};

use crate::{
    provider::{
        preload_mapper::{address_to_public_key, map_transaction_preload},
        state_mapper::map_gas_price_to_priorities,
    },
    rpc::client::NearClient,
};

#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for NearClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let public_key = address_to_public_key(&input.sender_address)?;
        let (access_key, block) = try_join!(self.get_account_access_key(&input.sender_address, &public_key), self.get_latest_block(),)?;
        Ok(map_transaction_preload(&access_key, &block))
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: TransactionFee::new_from_gas_price_limit(input.gas_price.gas_price(), BigInt::from_str("9000000000000")?), // "4174947687500" * 2
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let gas_price = self.get_gas_price().await?;
        map_gas_price_to_priorities(&gas_price)
    }
}
