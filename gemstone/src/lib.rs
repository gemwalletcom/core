use std::collections::HashMap;

pub mod config;
pub mod explorer;
pub mod sui;

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

impl GemstoneError {
    fn from(error: anyhow::Error) -> Self {
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

#[uniffi::export]
pub fn explorer_get_name_by_host(host: String) -> Option<String> {
    explorer::get_name_by_host(host)
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
