// https://api.etherscan.io/api?module=gastracker&action=gasoracle&apikey=YourApiKeyToken

use crate::client::GemstoneFeeData;
use anyhow::{anyhow, Result};
use gemstone::ethereum::model::{GemFeePriority, GemPriorityFeeRecord};
use serde::Deserialize;
use serde_serializers::deserialize_u64_from_str;

const ETHERSCAN_API_URL: &str = "https://api.etherscan.io/api";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EtherscanResult {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub last_block: u64,
    pub safe_gas_price: String,
    pub propose_gas_price: String,
    pub fast_gas_price: String,
    #[serde(rename = "suggestBaseFee")]
    pub suggest_base_fee: String,
    #[serde(rename = "gasUsedRatio")]
    pub gas_used_ratio: String,
}

impl EtherscanResult {
    /// Converts the raw Etherscan gas oracle data into the common `GemstoneFeeData` format.
    pub fn fee_data(&self) -> GemstoneFeeData {
        let base_fee: f64 = self.suggest_base_fee.parse().unwrap();
        let safe_fee: f64 = self.safe_gas_price.parse().unwrap();
        let propose_fee: f64 = self.propose_gas_price.parse().unwrap();
        let fast_fee: f64 = self.fast_gas_price.parse().unwrap();

        let gas_used_ratio_str = self
            .gas_used_ratio
            .split(',')
            .next()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .map(|val| format!("{:.1}%", val * 100.0));

        GemstoneFeeData {
            latest_block: self.last_block,
            suggest_base_fee: self.suggest_base_fee.clone(),
            gas_used_ratio: gas_used_ratio_str,
            priority_fees: vec![
                GemPriorityFeeRecord {
                    priority: GemFeePriority::Slow,
                    value: (safe_fee - base_fee).to_string(),
                },
                GemPriorityFeeRecord {
                    priority: GemFeePriority::Normal,
                    value: (propose_fee - base_fee).to_string(),
                },
                GemPriorityFeeRecord {
                    priority: GemFeePriority::Fast,
                    value: (fast_fee - base_fee).to_string(),
                },
            ],
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EtherscanResponse {
    pub status: String,
    pub message: String,
    pub result: EtherscanResult,
}

pub struct EtherscanClient {
    client: reqwest::Client,
    api_key: String,
}

impl EtherscanClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    pub async fn fetch_gas_oracle(&self) -> Result<EtherscanResponse> {
        let url = format!("{}?module=gastracker&action=gasoracle&apikey={}", ETHERSCAN_API_URL, self.api_key);
        let response = self.client.get(&url).send().await?.json::<EtherscanResponse>().await?;
        if response.status != "1" {
            return Err(anyhow!("Etherscan API error: {}", response.message));
        }
        Ok(response)
    }
}
