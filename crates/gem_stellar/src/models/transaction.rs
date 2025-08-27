use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarTransactionBroadcast {
    pub hash: Option<String>,
    #[serde(rename = "title")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarTransactionStatus {
    pub successful: bool,
    #[serde(deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub fee_charged: BigUint,
    pub hash: String,
}

// RPC models
#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub transaction_successful: bool,
    pub transaction_hash: String,
    #[serde(rename = "type")]
    pub payment_type: String,

    // payment
    pub asset_type: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub amount: Option<String>,

    pub created_at: String,

    // create account
    pub source_account: Option<String>,
    pub funder: Option<String>,
    pub account: Option<String>,
    pub starting_balance: Option<String>,
}

#[cfg(feature = "rpc")]
impl Payment {
    fn amount_formatter(value: &str) -> Option<String> {
        use number_formatter::BigNumberFormatter;
        use primitives::{Asset, Chain};
        BigNumberFormatter::value_from_amount(value, Asset::from_chain(Chain::Stellar).decimals as u32).ok()
    }

    pub fn from_address(&self) -> Option<String> {
        use crate::constants::{TRANSACTION_TYPE_CREATE_ACCOUNT, TRANSACTION_TYPE_PAYMENT};
        match self.payment_type.as_str() {
            TRANSACTION_TYPE_PAYMENT => self.from.clone(),
            TRANSACTION_TYPE_CREATE_ACCOUNT => self.funder.clone(),
            _ => None,
        }
    }

    pub fn to_address(&self) -> Option<String> {
        use crate::constants::{TRANSACTION_TYPE_CREATE_ACCOUNT, TRANSACTION_TYPE_PAYMENT};
        match self.payment_type.as_str() {
            TRANSACTION_TYPE_PAYMENT => self.to.clone(),
            TRANSACTION_TYPE_CREATE_ACCOUNT => self.account.clone(),
            _ => None,
        }
    }

    pub fn get_state(&self) -> primitives::TransactionState {
        use primitives::TransactionState;
        match self.transaction_successful {
            true => TransactionState::Confirmed,
            false => TransactionState::Failed,
        }
    }

    pub fn get_value(&self) -> Option<String> {
        use crate::constants::{TRANSACTION_TYPE_CREATE_ACCOUNT, TRANSACTION_TYPE_PAYMENT};
        match self.payment_type.as_str() {
            TRANSACTION_TYPE_PAYMENT => Self::amount_formatter(self.amount.as_ref()?),
            TRANSACTION_TYPE_CREATE_ACCOUNT => Self::amount_formatter(self.starting_balance.as_ref()?),
            _ => None,
        }
    }

    pub fn get_memo(&self) -> Option<String> {
        None
    }
}
