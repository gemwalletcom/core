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
pub fn sui_encode_split_stake(
    input: &sui::SuiStakeInput,
) -> Result<sui::SuiStakeOutput, GemstoneError> {
    sui::encode_split_and_stake(input).map_err(|op| GemstoneError::AnyError {
        msg: op.to_string(),
    })
}

#[uniffi::export]
pub fn get_validators() -> HashMap<String, Vec<String>> {
    config::get_validators()
}
