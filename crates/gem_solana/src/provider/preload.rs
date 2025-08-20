use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{SignerInputToken, TransactionLoadData, TransactionLoadInput, TransactionPreload, TransactionPreloadInput, TransactionInputType, SolanaTokenProgramId, AssetSubtype};
use primitives::transaction_load::TransactionLoadMetadata;

use crate::{provider::preload_mapper, rpc::client::SolanaClient, get_token_program_id_by_address};

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainPreload for SolanaClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        let blockhash_result = self.get_latest_blockhash().await?;
        Ok(TransactionPreload::builder()
            .block_hash(blockhash_result.value.blockhash)
            .build())
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let prioritization_fees = self.get_recent_prioritization_fees().await?;
        let fee = preload_mapper::calculate_transaction_fee(&input.input_type, &input.gas_price, &prioritization_fees);
        
        let metadata = match &input.input_type {
            TransactionInputType::Transfer(asset) => {
                match asset.id.token_subtype() {
                    AssetSubtype::TOKEN => {
                        if let Some(token_id) = &asset.id.token_id {
                            let token_info = self.get_token_transfer_info(
                                token_id,
                                &input.sender_address,
                                &input.destination_address
                            ).await?;
                            Some(TransactionLoadMetadata::Solana {
                                sender_token_address: token_info.sender_token_address,
                                recipient_token_address: token_info.recipient_token_address,
                                token_program: token_info.token_program,
                                sequence: input.sequence,
                            })
                        } else {
                            Some(TransactionLoadMetadata::Solana {
                                sender_token_address: String::new(),
                                recipient_token_address: None,
                                token_program: SolanaTokenProgramId::Token,
                                sequence: input.sequence,
                            })
                        }
                    },
                    AssetSubtype::NATIVE => Some(TransactionLoadMetadata::Solana {
                        sender_token_address: String::new(),
                        recipient_token_address: None,
                        token_program: SolanaTokenProgramId::Token,
                        sequence: input.sequence,
                    }),
                }
            },
            _ => Some(TransactionLoadMetadata::Solana {
                sender_token_address: String::new(),
                recipient_token_address: None,
                token_program: SolanaTokenProgramId::Token,
                sequence: input.sequence,
            }),
        };

        Ok(TransactionLoadData {
            fee,
            metadata: metadata.unwrap_or(TransactionLoadMetadata::Solana {
                sender_token_address: String::new(),
                recipient_token_address: None,
                token_program: SolanaTokenProgramId::Token,
                sequence: input.sequence,
            }),
        })
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
        let sender_token_account = sender_accounts.value.first()
            .ok_or("Sender token address is empty")?;
        
        let sender_token_address = sender_token_account.pubkey.clone();
        let token_program = get_token_program_id(&sender_token_account.account.owner)?;
        
        let recipient_token_address = recipient_accounts.value.first()
            .map(|account| account.pubkey.clone());

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