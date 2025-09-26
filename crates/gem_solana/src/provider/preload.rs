use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use std::error::Error;

use crate::provider::preload_mapper::{calculate_fee_rates, calculate_transaction_fee};
use gem_client::Client;
use primitives::{
    AssetType, Chain, FeeRate, SolanaTokenProgramId, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput,
};

use crate::rpc::client::SolanaClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for SolanaClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let TransactionPreloadInput {
            input_type,
            sender_address,
            destination_address,
        } = input;
        let (sender_lookup_address, recipient_lookup_address) = match &input_type {
            TransactionInputType::Transfer(_) | TransactionInputType::TransferNft(_, _) => (&sender_address, &destination_address),
            TransactionInputType::Swap(_, _, _) => (&sender_address, &sender_address),
            _ => (&sender_address, &destination_address),
        };

        let source_asset = input_type.get_asset().clone();
        let recipient_asset = input_type.get_recipient_asset().clone();

        let sender_token_future = async {
            if source_asset.chain() == Chain::Solana
                && let Some(token_id) = source_asset.id.token_id.clone()
            {
                let accounts = self.get_token_accounts_by_mint(sender_lookup_address.as_str(), &token_id).await?;
                return Ok(accounts.value.first().map(|account| account.pubkey.clone()));
            }
            Ok(None)
        };

        let recipient_token_future = async {
            if recipient_asset.chain() == Chain::Solana
                && let Some(token_id) = recipient_asset.id.token_id.clone()
            {
                let accounts = self.get_token_accounts_by_mint(recipient_lookup_address.as_str(), &token_id).await?;
                return Ok(accounts.value.first().map(|account| account.pubkey.clone()));
            }
            Ok(None)
        };

        let (block_hash, sender_token_address, recipient_token_address) =
            futures::try_join!(self.get_latest_blockhash(), sender_token_future, recipient_token_future)?;

        let token_program = match source_asset.asset_type {
            AssetType::SPL => Some(SolanaTokenProgramId::Token),
            AssetType::SPL2022 => Some(SolanaTokenProgramId::Token2022),
            _ => None,
        };

        Ok(TransactionLoadMetadata::Solana {
            sender_token_address,
            recipient_token_address,
            token_program,
            block_hash: block_hash.value.blockhash,
        })
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
    use crate::provider::testkit::{TEST_ADDRESS, TEST_EMPTY_ADDRESS, create_solana_test_client};
    use primitives::swap::SwapData;
    use primitives::{Asset, SwapProvider};

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

    #[tokio::test]
    async fn test_get_solana_transaction_preload_swap_spl_to_erc20() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let swap_data = SwapData::mock_with_provider(SwapProvider::Jupiter);
        let input = TransactionPreloadInput {
            input_type: TransactionInputType::Swap(Asset::mock_spl_token().clone(), Asset::mock_ethereum_usdc().clone(), swap_data),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: TEST_ADDRESS.to_string(),
        };

        let result = client.get_transaction_preload(input).await?;

        assert!(result.get_block_hash()?.len() == 44);
        assert!(result.get_recipient_token_address()?.is_none());
        assert_eq!(result.get_recipient_token_address()?, None);
        assert_eq!(
            result.get_sender_token_address()?,
            Some("HEeranxp3y7kVQKVSLdZW1rUmnbs7bAtUTMu8o88Jash".to_string())
        );

        if let TransactionLoadMetadata::Solana { token_program, .. } = &result {
            assert!(matches!(token_program, Some(SolanaTokenProgramId::Token)));
        } else {
            panic!("expected solana metadata");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_solana_transaction_preload_swap_spl_to_spl_with_empty_destination() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let swap_data = SwapData::mock_with_provider(SwapProvider::Jupiter);
        let input = TransactionPreloadInput {
            input_type: TransactionInputType::Swap(Asset::mock_spl_token(), Asset::mock_spl_token(), swap_data),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: TEST_EMPTY_ADDRESS.to_string(),
        };

        let result = client.get_transaction_preload(input).await?;

        assert!(result.get_block_hash()?.len() == 44);
        let sender_token_address = result.get_sender_token_address()?;
        let recipient_token_address = result.get_recipient_token_address()?;

        assert_eq!(sender_token_address, Some("HEeranxp3y7kVQKVSLdZW1rUmnbs7bAtUTMu8o88Jash".to_string()));
        assert_eq!(recipient_token_address, sender_token_address);

        Ok(())
    }
}
