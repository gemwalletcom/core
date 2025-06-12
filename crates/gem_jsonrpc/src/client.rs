use crate::types::{JsonRpcError, JsonRpcRequest, JsonRpcRequestConvert, JsonRpcResult, ERROR_INTERNAL_ERROR, ERROR_INVALID_REQUEST};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::SystemTime;

pub type CallTuple = (String, Value);

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

    pub fn new_with_client(url: String, client: ClientWithMiddleware) -> Self {
        Self { url, client }
    }

    pub async fn request<T: JsonRpcRequestConvert, U: DeserializeOwned>(&self, request: T) -> Result<U, JsonRpcError> {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let req = request.to_req(timestamp);
        self._request(req).await
    }

    pub async fn call<T: DeserializeOwned>(&self, method: &str, params: impl Into<Value>) -> Result<T, JsonRpcError> {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let req = JsonRpcRequest::new(timestamp, method, params.into());
        self._request(req).await
    }

    pub async fn batch_call<T: DeserializeOwned>(&self, calls: Vec<CallTuple>) -> Result<Vec<JsonRpcResult<T>>, JsonRpcError> {
        let requests: Vec<JsonRpcRequest> = calls
            .iter()
            .enumerate()
            .map(|(index, (method, params))| JsonRpcRequest::new(index as u64 + 1, method, params.clone()))
            .collect();

        self.batch_request(requests).await
    }

    pub async fn batch_request<T: DeserializeOwned>(&self, requests: Vec<JsonRpcRequest>) -> Result<Vec<JsonRpcResult<T>>, JsonRpcError> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        let body = serde_json::to_vec(&requests).map_err(|_| JsonRpcError {
            code: ERROR_INVALID_REQUEST,
            message: "Failed to serialize requests".into(),
        })?;
        let response = self.client.post(&self.url).body(body).send().await?;
        if response.status().is_client_error() || response.status().is_server_error() {
            return Err(JsonRpcError::from(reqwest_middleware::Error::from(response.error_for_status().unwrap_err())));
        }
        let results = response.json::<Vec<JsonRpcResult<T>>>().await?;
        if results.len() != requests.len() {
            return Err(JsonRpcError {
                message: "Batch call response length mismatch".into(),
                code: ERROR_INTERNAL_ERROR,
            });
        }

        Ok(results)
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
