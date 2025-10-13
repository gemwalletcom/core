use super::model::*;
use gem_client::{APPLICATION_JSON, CONTENT_TYPE, Client, ClientError};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct DFlowClient<C>
where
    C: Client + Clone,
{
    client: C,
}

impl<C> DFlowClient<C>
where
    C: Client + Clone,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_swap_quote(&self, request: QuoteRequest) -> Result<QuoteResponse, ClientError> {
        let query_string = serde_urlencoded::to_string(&request).map_err(|e| ClientError::Serialization(e.to_string()))?;
        let path = format!("/quote?{}", query_string);
        self.client.get(&path).await
    }

    pub async fn get_swap_quote_data(&self, request: &QuoteDataRequest) -> Result<QuoteDataResponse, ClientError> {
        let headers = HashMap::from([(CONTENT_TYPE.to_string(), APPLICATION_JSON.to_string())]);
        self.client.post("/swap", request, Some(headers)).await
    }
}
