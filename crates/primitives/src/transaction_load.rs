use crate::{Asset, SolanaTokenProgramId, TransactionPreloadInput, UTXO};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::{AsRefStr, Display, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, Display, EnumString, PartialEq, Eq, Hash)]
pub enum FeeOption {
    TokenAccountCreation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StakeOperation {
    Delegate(Asset, String),
    Undelegate(Asset, String),
    Redelegate(Asset, String, String),
    WithdrawRewards(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionInputType {
    Transfer(Asset),
    Swap(Asset, Asset),
    Stake(StakeOperation),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPrice {
    pub gas_price: BigInt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLoadInput {
    pub input_type: TransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
    pub value: String,
    pub gas_price: GasPrice,
    pub memo: Option<String>,
    pub is_max_value: bool,
    pub metadata: TransactionLoadMetadata,
}

impl TransactionLoadInput {
    pub fn to_preload_input(&self) -> TransactionPreloadInput {
        TransactionPreloadInput {
            sender_address: self.sender_address.clone(),
            destination_address: self.destination_address.clone(),
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SignerInputBlock {
    pub number: i64,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionFee {
    pub fee: BigInt,
    pub gas_price: BigInt,
    pub gas_limit: BigInt,
    pub options: HashMap<FeeOption, String>,
}

impl Default for TransactionFee {
    fn default() -> Self {
        Self {
            fee: BigInt::from(0),
            gas_price: BigInt::from(0),
            gas_limit: BigInt::from(0),
            options: HashMap::new(),
        }
    }
}

impl TransactionFee {
    pub fn new_from_fee(fee: BigInt) -> Self {
        Self {
            fee,
            gas_price: BigInt::from(1),
            gas_limit: BigInt::from(1),
            options: HashMap::new(),
        }
    }
}
impl TransactionLoadMetadata {
    pub fn get_sequence(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            TransactionLoadMetadata::Solana { sequence, .. } => Ok(*sequence),
            TransactionLoadMetadata::Ton { sequence, .. } => Ok(*sequence),
            TransactionLoadMetadata::Cosmos { sequence, .. } => Ok(*sequence),
            TransactionLoadMetadata::Near { sequence, .. } => Ok(*sequence),
            TransactionLoadMetadata::Stellar { sequence, .. } => Ok(*sequence),
            TransactionLoadMetadata::Xrp { sequence } => Ok(*sequence),
            TransactionLoadMetadata::Algorand { sequence } => Ok(*sequence),
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
            _ => Err("Block number not available for this metadata type".into()),
        }
    }

    pub fn get_block_hash(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            TransactionLoadMetadata::Near { block_hash, .. } => Ok(block_hash.clone()),
            TransactionLoadMetadata::Polkadot { block_hash, .. } => Ok(block_hash.clone()),
            _ => Err("Block hash not available for this metadata type".into()),
        }
    }

    pub fn get_chain_id(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            TransactionLoadMetadata::Cosmos { chain_id, .. } => Ok(chain_id.clone()),
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
            TransactionLoadMetadata::Near {
                is_destination_address_exist, ..
            } => Ok(*is_destination_address_exist),
            TransactionLoadMetadata::Stellar {
                is_destination_address_exist, ..
            } => Ok(*is_destination_address_exist),
            _ => Err("Destination existence flag not available for this metadata type".into()),
        }
    }
}

impl TransactionFee {
    pub fn calculate(gas_limit: u64, gas_price: &GasPrice) -> Self {
        let gas_limit_bigint = BigInt::from(gas_limit);
        let total_fee = &gas_price.gas_price * &gas_limit_bigint;

        Self {
            fee: total_fee,
            gas_price: gas_price.gas_price.clone(),
            gas_limit: gas_limit_bigint,
            options: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerInputToken {
    pub sender_token_address: String,
    pub recipient_token_address: Option<String>,
    pub token_program: SolanaTokenProgramId,
}

impl Default for SignerInputToken {
    fn default() -> Self {
        Self {
            sender_token_address: String::new(),
            recipient_token_address: None,
            token_program: SolanaTokenProgramId::Token,
        }
    }
}

impl SignerInputToken {
    pub fn new_sender_token_address(address: String) -> Self {
        Self {
            sender_token_address: address,
            recipient_token_address: None,
            token_program: SolanaTokenProgramId::Token,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionLoadMetadata {
    None,
    Solana {
        sender_token_address: String,
        recipient_token_address: Option<String>,
        token_program: SolanaTokenProgramId,
        sequence: u64,
    },
    Ton {
        jetton_wallet_address: String,
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
        is_destination_address_exist: bool,
    },
    Stellar {
        sequence: u64,
        is_destination_address_exist: bool,
    },
    Xrp {
        sequence: u64,
    },
    Algorand {
        sequence: u64,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLoadData {
    pub fee: TransactionFee,
    pub metadata: TransactionLoadMetadata,
}

impl TransactionLoadData {
    pub fn new_from(&self, fee: TransactionFee) -> Self {
        Self {
            fee,
            metadata: self.metadata.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_fee_calculate() {
        let gas_price = GasPrice {
            gas_price: BigInt::from(100u64),
        };
        let gas_limit = 1000u64;

        let fee = TransactionFee::calculate(gas_limit, &gas_price);

        assert_eq!(fee.fee, BigInt::from(100000u64)); // 100 * 1000
        assert_eq!(fee.gas_price, BigInt::from(100u64));
        assert_eq!(fee.gas_limit, BigInt::from(1000u64));
    }
}
