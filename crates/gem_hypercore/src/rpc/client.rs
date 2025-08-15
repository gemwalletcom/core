use crate::typeshare::balance::{HypercoreBalances, HypercoreDelegationBalance, HypercoreStakeBalance, HypercoreValidator};
use crate::typeshare::order::HypercorePerpetualFill;
use crate::typeshare::response::{HyperCoreBroadcastResult, TransactionBroadcastResponse};
use chain_traits::ChainTraits;
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

    pub async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        let json_data: serde_json::Value = serde_json::from_str(&data)?;
        let response: serde_json::Value = self.client.post("/exchange", &json_data, None).await?;
        match serde_json::from_value::<TransactionBroadcastResponse>(response)?.into_result(data) {
            HyperCoreBroadcastResult::Success(result) => Ok(result),
            HyperCoreBroadcastResult::Error(error) => Err(error.into()),
        }
    }
    pub async fn get_staking_validators(&self) -> Result<Vec<HypercoreValidator>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post("/info", &json!({"type": "validatorSummaries"}), None).await?)
    }

    pub async fn get_staking_delegations(&self, user: &str) -> Result<Vec<HypercoreDelegationBalance>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post("/info", &json!({"type": "delegations", "user": user}), None).await?)
    }

    pub async fn get_staking_apy(&self) -> Result<f64, Box<dyn Error + Send + Sync>> {
        let validators = self.get_staking_validators().await?;
        Ok(HypercoreValidator::max_apr(validators))
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
                None,
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
                None,
            )
            .await?)
    }

    pub async fn get_user_fills_by_time(&self, user: &str, start_time: i64) -> Result<Vec<HypercorePerpetualFill>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .post(
                "/info",
                &serde_json::json!({
                    "type": "userFillsByTime",
                    "user": user,
                    "startTime": start_time
                }),
                None,
            )
            .await?)
    }
}

impl<C: Client> ChainTraits for HyperCoreClient<C> {}
