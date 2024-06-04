use config::{docs::DocsUrl, public::PublicUrl, social::SocialUrl};
use gem_bsc::stake_hub;
use payment::PaymentWrapper;
use primitives::Chain;
use std::{collections::HashMap, str::FromStr};
pub mod asset;
pub mod bsc;
pub mod config;
pub mod explorer;
pub mod solana;
use solana::MplMetadata;
pub mod chain;
pub mod payment;
pub mod sui;
pub mod ton;
pub mod wallet_connect;

uniffi::include_scaffolding!("gemstone");
static LIB_VERSION: &str = "0.2.1";

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

/// Explorer
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

/// Sui
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

/// Config
#[derive(uniffi::Object)]
struct Config {}
#[uniffi::export]
impl Config {
    #[uniffi::constructor]
    fn new() -> Self {
        Self {}
    }

    fn get_validators(&self) -> HashMap<String, Vec<String>> {
        config::get_validators()
    }

    fn get_stake_locktime(&self, chain: String) -> u64 {
        config::get_stake_lock_time(&chain)
    }

    fn get_min_stake_amount(&self, chain: String) -> u64 {
        config::get_min_stake_amount(&chain)
    }

    fn get_docs_url(&self, item: DocsUrl) -> String {
        config::get_docs_url(item)
    }

    fn get_social_url(&self, item: SocialUrl) -> String {
        config::get_social_url(item).to_string()
    }

    fn get_public_url(&self, item: PublicUrl) -> String {
        config::get_public_url(item).to_string()
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

/// Chain
#[uniffi::export]
pub fn chain_transaction_timeout_seconds(chain: String) -> f64 {
    let chain = Chain::from_str(&chain).unwrap();
    chain::chain_transaction_timeout_seconds(chain)
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
pub fn bsc_decode_validators_return(
    result: Vec<u8>,
) -> Result<Vec<bsc::BscValidator>, GemstoneError> {
    bsc::decode_validators_return(&result).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_delegations_call(
    delegator: &str,
    offset: u16,
    limit: u16,
) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_delegations_call(delegator, offset, limit).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_decode_delegations_return(
    result: Vec<u8>,
) -> Result<Vec<bsc::BscDelegation>, GemstoneError> {
    bsc::decode_delegations_return(&result).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_undelegations_call(
    delegator: &str,
    offset: u16,
    limit: u16,
) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_undelegations_call(delegator, offset, limit).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_decode_undelegations_return(
    result: Vec<u8>,
) -> Result<Vec<bsc::BscDelegation>, GemstoneError> {
    bsc::decode_undelegations_return(&result).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_delegate_call(
    operator_address: String,
    delegate_vote_power: bool,
) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_delegate_call(&operator_address, delegate_vote_power)
        .map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_undelegate_call(
    operator_address: String,
    shares: String,
) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_undelegate_call(&operator_address, &shares).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_redelegate_call(
    src_validator: String,
    dst_validator: String,
    shares: String,
    delegate_vote_power: bool,
) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_redelegate_call(&src_validator, &dst_validator, &shares, delegate_vote_power)
        .map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn bsc_encode_claim_call(
    operator_address: String,
    request_number: u64,
) -> Result<Vec<u8>, GemstoneError> {
    stake_hub::encode_claim_call(&operator_address, request_number).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn payment_decode_url(string: &str) -> Result<PaymentWrapper, GemstoneError> {
    payment::decode_url(string).map_err(GemstoneError::from)
}
