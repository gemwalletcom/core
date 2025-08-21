use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{
    AssetSubtype, SignerInputToken, SolanaTokenProgramId, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput,
};

use crate::{get_token_program_id_by_address, provider::preload_mapper, rpc::client::SolanaClient};

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainPreload for SolanaClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        // For Solana, we need to get the sequence number (account nonce)
        // For now, use a default sequence - this would normally come from RPC
        let sequence = 0;

        Ok(TransactionLoadMetadata::Solana {
            sender_token_address: String::new(),        // Will be populated during load
            recipient_token_address: None,              // Will be populated during load
            token_program: SolanaTokenProgramId::Token, // Default
            sequence,
        })
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let prioritization_fees = self.get_recent_prioritization_fees().await?;
        let fee = preload_mapper::calculate_transaction_fee(&input.input_type, &input.gas_price, &prioritization_fees);

        let sequence = match &input.metadata {
            TransactionLoadMetadata::Solana { sequence, .. } => *sequence,
            _ => return Err("Invalid metadata type for Solana".into()),
        };

        let metadata = match &input.input_type {
            TransactionInputType::Transfer(asset) => match asset.id.token_subtype() {
                AssetSubtype::TOKEN => {
                    if let Some(token_id) = &asset.id.token_id {
                        let token_info = self
                            .get_token_transfer_info(token_id, &input.sender_address, &input.destination_address)
                            .await?;
                        TransactionLoadMetadata::Solana {
                            sender_token_address: token_info.sender_token_address,
                            recipient_token_address: token_info.recipient_token_address,
                            token_program: token_info.token_program,
                            sequence,
                        }
                    } else {
                        input.metadata
                    }
                }
                AssetSubtype::NATIVE => input.metadata,
            },
            _ => input.metadata,
        };

        Ok(TransactionLoadData { fee, metadata })
    }
}

impl<C: Client + Clone> SolanaClient<C> {
    async fn get_token_transfer_info(
        &self,
        token_id: &str,
        sender_address: &str,
        recipient_address: &str,
    ) -> Result<SignerInputToken, Box<dyn Error + Sync + Send>> {
        let sender_accounts = self.get_token_accounts_by_mint(sender_address, token_id).await?;
        let recipient_accounts = self.get_token_accounts_by_mint(recipient_address, token_id).await?;
        let sender_token_account = sender_accounts.value.first().ok_or("Sender token address is empty")?;

        let sender_token_address = sender_token_account.pubkey.clone();
        let token_program = get_token_program_id(&sender_token_account.account.owner)?;

        let recipient_token_address = recipient_accounts.value.first().map(|account| account.pubkey.clone());

        Ok(SignerInputToken {
            sender_token_address,
            recipient_token_address,
            token_program,
        })
    }
}

fn get_token_program_id(owner: &str) -> Result<SolanaTokenProgramId, Box<dyn Error + Sync + Send>> {
    match get_token_program_id_by_address(owner) {
        Some(token_program_id) => Ok(token_program_id),
        None => Err("Unknown token program id".into()),
    }
}
