use crate::types::{JsonRpcError, JsonRpcRequest, JsonRpcRequestConvert, JsonRpcResult, ERROR_INTERNAL_ERROR};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::SystemTime;

pub struct JsonRpcClient {
    url: String,
    client: ClientWithMiddleware,
}

impl From<reqwest::Error> for JsonRpcError {
    fn from(value: reqwest::Error) -> Self {
        JsonRpcError {
            code: ERROR_INTERNAL_ERROR,
            message: value.to_string(),
        }
    }
}

impl From<reqwest_middleware::Error> for JsonRpcError {
    fn from(value: reqwest_middleware::Error) -> Self {
        JsonRpcError {
            code: ERROR_INTERNAL_ERROR,
            message: value.to_string(),
        }
    }
}

impl JsonRpcClient {
    pub fn new(url: String) -> Result<Self, anyhow::Error> {
        let client = ClientBuilder::new(reqwest::Client::new()).build();
        Ok(Self { url, client })
    }

    pub async fn request<T: JsonRpcRequestConvert, U: DeserializeOwned>(&self, request: T) -> Result<U, JsonRpcError> {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let req = request.to_req(timestamp);

        self._request(req).await
    }

    pub async fn call<T: DeserializeOwned>(&self, method: &str, params: Vec<Value>) -> Result<T, JsonRpcError> {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let req = JsonRpcRequest::new(timestamp, method, params);
        self._request(req).await
    }

    async fn _request<T: DeserializeOwned>(&self, req: JsonRpcRequest) -> Result<T, JsonRpcError> {
        let res = self.client.post(&self.url).json(&req).send().await?;
        let result = res.json::<JsonRpcResult<T>>().await?;

        match result {
            JsonRpcResult::Value(value) => Ok(value.result),
            JsonRpcResult::Error(error) => Err(error.error),
        }
    }
}
