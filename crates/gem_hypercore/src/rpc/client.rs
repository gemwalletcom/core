use crate::typeshare::balance::{HypercoreBalances, HypercoreStakeBalance, HypercoreValidator};
use gem_client::Client;
use primitives::Chain;
use serde_json::json;
use std::error::Error;

#[derive(Debug)]
pub struct HyperCoreClient<C: Client> {
    client: C,
    pub chain: Chain,
}

impl<C: Client> HyperCoreClient<C> {
    pub fn new(client: C) -> Self {
        Self {
            client,
            chain: Chain::HyperCore,
        }
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

    pub async fn get_spot_balances(&self, user: &str) -> Result<HypercoreBalances, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .post(
                "/info",
                &serde_json::json!({
                    "type": "spotClearinghouseState",
                    "user": user
                }),
            )
            .await?)
    }

    pub async fn get_stake_balance(&self, user: &str) -> Result<HypercoreStakeBalance, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .post(
                "/info",
                &serde_json::json!({
                    "type": "delegatorSummary",
                    "user": user
                }),
            )
            .await?)
    }
}
