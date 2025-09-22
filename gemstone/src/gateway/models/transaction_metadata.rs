use crate::gateway::{GemStakeData, GemUTXO};
use crate::models::GemSolanaTokenProgramId;
use primitives::TransactionLoadMetadata;
use std::collections::HashMap;

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransactionLoadMetadata {
    None,
    Solana {
        sender_token_address: Option<String>,
        recipient_token_address: Option<String>,
        token_program: Option<GemSolanaTokenProgramId>,
        block_hash: String,
    },
    Ton {
        sender_token_address: Option<String>,
        recipient_token_address: Option<String>,
        sequence: u64,
    },
    Cosmos {
        account_number: u64,
        sequence: u64,
        chain_id: String,
    },
    Bitcoin {
        utxos: Vec<GemUTXO>,
    },
    Cardano {
        utxos: Vec<GemUTXO>,
    },
    Evm {
        nonce: u64,
        chain_id: u64,
        stake_data: Option<GemStakeData>,
    },
    Near {
        sequence: u64,
        block_hash: String,
    },
    Stellar {
        sequence: u64,
        is_destination_address_exist: bool,
    },
    Xrp {
        sequence: u64,
        block_number: u64,
    },
    Algorand {
        sequence: u64,
        block_hash: String,
        chain_id: String,
    },
    Aptos {
        sequence: u64,
    },
    Polkadot {
        sequence: u64,
        genesis_hash: String,
        block_hash: String,
        block_number: u64,
        spec_version: u64,
        transaction_version: u64,
        period: u64,
    },
    Tron {
        block_number: u64,
        block_version: u64,
        block_timestamp: u64,
        transaction_tree_root: String,
        parent_hash: String,
        witness_address: String,
        votes: HashMap<String, u64>,
    },
    Sui {
        message_bytes: String,
    },
    Hyperliquid {
        approve_agent_required: bool,
        approve_referral_required: bool,
        approve_builder_required: bool,
        builder_fee_bps: u32,
        agent_address: String,
        agent_private_key: String,
    },
}

impl From<TransactionLoadMetadata> for GemTransactionLoadMetadata {
    fn from(value: TransactionLoadMetadata) -> Self {
        match value {
            TransactionLoadMetadata::None => GemTransactionLoadMetadata::None,
            TransactionLoadMetadata::Solana {
                sender_token_address,
                recipient_token_address,
                token_program,
                block_hash,
            } => GemTransactionLoadMetadata::Solana {
                sender_token_address,
                recipient_token_address,
                token_program: token_program.map(|tp| tp.into()),
                block_hash,
            },
            TransactionLoadMetadata::Ton {
                sender_token_address,
                recipient_token_address,
                sequence,
            } => GemTransactionLoadMetadata::Ton {
                sender_token_address,
                recipient_token_address,
                sequence,
            },
            TransactionLoadMetadata::Cosmos {
                account_number,
                sequence,
                chain_id,
            } => GemTransactionLoadMetadata::Cosmos {
                account_number,
                sequence,
                chain_id,
            },
            TransactionLoadMetadata::Bitcoin { utxos } => GemTransactionLoadMetadata::Bitcoin {
                utxos: utxos.into_iter().map(|utxo| utxo.into()).collect(),
            },
            TransactionLoadMetadata::Cardano { utxos } => GemTransactionLoadMetadata::Cardano {
                utxos: utxos.into_iter().map(|utxo| utxo.into()).collect(),
            },
            TransactionLoadMetadata::Evm { nonce, chain_id, stake_data } => GemTransactionLoadMetadata::Evm {
                nonce,
                chain_id,
                stake_data: stake_data.map(|sd| sd.into()),
            },
            TransactionLoadMetadata::Near { sequence, block_hash } => GemTransactionLoadMetadata::Near { sequence, block_hash },
            TransactionLoadMetadata::Stellar {
                sequence,
                is_destination_address_exist,
            } => GemTransactionLoadMetadata::Stellar {
                sequence,
                is_destination_address_exist,
            },
            TransactionLoadMetadata::Xrp { sequence, block_number } => GemTransactionLoadMetadata::Xrp { sequence, block_number },
            TransactionLoadMetadata::Algorand {
                sequence,
                block_hash,
                chain_id,
            } => GemTransactionLoadMetadata::Algorand {
                sequence,
                block_hash,
                chain_id,
            },
            TransactionLoadMetadata::Aptos { sequence } => GemTransactionLoadMetadata::Aptos { sequence },
            TransactionLoadMetadata::Polkadot {
                sequence,
                genesis_hash,
                block_hash,
                block_number,
                spec_version,
                transaction_version,
                period,
            } => GemTransactionLoadMetadata::Polkadot {
                sequence,
                genesis_hash,
                block_hash,
                block_number,
                spec_version,
                transaction_version,
                period,
            },
            TransactionLoadMetadata::Tron {
                block_number,
                block_version,
                block_timestamp,
                transaction_tree_root,
                parent_hash,
                witness_address,
                votes,
            } => GemTransactionLoadMetadata::Tron {
                block_number,
                block_version,
                block_timestamp,
                transaction_tree_root,
                parent_hash,
                witness_address,
                votes,
            },
            TransactionLoadMetadata::Sui { message_bytes } => GemTransactionLoadMetadata::Sui { message_bytes },
            TransactionLoadMetadata::Hyperliquid {
                approve_agent_required,
                approve_referral_required,
                approve_builder_required,
                builder_fee_bps,
                agent_address,
                agent_private_key,
            } => GemTransactionLoadMetadata::Hyperliquid {
                approve_agent_required,
                approve_referral_required,
                approve_builder_required,
                builder_fee_bps,
                agent_address,
                agent_private_key,
            },
        }
    }
}

impl From<GemTransactionLoadMetadata> for TransactionLoadMetadata {
    fn from(value: GemTransactionLoadMetadata) -> Self {
        match value {
            GemTransactionLoadMetadata::None => TransactionLoadMetadata::None,
            GemTransactionLoadMetadata::Solana {
                sender_token_address,
                recipient_token_address,
                token_program,
                block_hash,
            } => TransactionLoadMetadata::Solana {
                sender_token_address,
                recipient_token_address,
                token_program: token_program.map(|tp| tp.into()),
                block_hash,
            },
            GemTransactionLoadMetadata::Ton {
                sender_token_address,
                recipient_token_address,
                sequence,
            } => TransactionLoadMetadata::Ton {
                sender_token_address,
                recipient_token_address,
                sequence,
            },
            GemTransactionLoadMetadata::Cosmos {
                account_number,
                sequence,
                chain_id,
            } => TransactionLoadMetadata::Cosmos {
                account_number,
                sequence,
                chain_id,
            },
            GemTransactionLoadMetadata::Bitcoin { utxos } => TransactionLoadMetadata::Bitcoin {
                utxos: utxos.into_iter().map(|utxo| utxo.into()).collect(),
            },
            GemTransactionLoadMetadata::Cardano { utxos } => TransactionLoadMetadata::Cardano {
                utxos: utxos.into_iter().map(|utxo| utxo.into()).collect(),
            },
            GemTransactionLoadMetadata::Evm { nonce, chain_id, stake_data } => TransactionLoadMetadata::Evm {
                nonce,
                chain_id,
                stake_data: stake_data.map(|sd| sd.into()),
            },
            GemTransactionLoadMetadata::Near { sequence, block_hash } => TransactionLoadMetadata::Near { sequence, block_hash },
            GemTransactionLoadMetadata::Stellar {
                sequence,
                is_destination_address_exist,
            } => TransactionLoadMetadata::Stellar {
                sequence,
                is_destination_address_exist,
            },
            GemTransactionLoadMetadata::Xrp { sequence, block_number } => TransactionLoadMetadata::Xrp { sequence, block_number },
            GemTransactionLoadMetadata::Algorand {
                sequence,
                block_hash,
                chain_id,
            } => TransactionLoadMetadata::Algorand {
                sequence,
                block_hash,
                chain_id,
            },
            GemTransactionLoadMetadata::Aptos { sequence } => TransactionLoadMetadata::Aptos { sequence },
            GemTransactionLoadMetadata::Polkadot {
                sequence,
                genesis_hash,
                block_hash,
                block_number,
                spec_version,
                transaction_version,
                period,
            } => TransactionLoadMetadata::Polkadot {
                sequence,
                genesis_hash,
                block_hash,
                block_number,
                spec_version,
                transaction_version,
                period,
            },
            GemTransactionLoadMetadata::Tron {
                block_number,
                block_version,
                block_timestamp,
                transaction_tree_root,
                parent_hash,
                witness_address,
                votes,
            } => TransactionLoadMetadata::Tron {
                block_number,
                block_version,
                block_timestamp,
                transaction_tree_root,
                parent_hash,
                witness_address,
                votes,
            },
            GemTransactionLoadMetadata::Sui { message_bytes } => TransactionLoadMetadata::Sui { message_bytes },
            GemTransactionLoadMetadata::Hyperliquid {
                approve_agent_required,
                approve_referral_required,
                approve_builder_required,
                builder_fee_bps,
                agent_address,
                agent_private_key,
            } => TransactionLoadMetadata::Hyperliquid {
                approve_agent_required,
                approve_referral_required,
                approve_builder_required,
                builder_fee_bps,
                agent_address,
                agent_private_key,
            },
        }
    }
}
