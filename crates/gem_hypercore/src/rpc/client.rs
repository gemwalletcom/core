use crate::models::balance::{HypercoreBalances, HypercoreDelegationBalance, HypercoreStakeBalance, HypercoreValidator};
use crate::models::candlestick::HypercoreCandlestick;
use crate::models::metadata::HypercoreMetadataResponse;
use crate::models::order::HypercorePerpetualFill;
use crate::models::position::HypercoreAssetPositions;
use crate::models::referral::HypercoreReferral;
use crate::models::response::{HyperCoreBroadcastResult, TransactionBroadcastResponse};
use crate::models::user::{HypercoreAgentSession, HypercoreUserFee, HypercoreUserRole};
use async_trait::async_trait;
use chain_traits::{ChainTraits, ChainTransactionLoad};
use gem_client::Client;
use num_bigint::BigInt;
use primitives::{
    Chain, FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput,
};
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

    async fn info<T>(&self, payload: serde_json::Value) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned,
    {
        Ok(self.client.post("/info", &payload, None).await?)
    }

    async fn exchange(&self, payload: serde_json::Value) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post("/exchange", &payload, None).await?)
    }

    pub async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        let json_data: serde_json::Value = serde_json::from_str(&data)?;
        let response = self.exchange(json_data).await?;
        match serde_json::from_value::<TransactionBroadcastResponse>(response)?.into_result(data) {
            HyperCoreBroadcastResult::Success(result) => Ok(result),
            HyperCoreBroadcastResult::Error(error) => Err(error.into()),
        }
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
        })).await
    }

    pub async fn get_stake_balance(&self, user: &str) -> Result<HypercoreStakeBalance, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "delegatorSummary",
            "user": user
        })).await
    }

    pub async fn get_user_fills_by_time(&self, user: &str, start_time: i64) -> Result<Vec<HypercorePerpetualFill>, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "userFillsByTime",
            "user": user,
            "startTime": start_time
        })).await
    }

    pub async fn get_clearinghouse_state(&self, user: &str) -> Result<HypercoreAssetPositions, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "clearinghouseState",
            "user": user
        })).await
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
        })).await
    }

    pub async fn get_user_role(&self, user: &str) -> Result<HypercoreUserRole, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "userRole",
            "user": user
        })).await
    }

    pub async fn get_referral(&self, user: &str) -> Result<HypercoreReferral, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "referral",
            "user": user
        })).await
    }

    pub async fn get_extra_agents(&self, user: &str) -> Result<Vec<HypercoreAgentSession>, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "extraAgents",
            "user": user
        })).await
    }

    pub async fn get_builder_fee(&self, user: &str, builder: &str) -> Result<i32, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "maxBuilderFee",
            "user": user,
            "builder": builder
        })).await
    }

    pub async fn get_user_fees(&self, user: &str) -> Result<HypercoreUserFee, Box<dyn Error + Send + Sync>> {
        self.info(json!({
            "type": "userFees",
            "user": user
        })).await
    }
}

#[async_trait]
impl<C: Client> ChainTransactionLoad for HyperCoreClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: TransactionFee::default(),
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(1)))])
    }
}

impl<C: Client> ChainTraits for HyperCoreClient<C> {}
