use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use std::error::Error;

use crate::provider::preload_mapper::{calculate_fee_rates, calculate_transaction_fee};
use gem_client::Client;
use primitives::{
    FeeRate, SolanaTokenProgramId, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput,
};

use crate::{get_token_program_id_by_address, rpc::client::SolanaClient};

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for SolanaClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        if let Some(token_id) = &input.asset.id.token_id {
            let (block_hash, sender_accounts, recipient_accounts) = futures::try_join!(
                self.get_latest_blockhash(),
                self.get_token_accounts_by_mint(&input.sender_address, token_id),
                self.get_token_accounts_by_mint(&input.destination_address, token_id)
            )?;
            let sender_token_account = sender_accounts.value.first().ok_or("Sender token address is empty")?;
            let sender_token_address = sender_token_account.pubkey.clone();
            let token_program = get_token_program_id_by_address(&sender_token_account.account.owner).ok();
            let recipient_token_address = recipient_accounts.value.first().map(|account| account.pubkey.clone());

            Ok(TransactionLoadMetadata::Solana {
                sender_token_address: Some(sender_token_address),
                recipient_token_address,
                token_program,
                block_hash: block_hash.value.blockhash,
            })
        } else {
            let block_hash = self.get_latest_blockhash().await?.value.blockhash;
            Ok(TransactionLoadMetadata::Solana {
                sender_token_address: None,
                recipient_token_address: None,
                token_program: Some(SolanaTokenProgramId::Token),
                block_hash,
            })
        }
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let fee = calculate_transaction_fee(&input.input_type, &input.gas_price, input.metadata.get_recipient_token_address()?);
        Ok(TransactionLoadData { fee, metadata: input.metadata })
    }

    async fn get_transaction_fee_rates(&self, input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let prioritization_fees = self.get_recent_prioritization_fees().await?;
        Ok(calculate_fee_rates(&input_type, &prioritization_fees))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::create_solana_test_client;
    use primitives::Asset;

    #[tokio::test]
    async fn test_solana_get_transaction_fee_rates() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let rates = client.get_transaction_fee_rates(TransactionInputType::Transfer(Asset::mock_sol())).await?;
        assert!(rates.len() == 3);
        Ok(())
    }
}
