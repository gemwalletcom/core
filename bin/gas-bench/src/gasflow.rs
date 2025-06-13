// https://api.gasflow.dev/predict

use anyhow::Result;
use gemstone::ethereum::model::{GemFeePriority, GemPriorityFeeRecord};
use serde::Deserialize;

use crate::client::GemstoneFeeData;

const GASFLOW_API_URL: &str = "https://api.gasflow.dev/predict";

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct PredictedQuantiles {
    pub minimum: f64,
    pub normal: f64,
    pub fast: f64,
    pub urgent: f64,
    pub critical: f64,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct NetworkMetrics {
    pub gas_ratio_5: f64,
    pub gas_spikes_25: f64,
    pub fee_ewma_10: f64,
    pub fee_ewma_25: f64,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct GasflowResponse {
    pub current_block_number: u64,
    pub current_base_fee_gwei: f64,
    pub timestamp: u64,
    pub predicted_quantiles: PredictedQuantiles,
    pub network_metrics: NetworkMetrics,
}

impl GasflowResponse {
    pub fn fee_data(&self) -> GemstoneFeeData {
        GemstoneFeeData {
            latest_block: self.current_block_number,
            suggest_base_fee: self.current_base_fee_gwei.to_string(),
            priority_fees: vec![
                GemPriorityFeeRecord {
                    priority: GemFeePriority::Slow,
                    value: self.predicted_quantiles.minimum.to_string(),
                },
                GemPriorityFeeRecord {
                    priority: GemFeePriority::Normal,
                    value: self.predicted_quantiles.normal.to_string(),
                },
                GemPriorityFeeRecord {
                    priority: GemFeePriority::Fast,
                    value: self.predicted_quantiles.fast.to_string(),
                },
            ],
        }
    }
}

pub struct GasflowClient {
    client: reqwest::Client,
}

impl GasflowClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_prediction(&self) -> Result<GasflowResponse> {
        let response = self.client.get(GASFLOW_API_URL).send().await?.json::<GasflowResponse>().await?;
        Ok(response)
    }
}
