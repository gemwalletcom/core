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
        let asset = input.input_type.get_asset();

        let (sender_address, destination_address) = match input.input_type {
            TransactionInputType::Transfer(_) => (input.sender_address.clone(), input.destination_address.clone()),
            TransactionInputType::Swap(_, _, _) => (input.sender_address.clone(), input.sender_address.clone()),
            _ => (input.sender_address.clone(), input.destination_address.clone()),
        };

        if let Some(token_id) = &asset.id.token_id {
            let (block_hash, sender_accounts, recipient_accounts) = futures::try_join!(
                self.get_latest_blockhash(),
                self.get_token_accounts_by_mint(&sender_address, token_id),
                self.get_token_accounts_by_mint(&destination_address, token_id)
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
        Ok(TransactionLoadData { 
            fee, 
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self, input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let prioritization_fees = self.get_recent_prioritization_fees().await?;
        Ok(calculate_fee_rates(&input_type, &prioritization_fees))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{create_solana_test_client, TEST_ADDRESS};
    use primitives::Asset;

    #[tokio::test]
    async fn test_solana_get_transaction_fee_rates() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let rates = client.get_transaction_fee_rates(TransactionInputType::Transfer(Asset::mock_sol())).await?;
        assert!(rates.len() == 3);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_solana_transaction_preload_transfer_sol() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Asset::mock_sol()),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: TEST_ADDRESS.to_string(),
        };
        let result = client.get_transaction_preload(input).await?;

        println!("Tranasction load metadata: {:?}", result);

        assert!(result.get_block_hash()?.len() == 44);
        assert!(result.get_sender_token_address()?.is_none());
        assert!(result.get_recipient_token_address()?.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_solana_transaction_preload_transfer_spl_token() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Asset::mock_spl_token()),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: "4BgapREafMMprtU6CehRmH8LUY26PRFmGf7K4S44oSMW".to_string(),
        };

        let result = client.get_transaction_preload(input).await?;

        println!("Tranasction load metadata: {:?}", result);

        assert!(result.get_block_hash()?.len() == 44);
        assert!(result.get_sender_token_address()? == Some("HEeranxp3y7kVQKVSLdZW1rUmnbs7bAtUTMu8o88Jash".to_string()));
        assert!(result.get_recipient_token_address()?.is_none());

        Ok(())
    }
}
