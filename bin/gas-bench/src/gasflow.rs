// https://api.gasflow.dev/predict

use anyhow::Result;
use primitives::{fee::FeePriority, PriorityFeeValue};
use num_bigint::BigInt;
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

#[derive(Debug, Deserialize)]
pub struct GasflowResponse {
    pub current_block_number: u64,
    pub current_base_fee_gwei: f64,
    pub predicted_quantiles: PredictedQuantiles,
    pub network_metrics: NetworkMetrics,
}

impl GasflowResponse {
    /// Converts the raw Gasflow API data into the common `GemstoneFeeData` format.
    pub fn fee_data(&self) -> GemstoneFeeData {
        let gas_used_ratio_str = Some(format!("{:.1}%", self.network_metrics.gas_ratio_5 * 100.0));

        GemstoneFeeData {
            latest_block: self.current_block_number,
            suggest_base_fee: self.current_base_fee_gwei.to_string(),
            gas_used_ratio: gas_used_ratio_str,
            priority_fees: vec![
                PriorityFeeValue {
                    priority: FeePriority::Slow,
                    value: BigInt::from(self.predicted_quantiles.minimum as i64),
                },
                PriorityFeeValue {
                    priority: FeePriority::Normal,
                    value: BigInt::from(self.predicted_quantiles.normal as i64),
                },
                PriorityFeeValue {
                    priority: FeePriority::Fast,
                    value: BigInt::from(self.predicted_quantiles.fast as i64),
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
