use gem_client::{Client, ReqwestClient};
use serde::Deserialize;
use serde_serializers::deserialize_f64_from_str;
use std::error::Error;

const EVERSTAKE_STATS_BASE_URL: &str = "https://eth-api-b2c.everstake.one";
const EVERSTAKE_STATS_PATH: &str = "/api/v1/stats";

#[derive(Deserialize)]
struct EverstakeStatsResponse {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    apr: f64,
}

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub async fn get_everstake_staking_apy() -> Result<Option<f64>, Box<dyn Error + Send + Sync>> {
    let client = ReqwestClient::new(EVERSTAKE_STATS_BASE_URL.to_string(), reqwest::Client::new());
    let stats: EverstakeStatsResponse = client.get(EVERSTAKE_STATS_PATH).await?;

    Ok(Some(stats.apr * 100.0))
}
