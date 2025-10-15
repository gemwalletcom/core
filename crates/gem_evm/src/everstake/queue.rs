use crate::everstake::constants::{EVERSTAKE_API_BASE_URL, EVERSTAKE_VALIDATORS_QUEUE_PATH};
use gem_client::{Client, ReqwestClient};
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct EverstakeValidatorQueueResponse {
    pub validator_activation_time: u64,
    pub validator_exit_time: u64,
    pub validator_withdraw_time: u64,
    pub validator_adding_delay: u64,
}

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub async fn get_everstake_validator_queue() -> Result<EverstakeValidatorQueueResponse, Box<dyn Error + Send + Sync>> {
    let client = ReqwestClient::new(EVERSTAKE_API_BASE_URL.to_string(), reqwest::Client::new());
    let response: EverstakeValidatorQueueResponse = client.get(EVERSTAKE_VALIDATORS_QUEUE_PATH).await?;
    Ok(response)
}
