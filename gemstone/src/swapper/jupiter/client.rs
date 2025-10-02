use super::model::*;
use gem_client::{CONTENT_TYPE, Client, ClientError};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct JupiterClient<C>
where
    C: Client + Clone,
{
    client: C,
}

impl<C> JupiterClient<C>
where
    C: Client + Clone,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_swap_quote(&self, request: QuoteRequest) -> Result<QuoteResponse, ClientError> {
        let query_string = serde_urlencoded::to_string(&request).map_err(|e| ClientError::Serialization(e.to_string()))?;
        let path = format!("/swap/v1/quote?{}", query_string);
        self.client.get(&path).await
    }

    pub async fn get_swap_quote_data(&self, request: &QuoteDataRequest) -> Result<QuoteDataResponse, ClientError> {
        let headers = HashMap::from([(CONTENT_TYPE.to_string(), "application/json".into())]);
        self.client.post("/swap/v1/swap", request, Some(headers)).await
    }
}
