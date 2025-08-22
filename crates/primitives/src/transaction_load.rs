use crate::solana_token_program::SolanaTokenProgramId;
use crate::{Asset, Delegation, DelegationValidator, GasPriceType, TransactionPreloadInput, UTXO};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::{AsRefStr, Display, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, Display, EnumString, PartialEq, Eq, Hash)]
pub enum FeeOption {
    TokenAccountCreation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StakeType {
    Delegate(DelegationValidator),
    Undelegate(DelegationValidator),
    Redelegate(Delegation, DelegationValidator),
    WithdrawRewards(Vec<DelegationValidator>),
    Withdraw(Delegation),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum TransactionInputType {
    Transfer(Asset),
    Swap(Asset, Asset),
    Stake(Asset, StakeType),
}

impl TransactionInputType {
    pub fn get_asset(&self) -> &Asset {
        match self {
            TransactionInputType::Transfer(asset) => asset,
            TransactionInputType::Swap(_, asset) => asset,
            TransactionInputType::Stake(asset, _) => asset,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLoadInput {
    pub input_type: TransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
    pub value: String,
    pub gas_price: GasPriceType,
    pub memo: Option<String>,
    pub is_max_value: bool,
    pub metadata: TransactionLoadMetadata,
}

impl TransactionLoadInput {
    pub fn default_fee(&self) -> TransactionFee {
        TransactionFee::new_from_fee(self.gas_price.total_fee())
    }
}

impl TransactionLoadInput {
    pub fn to_preload_input(&self) -> TransactionPreloadInput {
        TransactionPreloadInput {
            asset: self.input_type.get_asset().clone(),
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
    pub gas_price_type: GasPriceType,
    pub gas_limit: BigInt,
    pub options: HashMap<FeeOption, BigInt>,
}

impl Default for TransactionFee {
    fn default() -> Self {
        Self {
            fee: BigInt::from(0),
            gas_price_type: GasPriceType::regular(BigInt::from(0)),
            gas_limit: BigInt::from(0),
            options: HashMap::new(),
        }
    }
}

impl TransactionFee {
    pub fn new_from_fee(fee: BigInt) -> Self {
        Self {
            fee: fee.clone(),
            gas_price_type: GasPriceType::regular(fee),
            gas_limit: BigInt::from(0),
            options: HashMap::new(),
        }
    }
    pub fn new_from_gas_price_and_limit(gas_price: BigInt, gas_limit: BigInt) -> Self {
        Self {
            fee: gas_price.clone() * &gas_limit,
            gas_price_type: GasPriceType::regular(gas_price),
            gas_limit,
            options: HashMap::new(),
        }
    }
    pub fn new_from_fee_with_option(fee: BigInt, option: FeeOption, option_value: BigInt) -> Self {
        Self {
            fee: fee.clone(),
            gas_price_type: GasPriceType::regular(fee.clone()),
            gas_limit: BigInt::from(0),
            options: HashMap::from([(option, option_value)]),
        }
    }
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

impl TransactionFee {
    pub fn calculate(gas_limit: u64, gas_price_type: &GasPriceType) -> Self {
        let gas_limit = BigInt::from(gas_limit);
        let gas_price = gas_price_type.gas_price();
        let total_fee = gas_price.clone() * &gas_limit;

        Self {
            fee: total_fee,
            gas_price_type: gas_price_type.clone(),
            gas_limit,
            options: HashMap::new(),
        }
    }
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
        let gas_price_type = GasPriceType::regular(BigInt::from(100u64));
        let gas_limit = 1000u64;

        let fee = TransactionFee::calculate(gas_limit, &gas_price_type);

        assert_eq!(fee.fee, BigInt::from(100000u64)); // 100 * 1000
        assert_eq!(fee.gas_price_type.gas_price(), BigInt::from(100u64));
        assert_eq!(fee.gas_limit, BigInt::from(1000u64));
    }
}
