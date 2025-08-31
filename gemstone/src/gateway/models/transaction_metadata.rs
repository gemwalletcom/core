use crate::gateway::GemUTXO;
use num_bigint::BigInt;
use primitives::solana_token_program::SolanaTokenProgramId;
use primitives::{TransactionLoadMetadata, transaction_load_metadata::SuiCoin};
use std::collections::HashMap;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSuiCoin {
    pub coin_type: String,
    pub coin_object_id: String,
    pub balance: String,
    pub version: String,
    pub digest: String,
}

impl From<SuiCoin> for GemSuiCoin {
    fn from(value: SuiCoin) -> Self {
        Self {
            coin_type: value.coin_type,
            coin_object_id: value.coin_object_id,
            balance: value.balance.to_string(),
            version: value.version,
            digest: value.digest,
        }
    }
}

impl From<GemSuiCoin> for SuiCoin {
    fn from(value: GemSuiCoin) -> Self {
        Self {
            coin_type: value.coin_type,
            coin_object_id: value.coin_object_id,
            balance: value.balance.parse::<BigInt>().unwrap_or_default(),
            version: value.version,
            digest: value.digest,
        }
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemSolanaTokenProgramId {
    Token,
    Token2022,
}

impl From<SolanaTokenProgramId> for GemSolanaTokenProgramId {
    fn from(value: SolanaTokenProgramId) -> Self {
        match value {
            SolanaTokenProgramId::Token => GemSolanaTokenProgramId::Token,
            SolanaTokenProgramId::Token2022 => GemSolanaTokenProgramId::Token2022,
        }
    }
}

impl From<GemSolanaTokenProgramId> for SolanaTokenProgramId {
    fn from(value: GemSolanaTokenProgramId) -> Self {
        match value {
            GemSolanaTokenProgramId::Token => SolanaTokenProgramId::Token,
            GemSolanaTokenProgramId::Token2022 => SolanaTokenProgramId::Token2022,
        }
    }
}

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
            TransactionLoadMetadata::Evm { nonce, chain_id } => GemTransactionLoadMetadata::Evm { nonce, chain_id },
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
            TransactionLoadMetadata::Sui { message_bytes } => {
                GemTransactionLoadMetadata::Sui {
                    message_bytes,
                }
            },
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
            GemTransactionLoadMetadata::Evm { nonce, chain_id } => TransactionLoadMetadata::Evm { nonce, chain_id },
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
            GemTransactionLoadMetadata::Sui { message_bytes } => TransactionLoadMetadata::Sui {
                message_bytes,
            },
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
