use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate, TransactionState, TransactionChange};

use crate::rpc::client::StellarClient;

#[async_trait]
impl<C: Client> ChainTransactions for StellarClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        let result = self.broadcast_transaction(&data).await?;
        
        if let Some(hash) = result.hash {
            Ok(hash)
        } else if let Some(error) = result.error_message {
            Err(format!("Broadcast error: {}", error).into())
        } else {
            Err("Unknown broadcast error".into())
        }
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let tx = self.get_transaction_status(&request.id).await?;
        
        let state = if tx.successful {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        };
        
        let network_fee = tx.fee_charged.parse::<u64>().unwrap_or(0);
        
        Ok(TransactionUpdate {
            state,
            changes: vec![TransactionChange::NetworkFee(network_fee.to_string())],
        })
    }
}