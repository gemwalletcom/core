use crate::typeshare::balance::{HypercoreBalances, HypercoreValidator};
use std::error::Error;

pub struct HyperCoreClient {
    url: String,
    client: reqwest::Client,
}

impl HyperCoreClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_validators(&self) -> Result<Vec<HypercoreValidator>, Box<dyn Error + Send + Sync>> {
        let response = self
            .client
            .post(format!("{}/info", self.url))
            .json(&serde_json::json!({"type": "validatorSummaries"}))
            .send()
            .await?
            .json::<Vec<HypercoreValidator>>()
            .await?;
        Ok(response)
    }

    pub async fn get_staking_apy(&self) -> Result<f64, Box<dyn Error + Send + Sync>> {
        Ok(self
            .get_validators()
            .await?
            .into_iter()
            .filter(|x| x.is_active)
            .flat_map(|x| x.stats.into_iter().map(|(_, stat)| stat.predicted_apr))
            .fold(0.0, f64::max)
            * 100.0)
    }

    pub async fn spot_balances(&self, user: &str) -> Result<HypercoreBalances, Box<dyn Error + Send + Sync>> {
        let response = self
            .client
            .post(format!("{}/info", self.url))
            .json(&serde_json::json!({
                "type": "spotClearinghouseState",
                "user": user
            }))
            .send()
            .await?
            .json::<HypercoreBalances>()
            .await?;
        Ok(response)
    }
}
