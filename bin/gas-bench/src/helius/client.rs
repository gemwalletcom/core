use super::{HeliusPriorityFeeOptions, HeliusPriorityFeeParams, HeliusPriorityFeeRequest, HeliusPriorityFeeResponse, HeliusPriorityFees};
use std::error::Error;

pub struct HeliusClient {
    client: reqwest::Client,
    endpoint: String,
}

impl HeliusClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: format!("https://mainnet.helius-rpc.com/?api-key={}", api_key),
        }
    }

    pub async fn fetch_priority_fee_estimate(&self, account_keys: Option<Vec<String>>) -> Result<HeliusPriorityFees, Box<dyn Error + Send + Sync>> {
        let request = HeliusPriorityFeeRequest {
            jsonrpc: "2.0",
            id: "1",
            method: "getPriorityFeeEstimate",
            params: vec![HeliusPriorityFeeParams {
                account_keys,
                options: HeliusPriorityFeeOptions {
                    include_all_priority_fee_levels: true,
                    lookback_slots: 150,
                },
            }],
        };

        let response = self.client.post(&self.endpoint).json(&request).send().await?;

        let result: HeliusPriorityFeeResponse = response.json().await?;

        let levels = result.result.priority_fee_levels.ok_or("No priority fee levels in response")?;

        Ok(HeliusPriorityFees::from_levels(&levels))
    }
}
