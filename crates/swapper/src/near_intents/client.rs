use crate::SwapperError;
use gem_client::{Client, ClientError};
use std::{collections::HashMap, fmt::Debug};

use super::model::{NearIntentsExecutionStatus, NearIntentsQuoteRequest, NearIntentsQuoteResponse};

pub const DEFAULT_NEAR_INTENTS_BASE_URL: &str = "https://1click.chaindefuser.com";

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

    fn build_headers(&self) -> Option<HashMap<String, String>> {
        self.api_token
            .as_ref()
            .map(|token| HashMap::from([(String::from("Authorization"), format!("Bearer {token}"))]))
    }

    pub async fn fetch_quote(&self, request: &NearIntentsQuoteRequest) -> Result<NearIntentsQuoteResponse, SwapperError> {
        self.client.post("/v0/quote", request, self.build_headers()).await.map_err(SwapperError::from)
    }

    pub async fn get_transaction_status(&self, deposit_address: &str) -> Result<NearIntentsExecutionStatus, SwapperError> {
        let path = format!("/v0/status?depositAddress={deposit_address}");

        match self.client.get_with_headers::<NearIntentsExecutionStatus>(&path, self.build_headers()).await {
            Ok(result) => Ok(result),
            Err(ClientError::Http { status: 404, .. }) => Ok(NearIntentsExecutionStatus {
                quote_response: None,
                status: "UNKNOWN".into(),
                updated_at: String::new(),
                swap_details: None,
            }),
            Err(err) => Err(SwapperError::from(err)),
        }
    }
}
