use crate::{UTXO, solana_token_program::SolanaTokenProgramId, stake_type::StakeData};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_bigint_from_str;

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperliquidOrder {
    pub approve_agent_required: bool,
    pub approve_referral_required: bool,
    pub approve_builder_required: bool,
    pub builder_fee_bps: u32,
    pub agent_address: String,
    pub agent_private_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiCoin {
    pub coin_type: String,
    pub coin_object_id: String,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub balance: BigInt,
    pub version: String,
    pub digest: String,
}

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
        utxos: Vec<UTXO>,
    },
    Cardano {
        utxos: Vec<UTXO>,
    },
    Evm {
        nonce: u64,
        chain_id: u64,
        stake_data: Option<StakeData>,
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
        order: Option<HyperliquidOrder>,
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

    pub fn get_recipient_token_address(&self) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            TransactionLoadMetadata::Solana { recipient_token_address, .. } => Ok(recipient_token_address.clone()),
            TransactionLoadMetadata::Ton { recipient_token_address, .. } => Ok(recipient_token_address.clone()),
            _ => Err("Recipient token address not available for this metadata type".into()),
        }
    }

    pub fn get_sender_token_address(&self) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            TransactionLoadMetadata::Solana { sender_token_address, .. } => Ok(sender_token_address.clone()),
            TransactionLoadMetadata::Ton { sender_token_address, .. } => Ok(sender_token_address.clone()),
            _ => Err("Sender token address not available for this metadata type".into()),
        }
    }

    pub fn get_message_bytes(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            TransactionLoadMetadata::Sui { message_bytes, .. } => Ok(message_bytes.clone()),
            _ => Err("Message bytes not available for this metadata type".into()),
        }
    }
}
