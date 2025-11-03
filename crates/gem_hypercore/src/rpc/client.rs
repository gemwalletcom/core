use crate::models::{
    balance::{Balances, DelegationBalance, StakeBalance, Validator},
    candlestick::Candlestick,
    metadata::HypercoreMetadataResponse,
    order::{OpenOrder, PerpetualFill},
    position::AssetPositions,
    referral::Referral,
    spot::{OrderbookResponse, SpotMeta, SpotMetaRaw},
    user::{AgentSession, LedgerUpdate, UserFee, UserRole},
};
use chain_traits::ChainTraits;
use gem_client::Client;
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
};

use crate::config::HypercoreConfig;
use gem_jsonrpc::X_CACHE_TTL;
use primitives::{Chain, Preferences};
use serde_json::json;

const SPOT_META_CACHE_TTL_SECS: u64 = 3600;

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

    pub async fn get_validators(&self) -> Result<Vec<Validator>, Box<dyn Error + Send + Sync>> {
        self.info(json!({"type": "validatorSummaries"})).await
    }

    pub async fn get_staking_delegations(&self, user: &str) -> Result<Vec<DelegationBalance>, Box<dyn Error + Send + Sync>> {
        self.info(json!({"type": "delegations", "user": user})).await
    }

    pub async fn get_staking_apy(&self) -> Result<f64, Box<dyn Error + Send + Sync>> {
        let validators = self.get_validators().await?;
        Ok(Validator::max_apr(validators))
    }

    pub async fn get_spot_balances(&self, user: &str) -> Result<Balances, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "spotClearinghouseState",
            "user": user
        }))
        .await
    }

    pub async fn get_stake_balance(&self, user: &str) -> Result<StakeBalance, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "delegatorSummary",
            "user": user
        }))
        .await
    }

    pub async fn get_user_fills_by_time(&self, user: &str, start_time: i64) -> Result<Vec<PerpetualFill>, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "userFillsByTime",
            "user": user,
            "startTime": start_time
        }))
        .await
    }

    pub async fn get_clearinghouse_state(&self, user: &str) -> Result<AssetPositions, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "clearinghouseState",
            "user": user
        }))
        .await
    }

    pub async fn get_metadata(&self) -> Result<HypercoreMetadataResponse, Box<dyn Error + Send + Sync>> {
        self.info(json!({"type": "metaAndAssetCtxs"})).await
    }

    pub async fn get_spot_meta(&self) -> Result<SpotMeta, Box<dyn Error + Send + Sync>> {
        let headers = HashMap::from([(String::from(X_CACHE_TTL), SPOT_META_CACHE_TTL_SECS.to_string())]);
        let response = self.client.post("/info", &json!({ "type": "spotMeta" }), Some(headers)).await?;
        let raw: SpotMetaRaw = serde_json::from_value(response)?;
        Ok(raw.into())
    }

    pub async fn get_spot_orderbook(&self, coin: &str) -> Result<OrderbookResponse, Box<dyn Error + Send + Sync>> {
        let response = self.info(json!({ "type": "l2Book", "coin": coin })).await?;
        Ok(serde_json::from_value(response)?)
    }

    pub async fn get_candlesticks(&self, coin: &str, interval: &str, start_time: i64, end_time: i64) -> Result<Vec<Candlestick>, Box<dyn Error + Send + Sync>> {
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

    pub async fn get_user_role(&self, user: &str) -> Result<UserRole, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "userRole",
            "user": user
        }))
        .await
    }

    pub async fn get_referral(&self, user: &str) -> Result<Referral, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "referral",
            "user": user
        }))
        .await
    }

    pub async fn get_extra_agents(&self, user: &str) -> Result<Vec<AgentSession>, Box<dyn Error + Send + Sync>> {
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

    pub async fn get_user_fees(&self, user: &str) -> Result<UserFee, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "userFees",
            "user": user
        }))
        .await
    }

    pub async fn get_ledger_updates(&self, user: &str) -> Result<Vec<LedgerUpdate>, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "userNonFundingLedgerUpdates",
            "user": user
        }))
        .await
    }

    pub async fn get_tx_hash_by_nonce(&self, user: &str, nonce: u64) -> Result<String, Box<dyn Error + Send + Sync>> {
        let updates = self.get_ledger_updates(user).await?;
        let update = updates.iter().find(|update| update.delta.nonce == Some(nonce)).ok_or("Nonce not found")?;
        Ok(update.hash.clone())
    }

    pub async fn get_open_orders(&self, user: &str) -> Result<Vec<OpenOrder>, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "frontendOpenOrders",
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

#[cfg(all(test, feature = "reqwest"))]
mod tests {
    use super::*;
    use gem_client::ReqwestClient;

    #[tokio::test]
    #[ignore]
    async fn test_get_tx_hash_by_nonce() {
        let url = "https://api.hyperliquid.xyz";
        let client = HyperCoreClient::new(ReqwestClient::new(url.to_string(), reqwest::Client::new()));
        let user = "0x1085c5f70f7f7591d97da281a64688385455c2bd";
        let nonce = 1758781366692_u64;

        let hash = client.get_tx_hash_by_nonce(user, nonce).await.unwrap();

        assert_eq!(hash, "0x610840f41a814c016281042c3882980202c800d9b5846ad304d0ec46d98525ec");
    }
}
