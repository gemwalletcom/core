use super::model::*;
use crate::network::{AlienError, AlienHttpMethod, AlienProvider, AlienTarget};
use serde_json;
use std::{collections::HashMap, sync::Arc};

pub struct JupiterClient {
    api_url: String,
    provider: Arc<dyn AlienProvider>,
}

impl JupiterClient {
    pub fn new(url: String, provider: Arc<dyn AlienProvider>) -> Self {
        Self { api_url: url, provider }
    }

    pub async fn get_swap_quote(&self, request: QuoteRequest) -> Result<QuoteResponse, AlienError> {
        let query_string = serde_urlencoded::to_string(&request).map_err(|e| AlienError::RequestError { msg: e.to_string() })?;
        let target = AlienTarget {
            url: format!("{}/v6/quote?{}", self.api_url, &query_string),
            method: AlienHttpMethod::Get,
            headers: None,
            body: None,
        };
        let response = self.provider.request(target).await?;
        let quote_response: QuoteResponse = serde_json::from_slice(&response).map_err(|e| AlienError::ResponseError { msg: e.to_string() })?;
        Ok(quote_response)
    }
    pub async fn get_swap_quote_data(&self, request: QuoteDataRequest) -> Result<QuoteDataResponse, AlienError> {
        let headers = HashMap::from([("Content-Type".to_string(), "application/json".to_string())]);
        let json = serde_json::to_string(&request).map_err(|e| AlienError::RequestError { msg: e.to_string() })?;
        let target = AlienTarget {
            url: format!("{}/v6/swap", self.api_url),
            method: AlienHttpMethod::Post,
            headers: Some(headers),
            body: Some(json.as_bytes().into()),
        };
        let response = self.provider.request(target).await?;
        let quote_response: QuoteDataResponse = serde_json::from_slice(&response).map_err(|e| AlienError::ResponseError { msg: e.to_string() })?;
        Ok(quote_response)
    }
}
