use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use futures::try_join;
use gem_client::Client;
use primitives::{FeePriority, FeeRate, GasPriceType, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput, UTXO};

use super::preload_mapper::{map_transaction_fee, map_transaction_preload};
use crate::planner::plan_transfer;
use crate::rpc::client::CardanoClient;

#[async_trait]
impl<C: Client> ChainTransactionLoad for CardanoClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let (utxos, tip) = try_join!(self.get_utxos(&input.sender_address), self.get_tip())?;
        Ok(map_transaction_preload(utxos, tip.slot_no))
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let plan = plan_transfer(&input)?;
        Ok(TransactionLoadData {
            fee: map_transaction_fee(plan.fee),
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(1))])
    }

    async fn get_utxos(&self, address: String) -> Result<Vec<UTXO>, Box<dyn Error + Sync + Send>> {
        Ok(CardanoClient::get_utxos(self, &address).await?.into_iter().map(UTXO::from).collect())
    }
}
