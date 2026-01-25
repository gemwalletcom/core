use std::{collections::HashMap, fmt::Debug};

use gem_client::{CONTENT_TYPE, Client};

use super::model::{RelayQuoteRequest, RelayQuoteResponse, RelayStatusResponse};
use crate::SwapperError;

#[derive(Clone, Debug)]
pub struct RelayClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    client: C,
}

impl<C> RelayClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_quote(&self, request: RelayQuoteRequest) -> Result<RelayQuoteResponse, SwapperError> {
        let headers = HashMap::from([(CONTENT_TYPE.to_string(), "application/json".into())]);
        self.client.post("/quote/v2", &request, Some(headers)).await.map_err(SwapperError::from)
    }

    pub async fn get_swap_status(&self, request_id: &str) -> Result<RelayStatusResponse, SwapperError> {
        let path = format!("/intents/status?requestId={}", request_id);
        self.client.get(&path).await.map_err(SwapperError::from)
    }
}
