use primitives::Chain;
use std::{collections::HashMap, str::FromStr};

pub mod asset;
pub mod config;
pub mod explorer;
pub mod sui;
pub mod wallet_connect;

uniffi::include_scaffolding!("gemstone");
static LIB_VERSION: &str = "0.1.1";

#[uniffi::export]
pub fn lib_version() -> String {
    LIB_VERSION.into()
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum GemstoneError {
    #[error("{msg}")]
    AnyError { msg: String },
}

impl From<anyhow::Error> for GemstoneError {
    fn from(error: anyhow::Error) -> Self {
        Self::AnyError {
            msg: error.to_string(),
        }
    }
}

impl From<strum::ParseError> for GemstoneError {
    fn from(error: strum::ParseError) -> Self {
        Self::AnyError {
            msg: error.to_string(),
        }
    }
}

impl From<&str> for GemstoneError {
    fn from(error: &str) -> Self {
        Self::AnyError {
            msg: error.to_string(),
        }
    }
}

#[uniffi::export]
pub async fn say_after(ms: u64, who: String) -> String {
    use async_std::future::{pending, timeout};
    let never = pending::<()>();
    timeout(std::time::Duration::from_millis(ms), never)
        .await
        .unwrap_err();
    format!("Hello, {who}!")
}

#[derive(uniffi::Object)]
struct Explorer {}
#[uniffi::export]
impl Explorer {
    #[uniffi::constructor]
    fn new() -> Self {
        Self {}
    }

    pub fn get_name_by_host(&self, host: String) -> Option<String> {
        explorer::get_name_by_host(host)
    }

    pub fn get_transaction_url(&self, chain: String, transaction_id: String) -> String {
        let chain = Chain::from_str(&chain).unwrap();
        explorer::get_explorer_transaction_url(chain, &transaction_id)
    }

    pub fn get_address_url(&self, chain: String, address: String) -> String {
        let chain = Chain::from_str(&chain).unwrap();
        explorer::get_explorer_address_url(chain, &address)
    }

    pub fn get_token_url(&self, chain: String, address: String) -> Option<String> {
        let chain = Chain::from_str(&chain).unwrap();
        explorer::get_explorer_token_url(chain, &address)
    }
}

#[uniffi::export]
pub fn sui_encode_transfer(
    input: &sui::model::SuiTransferInput,
) -> Result<sui::model::SuiTxOutput, GemstoneError> {
    sui::encode_transfer(input).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn sui_encode_token_transfer(
    input: &sui::model::SuiTokenTransferInput,
) -> Result<sui::model::SuiTxOutput, GemstoneError> {
    sui::encode_token_transfer(input).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn sui_encode_split_stake(
    input: &sui::model::SuiStakeInput,
) -> Result<sui::model::SuiTxOutput, GemstoneError> {
    sui::encode_split_and_stake(input).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn sui_encode_unstake(
    input: &sui::model::SuiUnstakeInput,
) -> Result<sui::model::SuiTxOutput, GemstoneError> {
    sui::encode_unstake(input).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn sui_validate_and_hash(encoded: String) -> Result<sui::model::SuiTxOutput, GemstoneError> {
    sui::validate_and_hash(&encoded).map_err(GemstoneError::from)
}

#[derive(uniffi::Object)]
struct Config {}
#[uniffi::export]
impl Config {
    // Constructors need to be annotated as such.
    // The return value can be either `Self` or `Arc<Self>`
    // It is treated as the primary constructor, so in most languages this is invoked with
    #[uniffi::constructor]
    fn new() -> Self {
        Self {}
    }

    fn get_validators(&self) -> HashMap<String, Vec<String>> {
        config::get_validators()
    }
}

#[derive(uniffi::Object)]
struct WalletConnectNamespace {}
#[uniffi::export]
impl WalletConnectNamespace {
    #[uniffi::constructor]
    fn new() -> Self {
        Self {}
    }

    fn get_namespace(&self, chain: String) -> Option<String> {
        let chain = Chain::from_str(&chain).ok()?;
        wallet_connect::get_namespace(chain)
    }

    fn get_reference(&self, chain: String) -> Option<String> {
        let chain = Chain::from_str(&chain).ok()?;
        wallet_connect::get_reference(chain)
    }
}

#[uniffi::export]
pub fn asset_default_rank(chain: String) -> i32 {
    match Chain::from_str(&chain) {
        Ok(chain) => asset::get_default_rank(chain),
        Err(_) => 10,
    }
}

#[uniffi::export]
pub fn asset_wrapper(chain: String) -> asset::AssetWrapper {
    let chain =
        Chain::from_str(&chain).unwrap_or_else(|_| panic!("unknown primitives::Chain {}", chain));
    asset::get_asset(chain)
}

#[uniffi::export]
pub fn cosmos_convert_hrp(address: String, hrp: String) -> Result<String, GemstoneError> {
    gem_cosmos::converter::convert_cosmos_address(address.as_str(), hrp.as_str())
        .map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn cosmos_convert_hrp_by_chain(
    address: String,
    chain: String,
) -> Result<String, GemstoneError> {
    let chain = Chain::from_str(&chain)?;
    let hrp = chain.hrp();
    if hrp.is_empty() {
        return Err("hrp not found".into());
    }
    gem_cosmos::converter::convert_cosmos_address(address.as_str(), hrp)
        .map_err(GemstoneError::from)
}
