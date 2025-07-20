use num_bigint::BigInt;
use primitives::{AssetId, Chain};
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;
use std::collections::{HashMap, HashSet};

pub use num_bigint::BigUint;

pub const ENCODING_BASE64: &str = "base64";
pub const ENCODING_BASE58: &str = "base58";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteAccounts {
    pub current: Vec<VoteAccount>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteAccount {
    pub vote_pubkey: String,
    pub node_pubkey: String,
    pub commission: u8,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub blockhash: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub fee: u64,
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

    pub fn get_pre_token_balance_by_owner(&self, owner: &str) -> Vec<TokenBalance> {
        self.pre_token_balances.iter().filter(|b| b.owner == owner).cloned().collect()
    }

    pub fn get_post_token_balance_by_owner(&self, owner: &str) -> Vec<TokenBalance> {
        self.post_token_balances.iter().filter(|b| b.owner == owner).cloned().collect()
    }

    pub fn get_token_balance_changes_by_owner(&self, owner: &str) -> Vec<TokenBalanceChange> {
        let pre_balances: HashMap<_, _> = self
            .pre_token_balances
            .iter()
            .filter(|b| b.owner == owner)
            .map(|b| (b.mint.clone(), b.get_amount()))
            .collect();

        let post_balances: HashMap<_, _> = self
            .post_token_balances
            .iter()
            .filter(|b| b.owner == owner)
            .map(|b| (b.mint.clone(), b.get_amount()))
            .collect();
        let all_mints: HashSet<_> = pre_balances.keys().chain(post_balances.keys()).cloned().collect();

        all_mints
            .into_iter()
            .filter_map(|mint| {
                let asset_id = AssetId::from_token(Chain::Solana, &mint);
                let pre_amount = pre_balances.get(&mint).cloned().unwrap_or_else(|| BigUint::from(0u64));
                let post_amount = post_balances.get(&mint).cloned().unwrap_or_else(|| BigUint::from(0u64));

                if post_amount > pre_amount {
                    let diff = &post_amount - &pre_amount;
                    Some(TokenBalanceChange {
                        asset_id,
                        amount: BigInt::from_biguint(num_bigint::Sign::Plus, diff),
                    })
                } else if pre_amount > post_amount {
                    let diff = &pre_amount - &post_amount;
                    Some(TokenBalanceChange {
                        asset_id,
                        amount: BigInt::from_biguint(num_bigint::Sign::Minus, diff),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub ok: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountKey {
    pub pubkey: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub message: TransactionMessage,
    pub signatures: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    pub block_time: i64,
    pub signature: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionMessage {
    pub account_keys: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransaction {
    pub meta: Meta,
    pub transaction: Transaction,
}

impl BlockTransaction {
    pub fn fee(&self) -> BigUint {
        BigUint::from(self.meta.fee)
    }

    pub fn get_balance_changes_by_owner(&self, owner: &str) -> TokenBalanceChange {
        // Find all account indices that belong to the owner
        let account_indices: Vec<usize> = self
            .transaction
            .message
            .account_keys
            .iter()
            .enumerate()
            .filter_map(|(i, k)| if k == owner { Some(i) } else { None })
            .collect();

        let (total_pre, total_post) = account_indices.into_iter().fold((0u64, 0u64), |(pre_acc, post_acc), idx| {
            let pre = *self.meta.pre_balances.get(idx).unwrap_or(&0);
            let post = *self.meta.post_balances.get(idx).unwrap_or(&0);
            (pre_acc.wrapping_add(pre), post_acc.wrapping_add(post))
        });

        let (sign, diff) = if total_post > total_pre {
            let diff = total_post - total_pre;
            (num_bigint::Sign::Plus, BigUint::from(diff))
        } else {
            let diff = total_pre - total_post;
            (num_bigint::Sign::Minus, BigUint::from(diff))
        };
        let fee = self.fee();
        let data = if fee > diff { BigUint::from(0u64) } else { diff - fee };

        TokenBalanceChange {
            asset_id: Chain::Solana.as_asset_id(),
            amount: BigInt::from_biguint(sign, data),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransactions {
    pub block_time: i64,
    pub transactions: Vec<BlockTransaction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleTransaction {
    pub block_time: i64,
    pub meta: Meta,
    pub transaction: Transaction,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    pub account_index: i64,
    pub mint: String,
    pub owner: String,
    pub ui_token_amount: TokenAmount,
}

#[derive(Debug, Clone)]
pub struct TokenBalanceChange {
    pub asset_id: AssetId,
    pub amount: BigInt,
}

impl TokenBalance {
    pub fn new(account_index: i64, mint: String, owner: String, ui_token_amount: TokenAmount) -> Self {
        Self {
            account_index,
            mint,
            owner,
            ui_token_amount,
        }
    }

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

impl Default for TokenAmount {
    fn default() -> Self {
        Self { amount: BigUint::from(0u64) }
    }
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
    pub supply: String,
    pub extensions: Option<Vec<Extension>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionBase<T> {
    #[serde(rename = "extension")]
    pub extension_type: String,
    pub state: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Extension {
    TokenMetadata(ExtensionBase<TokenMetadata>),
    Other(ExtensionBase<serde_json::Value>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockhash {
    pub blockhash: String,
}

pub type AccountData = ValueData<Vec<String>>;

pub type LatestBlockhash = ValueResult<Blockhash>;

pub type ResultTokenInfo = ValueResult<ValueData<Parsed<Info<TokenInfo>>>>;
impl ResultTokenInfo {
    pub fn info(&self) -> TokenInfo {
        self.value.data.parsed.info.clone()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorConfig {
    pub pubkey: String,
    pub account: ValidatorConfigAccount,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorConfigAccount {
    pub data: Parsed<Info<ValidatorConfigInfo>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorConfigInfo {
    pub name: String,
    pub config_data: Option<serde_json::Value>,
}
