use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use std::error::Error;

use crate::provider::preload_mapper::{calculate_fee_rates, calculate_transaction_fee};
use gem_client::Client;
use primitives::{
    Chain, FeeRate, SolanaNftStandard, SolanaTokenProgramId, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput,
};

use crate::{METAPLEX_CORE_PROGRAM, get_token_program_id_by_address, metaplex_core, rpc::client::SolanaClient};

struct SolanaNftPreload {
    token_program: Option<SolanaTokenProgramId>,
    standard: SolanaNftStandard,
}

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for SolanaClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let TransactionPreloadInput {
            input_type,
            sender_address,
            destination_address,
        } = input;

        let (sender_lookup, recipient_lookup) = match input_type {
            TransactionInputType::Swap(_, _, _) => (sender_address.as_str(), sender_address.as_str()),
            _ => (sender_address.as_str(), destination_address.as_str()),
        };

        let (sender_mint, recipient_mint, token_program, nft) = match &input_type {
            TransactionInputType::TransferNft(_, nft_asset) => {
                let SolanaNftPreload { token_program, standard } = self.detect_solana_nft(&nft_asset.token_id).await?;
                let mint = token_program.is_some().then_some(nft_asset.token_id.as_str());
                (mint, mint, token_program, Some(standard))
            }
            _ => {
                let source = input_type.get_asset();
                let recipient = input_type.get_recipient_asset();
                let sender_mint = source.id.token_id.as_deref();
                let recipient_mint = match recipient.chain() {
                    Chain::Solana => recipient.id.token_id.as_deref(),
                    _ => None,
                };
                (sender_mint, recipient_mint, SolanaTokenProgramId::from_asset_type(&source.asset_type), None)
            }
        };

        let sender_token_future = async {
            match sender_mint {
                Some(mint) => self.find_token_account(sender_lookup, mint).await,
                None => Ok(None),
            }
        };
        let recipient_token_future = async {
            match recipient_mint {
                Some(mint) => self.find_token_account(recipient_lookup, mint).await,
                None => Ok(None),
            }
        };

        let (block_hash, sender_token_address, recipient_token_address) = futures::try_join!(self.get_latest_blockhash(), sender_token_future, recipient_token_future)?;

        if let TransactionInputType::TransferNft(_, _) = input_type
            && sender_mint.is_some()
            && sender_token_address.is_none()
        {
            return Err("sender does not own Solana NFT token account".into());
        }

        Ok(TransactionLoadMetadata::Solana {
            sender_token_address,
            recipient_token_address,
            token_program,
            nft,
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

#[cfg(feature = "rpc")]
impl<C: Client + Clone> SolanaClient<C> {
    async fn detect_solana_nft(&self, mint: &str) -> Result<SolanaNftPreload, Box<dyn Error + Sync + Send>> {
        let account = self.get_account_info_base64(mint).await?.value.ok_or("Solana NFT account not found")?;
        if account.owner == METAPLEX_CORE_PROGRAM {
            let data = account.data.first().ok_or("missing Metaplex Core asset data")?;
            let collection = metaplex_core::decode_asset(data)?.collection().map(|pubkey| pubkey.to_base58());
            return Ok(SolanaNftPreload {
                token_program: None,
                standard: SolanaNftStandard::Core { collection },
            });
        }
        let token_program = get_token_program_id_by_address(&account.owner).ok_or_else(|| format!("unsupported Solana NFT owner program: {}", account.owner))?;
        let metadata = self.get_metaplex_metadata(mint).await.ok();
        let standard = match metadata.filter(|m| m.is_programmable()) {
            Some(metadata) => SolanaNftStandard::ProgrammableNonFungible {
                rule_set: metadata.rule_set().map(|pubkey| pubkey.to_base58()),
            },
            None => SolanaNftStandard::NonFungible,
        };
        Ok(SolanaNftPreload {
            token_program: Some(token_program),
            standard,
        })
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_EMPTY_ADDRESS, create_solana_test_client};
    use primitives::swap::SwapData;
    use primitives::testkit::signer_mock::TEST_SOLANA_SENDER;
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
            sender_address: TEST_SOLANA_SENDER.to_string(),
            destination_address: TEST_SOLANA_SENDER.to_string(),
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
            sender_address: TEST_SOLANA_SENDER.to_string(),
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
            sender_address: TEST_SOLANA_SENDER.to_string(),
            destination_address: TEST_SOLANA_SENDER.to_string(),
        };

        let result = client.get_transaction_preload(input).await?;

        assert!(result.get_block_hash()?.len() == 44);
        assert!(result.get_recipient_token_address()?.is_none());
        assert_eq!(result.get_recipient_token_address()?, None);
        assert_eq!(result.get_sender_token_address()?, Some("HEeranxp3y7kVQKVSLdZW1rUmnbs7bAtUTMu8o88Jash".to_string()));

        if let TransactionLoadMetadata::Solana { token_program, .. } = &result {
            assert_eq!(token_program.as_ref(), Some(&SolanaTokenProgramId::Token));
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
            sender_address: TEST_SOLANA_SENDER.to_string(),
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
