use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionState, TransactionStateRequest, TransactionUpdate, TransactionChange};

use crate::rpc::client::PolkadotClient;

#[async_trait]
impl<C: Client> ChainTransactions for PolkadotClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        let response = self.broadcast_transaction(data).await?;
        
        if let Some(hash) = response.hash {
            Ok(hash)
        } else if let Some(error) = response.error {
            let cause = response.cause.unwrap_or_default();
            Err(format!("{}: {}", error, cause).into())
        } else {
            Err("Invalid broadcast response".into())
        }
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let block_number = request.block_number;
        
        let block_head = self.get_block_head().await?;
        let from_block = block_number as u64;
        let to_block = std::cmp::min(block_head.number, from_block + 64);
        
        let blocks = self.get_blocks(&from_block.to_string(), &to_block.to_string()).await?;
        
        for block in blocks {
            for extrinsic in block.extrinsics {
                if extrinsic.hash == request.id {
                    let state = if extrinsic.success {
                        TransactionState::Confirmed
                    } else {
                        TransactionState::Failed
                    };
                    return Ok(TransactionUpdate::new_state(state));
                }
            }
        }
        
        Ok(TransactionUpdate::new(
            TransactionState::Pending,
            vec![TransactionChange::BlockNumber(block_number.to_string())]
        ))
    }
}