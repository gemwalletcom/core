use network::AlienError;
use payment::PaymentWrapper;
use primitives::Chain;
pub mod lido;
use gem_bsc::stake_hub;
use std::str::FromStr;
pub mod asset;
pub mod bsc;
pub mod config;
pub mod solana;
use solana::MplMetadata;
pub mod block_explorer;
pub mod chain;
pub mod network;
pub mod payment;
pub mod sui;
pub mod swapper;
pub mod ton;
pub mod wallet_connect;

uniffi::setup_scaffolding!("gemstone");
static LIB_VERSION: &str = "0.2.1";

#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
}

#[uniffi::export]
pub fn lib_version() -> String {
    LIB_VERSION.into()
}

/// GemstoneError
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum GemstoneError {
    #[error("{msg}")]
    AnyError { msg: String },
}

impl From<anyhow::Error> for GemstoneError {
    fn from(error: anyhow::Error) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<&str> for GemstoneError {
    fn from(error: &str) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<Box<dyn std::error::Error>> for GemstoneError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<AlienError> for GemstoneError {
    fn from(error: AlienError) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

/// WalletConnect
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

/// Asset
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

/// Cosmos
#[uniffi::export]
pub fn cosmos_convert_hrp(address: String, hrp: String) -> Result<String, GemstoneError> {
    gem_cosmos::converter::convert_cosmos_address(&address, &hrp).map_err(GemstoneError::from)
}

/// Solana
#[uniffi::export]
pub fn solana_decode_metadata(base64_str: String) -> Result<MplMetadata, GemstoneError> {
    solana::decode_mpl_metadata(base64_str).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn solana_derive_metadata_pda(mint: String) -> Result<String, GemstoneError> {
    solana::derive_metadata_pda(&mint).map_err(GemstoneError::from)
}

/// Ton
#[uniffi::export]
pub fn ton_encode_get_wallet_address(address: String) -> Result<String, GemstoneError> {
    ton::jetton::encode_get_wallet_address_slice(&address).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn ton_decode_jetton_address(base64_data: String, len: u64) -> Result<String, GemstoneError> {
    ton::jetton::decode_data_to_address(&base64_data, len).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn ton_hex_to_base64_address(hex_str: String) -> Result<String, GemstoneError> {
    ton::address::hex_to_base64_address(hex_str).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn ton_base64_to_hex_address(base64_str: String) -> Result<String, GemstoneError> {
    ton::address::base64_to_hex_address(base64_str).map_err(GemstoneError::from)
}

/// Bsc
#[uniffi::export]
pub fn bsc_encode_validators_call(offset: u16, limit: u16) -> Vec<u8> {
    stake_hub::encode_validators_call(offset, limit)
}

#[uniffi::export]
pub fn bsc_decode_validators_return(result: Vec<u8>) -> Result<Vec<bsc::BscValidator>, GemstoneError> {
    bsc::decode_validators_return(&result).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_delegations_call(delegator: &str, offset: u16, limit: u16) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_delegations_call(delegator, offset, limit).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_decode_delegations_return(result: Vec<u8>) -> Result<Vec<bsc::BscDelegation>, GemstoneError> {
    bsc::decode_delegations_return(&result).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_undelegations_call(delegator: &str, offset: u16, limit: u16) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_undelegations_call(delegator, offset, limit).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_decode_undelegations_return(result: Vec<u8>) -> Result<Vec<bsc::BscDelegation>, GemstoneError> {
    bsc::decode_undelegations_return(&result).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_delegate_call(operator_address: String, delegate_vote_power: bool) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_delegate_call(&operator_address, delegate_vote_power).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_undelegate_call(operator_address: String, shares: String) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_undelegate_call(&operator_address, &shares).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_redelegate_call(src_validator: String, dst_validator: String, shares: String, delegate_vote_power: bool) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_redelegate_call(&src_validator, &dst_validator, &shares, delegate_vote_power).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_claim_call(operator_address: String, request_number: u64) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_claim_call(&operator_address, request_number).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn payment_decode_url(string: &str) -> Result<PaymentWrapper, GemstoneError> {
    payment::decode_url(string).map_err(GemstoneError::from)
}
