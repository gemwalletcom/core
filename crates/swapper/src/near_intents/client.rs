use crate::{SwapperError, config::get_swap_api_url};
use gem_client::{Client, ClientExt};
use std::{collections::HashMap, fmt::Debug};

use super::model::{ExplorerTransaction, QuoteRequest, QuoteResponseResult};

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
}

#[derive(Debug)]
pub struct NearIntentsExplorer<C: Client> {
    client: C,
}

impl<C: Client + Send + Sync + Debug> NearIntentsExplorer<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    async fn get_transactions(&self, query: &str) -> Result<Vec<ExplorerTransaction>, SwapperError> {
        let path = format!("/api/v0/transactions?{query}");
        self.client.get::<Vec<ExplorerTransaction>>(&path).await.map_err(SwapperError::from)
    }

    pub async fn search_transaction(&self, hash: &str) -> Result<Option<ExplorerTransaction>, SwapperError> {
        let transactions = self.get_transactions(&format!("search={hash}&numberOfTransactions=10")).await?;
        Ok(transactions.into_iter().find(|tx| tx.origin_chain_tx_hashes.iter().any(|h| h.eq_ignore_ascii_case(hash))))
    }
}
