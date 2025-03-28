use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub blockhash: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub fee: u64,
    pub inner_instructions: Vec<InnerInstruction>,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub pre_token_balances: Vec<TokenBalance>,
    pub post_token_balances: Vec<TokenBalance>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerInstruction {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instruction {
    pub parsed: Option<InstructionParsed>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionParsed {
    pub info: InstructionInfo,
    #[serde(rename = "type")]
    pub instruction_type: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionInfo {
    pub authority: Option<String>,
    pub mint: Option<String>,
    pub destination: Option<String>,
    pub source: Option<String>,
    pub token_amount: Option<TokenAmount>,
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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub account_keys: Vec<AccountKey>,
    //pub instructions: Vec<Instruction>,
    //pub recent_blockhash: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountKey {
    pub pubkey: String,
}

// #[derive(Debug, Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Instruction {
//     pub accounts: Vec<u64>,
//     pub data: String,
//     pub program_id_index: u64,
// }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub message: Message,
    pub signatures: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransaction {
    pub meta: Meta,
    pub transaction: Transaction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransactions {
    pub transactions: Vec<BlockTransaction>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    pub account_index: i64,
    pub mint: String,
    pub owner: String,
    pub ui_token_amount: TokenAmount,
}

impl TokenBalance {
    pub fn get_amount(&self) -> BigUint {
        self.ui_token_amount.amount.clone()
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount: BigUint,
}
