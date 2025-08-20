use crate::gateway::models::GemUTXO;
use primitives::transaction_load::TransactionLoadMetadata;

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemSolanaTokenProgramId {
    Token,
    Token2022,
}

impl From<primitives::SolanaTokenProgramId> for GemSolanaTokenProgramId {
    fn from(value: primitives::SolanaTokenProgramId) -> Self {
        match value {
            primitives::SolanaTokenProgramId::Token => GemSolanaTokenProgramId::Token,
            primitives::SolanaTokenProgramId::Token2022 => GemSolanaTokenProgramId::Token2022,
        }
    }
}

impl From<GemSolanaTokenProgramId> for primitives::SolanaTokenProgramId {
    fn from(value: GemSolanaTokenProgramId) -> Self {
        match value {
            GemSolanaTokenProgramId::Token => primitives::SolanaTokenProgramId::Token,
            GemSolanaTokenProgramId::Token2022 => primitives::SolanaTokenProgramId::Token2022,
        }
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransactionLoadMetadata {
    None,
    Solana {
        sender_token_address: String,
        recipient_token_address: Option<String>,
        token_program: GemSolanaTokenProgramId,
        sequence: i64,
    },
    Ton {
        jetton_wallet_address: String,
        sequence: i64,
    },
    Cosmos {
        account_number: i64,
        sequence: i64,
        chain_id: String,
    },
    Bitcoin {
        utxos: Vec<GemUTXO>,
    },
    Cardano {
        utxos: Vec<GemUTXO>,
    },
    Evm {
        nonce: i64,
        chain_id: i64,
    },
    Near {
        sequence: i64,
        block_hash: String,
        is_destination_address_exist: bool,
    },
    Stellar {
        sequence: i64,
        is_destination_address_exist: bool,
    },
    Xrp {
        sequence: i64,
    },
    Algorand {
        sequence: i64,
    },
    Aptos {
        sequence: i64,
    },
    Polkadot {
        sequence: i64,
        genesis_hash: String,
        block_hash: String,
        block_number: i64,
        spec_version: u64,
        transaction_version: u64,
        period: i64,
    },
    Tron {
        block_number: i64,
        block_version: i64,
        block_timestamp: i64,
        transaction_tree_root: String,
        parent_hash: String,
        witness_address: String,
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
                sequence,
            } => GemTransactionLoadMetadata::Solana {
                sender_token_address,
                recipient_token_address,
                token_program: token_program.into(),
                sequence: sequence as i64,
            },
            TransactionLoadMetadata::Ton { jetton_wallet_address, sequence } => GemTransactionLoadMetadata::Ton { 
                jetton_wallet_address,
                sequence: sequence as i64,
            },
            TransactionLoadMetadata::Cosmos { account_number, sequence, chain_id } => GemTransactionLoadMetadata::Cosmos { 
                account_number: account_number as i64,
                sequence: sequence as i64,
                chain_id,
            },
            TransactionLoadMetadata::Bitcoin { utxos } => GemTransactionLoadMetadata::Bitcoin {
                utxos: utxos.into_iter().map(|utxo| utxo.into()).collect(),
            },
            TransactionLoadMetadata::Cardano { utxos } => GemTransactionLoadMetadata::Cardano {
                utxos: utxos.into_iter().map(|utxo| utxo.into()).collect(),
            },
            TransactionLoadMetadata::Evm { nonce, chain_id } => GemTransactionLoadMetadata::Evm {
                nonce: nonce as i64,
                chain_id: chain_id as i64,
            },
            TransactionLoadMetadata::Near { sequence, block_hash, is_destination_address_exist } => GemTransactionLoadMetadata::Near {
                sequence: sequence as i64,
                block_hash,
                is_destination_address_exist,
            },
            TransactionLoadMetadata::Stellar { sequence, is_destination_address_exist } => GemTransactionLoadMetadata::Stellar {
                sequence: sequence as i64,
                is_destination_address_exist,
            },
            TransactionLoadMetadata::Xrp { sequence } => GemTransactionLoadMetadata::Xrp {
                sequence: sequence as i64,
            },
            TransactionLoadMetadata::Algorand { sequence } => GemTransactionLoadMetadata::Algorand {
                sequence: sequence as i64,
            },
            TransactionLoadMetadata::Aptos { sequence } => GemTransactionLoadMetadata::Aptos {
                sequence: sequence as i64,
            },
            TransactionLoadMetadata::Polkadot {
                sequence,
                genesis_hash,
                block_hash,
                block_number,
                spec_version,
                transaction_version,
                period,
            } => GemTransactionLoadMetadata::Polkadot {
                sequence: sequence as i64,
                genesis_hash,
                block_hash,
                block_number: block_number as i64,
                spec_version,
                transaction_version,
                period: period as i64,
            },
            TransactionLoadMetadata::Tron {
                block_number,
                block_version,
                block_timestamp,
                transaction_tree_root,
                parent_hash,
                witness_address,
            } => GemTransactionLoadMetadata::Tron {
                block_number: block_number as i64,
                block_version: block_version as i64,
                block_timestamp: block_timestamp as i64,
                transaction_tree_root,
                parent_hash,
                witness_address,
            },
        }
    }
}

impl From<GemTransactionLoadMetadata> for TransactionLoadMetadata {
    fn from(value: GemTransactionLoadMetadata) -> Self {
        match value {
            GemTransactionLoadMetadata::None => TransactionLoadMetadata::None,
            GemTransactionLoadMetadata::Solana { sender_token_address, recipient_token_address, token_program, sequence } => {
                TransactionLoadMetadata::Solana {
                    sender_token_address,
                    recipient_token_address,
                    token_program: token_program.into(),
                    sequence: sequence as u64,
                }
            },
            GemTransactionLoadMetadata::Ton { jetton_wallet_address, sequence } => {
                TransactionLoadMetadata::Ton {
                    jetton_wallet_address,
                    sequence: sequence as u64,
                }
            },
            GemTransactionLoadMetadata::Cosmos { account_number, sequence, chain_id } => {
                TransactionLoadMetadata::Cosmos {
                    account_number: account_number as u64,
                    sequence: sequence as u64,
                    chain_id,
                }
            },
            GemTransactionLoadMetadata::Bitcoin { utxos } => {
                TransactionLoadMetadata::Bitcoin {
                    utxos: utxos.into_iter().map(|utxo| utxo.into()).collect(),
                }
            },
            GemTransactionLoadMetadata::Cardano { utxos } => {
                TransactionLoadMetadata::Cardano {
                    utxos: utxos.into_iter().map(|utxo| utxo.into()).collect(),
                }
            },
            GemTransactionLoadMetadata::Evm { nonce, chain_id } => {
                TransactionLoadMetadata::Evm {
                    nonce: nonce as u64,
                    chain_id: chain_id as u64,
                }
            },
            GemTransactionLoadMetadata::Near { sequence, block_hash, is_destination_address_exist } => {
                TransactionLoadMetadata::Near {
                    sequence: sequence as u64,
                    block_hash,
                    is_destination_address_exist,
                }
            },
            GemTransactionLoadMetadata::Stellar { sequence, is_destination_address_exist } => {
                TransactionLoadMetadata::Stellar {
                    sequence: sequence as u64,
                    is_destination_address_exist,
                }
            },
            GemTransactionLoadMetadata::Xrp { sequence } => {
                TransactionLoadMetadata::Xrp {
                    sequence: sequence as u64,
                }
            },
            GemTransactionLoadMetadata::Algorand { sequence } => {
                TransactionLoadMetadata::Algorand {
                    sequence: sequence as u64,
                }
            },
            GemTransactionLoadMetadata::Aptos { sequence } => {
                TransactionLoadMetadata::Aptos {
                    sequence: sequence as u64,
                }
            },
            GemTransactionLoadMetadata::Polkadot { sequence, genesis_hash, block_hash, block_number, spec_version, transaction_version, period } => {
                TransactionLoadMetadata::Polkadot {
                    sequence: sequence as u64,
                    genesis_hash,
                    block_hash,
                    block_number: block_number as u64,
                    spec_version,
                    transaction_version,
                    period: period as u64,
                }
            },
            GemTransactionLoadMetadata::Tron { block_number, block_version, block_timestamp, transaction_tree_root, parent_hash, witness_address } => {
                TransactionLoadMetadata::Tron {
                    block_number: block_number as u64,
                    block_version: block_version as u64,
                    block_timestamp: block_timestamp as u64,
                    transaction_tree_root,
                    parent_hash,
                    witness_address,
                }
            },
        }
    }
}