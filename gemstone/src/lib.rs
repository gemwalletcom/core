use primitives::Chain;
use std::{collections::HashMap, str::FromStr};

pub mod asset;
pub mod config;
pub mod explorer;
pub mod solana;
use solana::MplMetadata;
pub mod sui;
pub mod ton;
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

impl From<&str> for GemstoneError {
    fn from(error: &str) -> Self {
        Self::AnyError {
            msg: error.to_string(),
        }
    }
}

impl From<Box<dyn std::error::Error>> for GemstoneError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::AnyError {
            msg: error.to_string(),
        }
    }
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
    let chain = Chain::from_str(&chain).unwrap();
    asset::get_asset(chain)
}

#[uniffi::export]
pub fn cosmos_convert_hrp(address: String, hrp: String) -> Result<String, GemstoneError> {
    gem_cosmos::converter::convert_cosmos_address(&address, &hrp).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn solana_decode_metadata(base64_str: String) -> Result<MplMetadata, GemstoneError> {
    solana::decode_mpl_metadata(base64_str).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn solana_derive_metadata_pda(mint: String) -> Result<String, GemstoneError> {
    solana::derive_metadata_pda(&mint).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn ton_encode_get_wallet_address(address: String) -> Result<String, GemstoneError> {
    ton::jetton::encode_wallet_address_data(&address).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn ton_decode_jetton_address(base64_str: String) -> Result<String, GemstoneError> {
    ton::jetton::decode_address_data(&base64_str).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn ton_hex_to_base64_address(_hex_str: String) -> Result<String, GemstoneError> {
    todo!()
}

#[uniffi::export]
pub fn ton_base64_to_hex_address(_base64_str: String) -> Result<String, GemstoneError> {
    todo!()
}
