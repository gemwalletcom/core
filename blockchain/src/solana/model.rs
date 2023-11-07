
use num_bigint::BigUint;
use typeshare::typeshare;
use serde::{Serialize, Deserialize};
use primitives::BigIntValue;

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub blockhash: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub fee: u64,
    pub log_messages: Vec<String>,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub pre_token_balances: Vec<TokenBalance>,
    pub post_token_balances: Vec<TokenBalance>,
}

impl Meta {
    pub fn get_pre_token_balance(&self, account_index: i64) -> Option<TokenBalance> {
        self.pre_token_balances.iter().find(|b| b.account_index == account_index).cloned()
    }

    pub fn get_post_token_balance(&self, account_index: i64) -> Option<TokenBalance> {
        self.post_token_balances.iter().find(|b| b.account_index == account_index).cloned()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub ok: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub account_keys: Vec<String>,
    //pub instructions: Vec<Instruction>,
    //pub recent_blockhash: String,
}

// #[derive(Debug, Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Instruction {
//     pub accounts: Vec<u64>,
//     pub data: String,
//     pub program_id_index: u64,
// }

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub message: Message,
    pub signatures: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransaction {
    pub meta: Meta,
    pub transaction: Transaction,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransactions {
    pub transactions: Vec<BlockTransaction>
}


#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    pub account_index: i64,
    pub mint: String,
    pub owner: String,
    pub ui_token_amount: TokenAmount,
}

impl TokenBalance {
    pub fn get_amount(&self) -> BigUint {
        self.ui_token_amount.amount.value.clone()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub amount: BigIntValue,
    pub decimals: i32,
}
