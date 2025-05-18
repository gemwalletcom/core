use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

pub use num_bigint::BigUint;

pub const ENCODING_BASE64: &str = "base64";
pub const ENCODING_BASE58: &str = "base58";

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

impl Meta {
    pub fn get_pre_token_balance(&self, account_index: i64) -> Option<TokenBalance> {
        self.pre_token_balances.iter().find(|b| b.account_index == account_index).cloned()
    }

    pub fn get_post_token_balance(&self, account_index: i64) -> Option<TokenBalance> {
        self.post_token_balances.iter().find(|b| b.account_index == account_index).cloned()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerInstruction {
    pub instructions: Vec<Parsed<Option<InstructionParsed>>>,
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub ok: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub account_keys: Vec<AccountKey>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountKey {
    pub pubkey: String,
}

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
pub struct TokenAccountInfo {
    pub pubkey: String,
    pub account: TokenAccountData,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenAccountData {
    pub data: Parsed<Info<TokenAccountInfoData>>,
    pub owner: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenAccountInfoData {
    pub mint: String,
    pub token_amount: TokenAmount,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount: BigUint,
}

// Models moved from gem_solana/src/jsonrpc.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub commitment: &'static str,
    pub encoding: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<Filter>,
}

impl Configuration {
    pub fn new(filters: Vec<Filter>) -> Self {
        Self {
            commitment: "confirmed",
            encoding: "base64",
            filters,
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            commitment: "confirmed",
            encoding: "base64",
            filters: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub memcmp: Memcmp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memcmp {
    pub offset: u8,
    pub bytes: String,
    pub encoding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueResult<T> {
    pub value: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueData<T> {
    pub data: T,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parsed<T> {
    pub parsed: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info<T> {
    pub info: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub decimals: i32,
    pub is_initialized: bool,
    pub mint_authority: String,
    pub supply: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockhash {
    pub blockhash: String,
}

pub type AccountData = ValueData<Vec<String>>;

pub type ResultTokenInfo = ValueResult<ValueData<Parsed<Info<TokenInfo>>>>;
pub type LatestBlockhash = ValueResult<Blockhash>;
