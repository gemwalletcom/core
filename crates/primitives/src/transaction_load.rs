use crate::{Asset, TransactionPreloadInput, UTXO, SolanaTokenProgramId};
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
    pub sequence: u64,
    pub block_hash: String,
    pub block_number: u64,
    pub chain_id: String,
    pub utxos: Vec<UTXO>,
    pub memo: Option<String>,
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
        chain_id: String,
        block_hash: String,
        block_number: u64,
    },
    Near {
        sequence: u64,
        block_hash: String,
        is_destination_exist: bool,
    },
    Stellar {
        sequence: u64,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLoadData {
    pub fee: TransactionFee,
    pub metadata: TransactionLoadMetadata,
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
