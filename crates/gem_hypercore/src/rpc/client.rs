use crate::models::{
    balance::{HypercoreBalances, HypercoreDelegationBalance, HypercoreStakeBalance, HypercoreValidator},
    candlestick::HypercoreCandlestick,
    metadata::HypercoreMetadataResponse,
    order::HypercorePerpetualFill,
    position::HypercoreAssetPositions,
    referral::HypercoreReferral,
    user::{HypercoreAgentSession, HypercoreUserFee, HypercoreUserRole},
};
use chain_traits::ChainTraits;
use gem_client::Client;
use std::sync::Arc;

use crate::config::HypercoreConfig;
use primitives::{Chain, Preferences};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;

#[derive(Debug)]
pub struct InMemoryPreferences {
    data: Mutex<HashMap<String, String>>,
}

impl Default for InMemoryPreferences {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryPreferences {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }
}

impl Preferences for InMemoryPreferences {
    fn get(&self, key: String) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        Ok(self.data.lock().unwrap().get(&key).cloned())
    }

    fn set(&self, key: String, value: String) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.data.lock().unwrap().insert(key, value);
        Ok(())
    }

    fn remove(&self, key: String) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.data.lock().unwrap().remove(&key);
        Ok(())
    }
}

pub struct HyperCoreClient<C: Client> {
    client: C,
    pub chain: Chain,
    pub config: HypercoreConfig,
    pub preferences: Arc<dyn Preferences>,
    pub secure_preferences: Arc<dyn Preferences>,
}

impl<C: Client> std::fmt::Debug for HyperCoreClient<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HyperCoreClient")
            .field("chain", &self.chain)
            .field("config", &self.config)
            .field("preferences", &"<Preferences>")
            .field("secure_preferences", &"<Preferences>")
            .finish()
    }
}

impl<C: Client> HyperCoreClient<C> {
    pub fn new(client: C) -> Self {
        let preferences = Arc::new(InMemoryPreferences::new());
        let secure_preferences = Arc::new(InMemoryPreferences::new());
        Self {
            client,
            chain: Chain::HyperCore,
            config: HypercoreConfig::default(),
            preferences,
            secure_preferences,
        }
    }

    pub fn new_with_preferences(client: C, preferences: Arc<dyn Preferences>, secure_preferences: Arc<dyn Preferences>) -> Self {
        Self {
            client,
            chain: Chain::HyperCore,
            config: HypercoreConfig::default(),
            preferences,
            secure_preferences,
        }
    }

    async fn info<T>(&self, payload: serde_json::Value) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned,
    {
        Ok(self.client.post("/info", &payload, None).await?)
    }

    pub async fn exchange(&self, payload: serde_json::Value) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post("/exchange", &payload, None).await?)
    }

    pub async fn get_validators(&self) -> Result<Vec<HypercoreValidator>, Box<dyn Error + Send + Sync>> {
        self.info(json!({"type": "validatorSummaries"})).await
    }

    pub async fn get_staking_delegations(&self, user: &str) -> Result<Vec<HypercoreDelegationBalance>, Box<dyn Error + Send + Sync>> {
        self.info(json!({"type": "delegations", "user": user})).await
    }

    pub async fn get_staking_apy(&self) -> Result<f64, Box<dyn Error + Send + Sync>> {
        let validators = self.get_validators().await?;
        Ok(HypercoreValidator::max_apr(validators))
    }

    pub async fn get_spot_balances(&self, user: &str) -> Result<HypercoreBalances, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "spotClearinghouseState",
            "user": user
        }))
        .await
    }

    pub async fn get_stake_balance(&self, user: &str) -> Result<HypercoreStakeBalance, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "delegatorSummary",
            "user": user
        }))
        .await
    }

    pub async fn get_user_fills_by_time(&self, user: &str, start_time: i64) -> Result<Vec<HypercorePerpetualFill>, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "userFillsByTime",
            "user": user,
            "startTime": start_time
        }))
        .await
    }

    pub async fn get_clearinghouse_state(&self, user: &str) -> Result<HypercoreAssetPositions, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "clearinghouseState",
            "user": user
        }))
        .await
    }

    pub async fn get_metadata(&self) -> Result<HypercoreMetadataResponse, Box<dyn Error + Send + Sync>> {
        self.info(json!({"type": "metaAndAssetCtxs"})).await
    }

    pub async fn get_candlesticks(
        &self,
        coin: &str,
        interval: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<HypercoreCandlestick>, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "candleSnapshot",
            "req": {
                "coin": coin,
                "interval": interval,
                "startTime": start_time,
                "endTime": end_time
            }
        }))
        .await
    }

    pub async fn get_user_role(&self, user: &str) -> Result<HypercoreUserRole, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "userRole",
            "user": user
        }))
        .await
    }

    pub async fn get_referral(&self, user: &str) -> Result<HypercoreReferral, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "referral",
            "user": user
        }))
        .await
    }

    pub async fn get_extra_agents(&self, user: &str) -> Result<Vec<HypercoreAgentSession>, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "extraAgents",
            "user": user
        }))
        .await
    }

    pub async fn get_builder_fee(&self, user: &str, builder: &str) -> Result<u32, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "maxBuilderFee",
            "user": user,
            "builder": builder
        }))
        .await
    }

    pub async fn get_user_fees(&self, user: &str) -> Result<HypercoreUserFee, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "userFees",
            "user": user
        }))
        .await
    }
}

impl<C: Client> ChainTraits for HyperCoreClient<C> {}

impl<C: Client> chain_traits::ChainProvider for HyperCoreClient<C> {
    fn get_chain(&self) -> primitives::Chain {
        Chain::HyperCore
    }
}
