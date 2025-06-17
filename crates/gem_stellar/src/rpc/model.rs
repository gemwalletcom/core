use core::str;

use number_formatter::BigNumberFormatter;
use primitives::{Asset, Chain, TransactionState};
use serde::{Deserialize, Serialize};

pub const TRANSACTION_TYPE_PAYMENT: &str = "payment";
pub const TRANSACTION_TYPE_CREATE_ACCOUNT: &str = "create_account";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub history_latest_ledger: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedded<T> {
    pub _embedded: Records<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Records<T> {
    pub records: Vec<T>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub closed_at: String,
    pub sequence: i64,
    pub base_fee_in_stroops: i64,
}

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
impl Payment {
    fn amount_formatter(value: &str) -> Option<String> {
        BigNumberFormatter::value_from_amount(value, Asset::from_chain(Chain::Stellar).decimals as u32)
    }

    pub fn from_address(&self) -> Option<String> {
        match self.payment_type.as_str() {
            TRANSACTION_TYPE_PAYMENT => self.from.clone(),
            TRANSACTION_TYPE_CREATE_ACCOUNT => self.funder.clone(),
            _ => None,
        }
    }

    pub fn to_address(&self) -> Option<String> {
        match self.payment_type.as_str() {
            TRANSACTION_TYPE_PAYMENT => self.to.clone(),
            TRANSACTION_TYPE_CREATE_ACCOUNT => self.account.clone(),
            _ => None,
        }
    }

    pub fn get_state(&self) -> TransactionState {
        match self.transaction_successful {
            true => TransactionState::Confirmed,
            false => TransactionState::Failed,
        }
    }

    pub fn get_value(&self) -> Option<String> {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub balances: Vec<Balance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub balance: String,
    pub asset_type: String,
    pub asset_issuer: Option<String>,
}
