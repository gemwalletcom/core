use crate::{Asset, TransactionPreloadInput, UTXO};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

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
    pub block_number: i64,
    pub chain_id: String,
    pub utxos: Vec<UTXO>,
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
}

impl Default for TransactionFee {
    fn default() -> Self {
        Self {
            fee: BigInt::from(0),
            gas_price: BigInt::from(0),
            gas_limit: BigInt::from(0),
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLoadData {
    pub account_number: u64,
    pub sequence: u64,
    pub fee: TransactionFee,
}

impl TransactionLoadData {
    pub fn builder() -> TransactionLoadDataBuilder {
        TransactionLoadDataBuilder {
            account_number: None,
            sequence: None,
            fee: None,
        }
    }
}

pub struct TransactionLoadDataBuilder {
    account_number: Option<u64>,
    sequence: Option<u64>,
    fee: Option<TransactionFee>,
}

impl TransactionLoadDataBuilder {
    pub fn account_number(mut self, account_number: u64) -> Self {
        self.account_number = Some(account_number);
        self
    }

    pub fn sequence(mut self, sequence: u64) -> Self {
        self.sequence = Some(sequence);
        self
    }

    pub fn fee(mut self, fee: TransactionFee) -> Self {
        self.fee = Some(fee);
        self
    }

    pub fn build(self) -> TransactionLoadData {
        TransactionLoadData {
            account_number: self.account_number.expect("account_number is required"),
            sequence: self.sequence.expect("sequence is required"),
            fee: self.fee.expect("fee is required"),
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
