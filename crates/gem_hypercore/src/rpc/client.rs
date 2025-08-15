use crate::typeshare::balance::HypercoreValidator;
use gem_client::Client;
use serde_json::json;
use std::error::Error;

pub struct HyperCoreClient<C: Client> {
    client: C,
}

impl<C: Client> HyperCoreClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_validators(&self) -> Result<Vec<HypercoreValidator>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post("/info", &json!({"type": "validatorSummaries"})).await?)
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
}
