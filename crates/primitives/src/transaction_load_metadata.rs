use crate::{solana_token_program::SolanaTokenProgramId, UTXO};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionLoadMetadata {
    None,
    Solana {
        sender_token_address: Option<String>,
        recipient_token_address: Option<String>,
        token_program: Option<SolanaTokenProgramId>,
        block_hash: String,
    },
    Ton {
        jetton_wallet_address: Option<String>,
        sequence: u64,
    },
    Cosmos {
        account_number: u64,
        sequence: u64,
        chain_id: String,
    },
    Bitcoin {
        utxos: Vec<UTXO>,
    },
    Cardano {
        utxos: Vec<UTXO>,
    },
    Evm {
        nonce: u64,
        chain_id: u64,
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
    },
    Sui {
        message_bytes: String,
    },
    Hyperliquid {
        approve_agent_required: bool,
        approve_referral_required: bool,
        approve_builder_required: bool,
        builder_fee_bps: i32,
    },
}

impl TransactionLoadMetadata {
    pub fn get_sequence(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            TransactionLoadMetadata::Ton { sequence, .. } => Ok(*sequence),
            TransactionLoadMetadata::Cosmos { sequence, .. } => Ok(*sequence),
            TransactionLoadMetadata::Near { sequence, .. } => Ok(*sequence),
            TransactionLoadMetadata::Stellar { sequence, .. } => Ok(*sequence),
            TransactionLoadMetadata::Xrp { sequence, .. } => Ok(*sequence),
            TransactionLoadMetadata::Algorand { sequence, .. } => Ok(*sequence),
            TransactionLoadMetadata::Aptos { sequence } => Ok(*sequence),
            TransactionLoadMetadata::Polkadot { sequence, .. } => Ok(*sequence),
            TransactionLoadMetadata::Evm { nonce, .. } => Ok(*nonce),
            _ => Err("Sequence not available for this metadata type".into()),
        }
    }

    pub fn get_block_number(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            TransactionLoadMetadata::Polkadot { block_number, .. } => Ok(*block_number),
            TransactionLoadMetadata::Tron { block_number, .. } => Ok(*block_number),
            TransactionLoadMetadata::Xrp { block_number, .. } => Ok(*block_number),
            _ => Err("Block number not available for this metadata type".into()),
        }
    }

    pub fn get_block_hash(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            TransactionLoadMetadata::Solana { block_hash, .. } => Ok(block_hash.clone()),
            TransactionLoadMetadata::Near { block_hash, .. } => Ok(block_hash.clone()),
            TransactionLoadMetadata::Algorand { block_hash, .. } => Ok(block_hash.clone()),
            TransactionLoadMetadata::Polkadot { block_hash, .. } => Ok(block_hash.clone()),
            _ => Err("Block hash not available for this metadata type".into()),
        }
    }

    pub fn get_chain_id(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            TransactionLoadMetadata::Cosmos { chain_id, .. } => Ok(chain_id.clone()),
            TransactionLoadMetadata::Algorand { chain_id, .. } => Ok(chain_id.clone()),
            TransactionLoadMetadata::Evm { chain_id, .. } => Ok(chain_id.to_string()),
            _ => Err("Chain ID not available for this metadata type".into()),
        }
    }

    pub fn get_utxos(&self) -> Result<Vec<crate::UTXO>, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            TransactionLoadMetadata::Bitcoin { utxos } => Ok(utxos.clone()),
            TransactionLoadMetadata::Cardano { utxos } => Ok(utxos.clone()),
            _ => Err("UTXOs not available for this metadata type".into()),
        }
    }

    pub fn get_is_destination_address_exist(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            TransactionLoadMetadata::Stellar {
                is_destination_address_exist, ..
            } => Ok(*is_destination_address_exist),
            _ => Err("Destination existence flag not available for this metadata type".into()),
        }
    }
}
