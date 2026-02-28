use crate::{SwapperError, config::get_swap_api_url};
use gem_client::{Client, ClientError, ClientExt};
use std::{collections::HashMap, fmt::Debug};

use super::model::{DEFAULT_REFERRAL, ExecutionStatus, ExplorerTransaction, QuoteRequest, QuoteResponseResult};

pub fn base_url() -> String {
    get_swap_api_url("near-intents/1click")
}

pub fn explorer_url() -> String {
    get_swap_api_url("near-intents/explorer")
}

#[derive(Clone, Debug)]
pub struct NearIntentsClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    client: C,
    api_token: Option<String>,
}

impl<C> NearIntentsClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new(client: C, api_key: Option<String>) -> Self {
        Self { client, api_token: api_key }
    }

    fn build_headers(&self) -> HashMap<String, String> {
        self.api_token
            .as_ref()
            .map(|token| HashMap::from([(String::from("Authorization"), format!("Bearer {token}"))]))
            .unwrap_or_default()
    }

    pub async fn fetch_quote(&self, request: &QuoteRequest) -> Result<QuoteResponseResult, SwapperError> {
        self.client.post_with_headers("/v0/quote", request, self.build_headers()).await.map_err(SwapperError::from)
    }

    pub async fn get_transaction_status(&self, deposit_address: &str) -> Result<ExecutionStatus, SwapperError> {
        let path = format!("/v0/status?depositAddress={deposit_address}");

        match self.client.get_with_headers::<ExecutionStatus>(&path, self.build_headers()).await {
            Ok(result) => Ok(result),
            Err(ClientError::Http { status: 404, .. }) => Ok(ExecutionStatus {
                quote_response: None,
                status: "UNKNOWN".into(),
                updated_at: String::new(),
                swap_details: None,
            }),
            Err(err) => Err(SwapperError::from(err)),
        }
    }
}

#[derive(Debug)]
pub struct NearIntentsExplorer<C: Client> {
    client: C,
}

impl<C: Client + Send + Sync + Debug> NearIntentsExplorer<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_deposit_addresses(&self, start_timestamp: u64) -> Result<Vec<String>, SwapperError> {
        let path = format!(
            "/api/v0/transactions?referral={}&startTimestampUnix={}&statuses=PENDING_DEPOSIT,FAILED,PROCESSING,REFUNDED,SUCCESS",
            DEFAULT_REFERRAL, start_timestamp
        );
        let transactions = self.client.get::<Vec<ExplorerTransaction>>(&path).await.map_err(SwapperError::from)?;
        Ok(transactions.into_iter().map(|t| t.deposit_address).collect())
    }
}
