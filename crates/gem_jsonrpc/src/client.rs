use crate::types::{ERROR_INTERNAL_ERROR, JsonRpcError, JsonRpcRequest, JsonRpcRequestConvert, JsonRpcResult, JsonRpcResults};
use gem_client::{Client, ClientError};
use serde::de::DeserializeOwned;
use serde_json::Value;
#[cfg(feature = "reqwest")]
use std::error::Error;
use std::time::SystemTime;

pub type CallTuple = (String, Value);

#[derive(Clone, Debug)]
pub struct JsonRpcClient<C: Client + Clone> {
    client: C,
}

impl From<ClientError> for JsonRpcError {
    fn from(value: ClientError) -> Self {
        JsonRpcError {
            code: ERROR_INTERNAL_ERROR,
            message: value.to_string(),
        }
    }
}

impl<C: Client + Clone> JsonRpcClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn request<T: JsonRpcRequestConvert, U: DeserializeOwned>(&self, request: T) -> Result<U, JsonRpcError> {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let req = request.to_req(timestamp);
        let result = self._request(req, None).await?;
        match result {
            JsonRpcResult::Value(value) => Ok(value.result),
            JsonRpcResult::Error(error) => Err(error.error),
        }
    }

    pub async fn call<T: DeserializeOwned>(&self, method: &str, params: impl Into<Value>) -> Result<T, JsonRpcError> {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let req = JsonRpcRequest::new(timestamp, method, params.into());
        let result = self._request(req, None).await?;
        match result {
            JsonRpcResult::Value(value) => Ok(value.result),
            JsonRpcResult::Error(error) => Err(error.error),
        }
    }

    pub async fn call_with_cache<T: JsonRpcRequestConvert, U: DeserializeOwned>(&self, call: &T, ttl: Option<u64>) -> Result<JsonRpcResult<U>, JsonRpcError> {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let req = call.to_req(timestamp);
        self._request(req, ttl).await
    }

    pub async fn call_method_with_param<T, U>(&self, method: &str, params: T, ttl: Option<u64>) -> Result<JsonRpcResult<U>, JsonRpcError>
    where
        T: serde::Serialize,
        U: DeserializeOwned,
    {
        let params_value = serde_json::to_value(params).map_err(|e| JsonRpcError {
            code: ERROR_INTERNAL_ERROR,
            message: format!("Failed to serialize RPC params: {e}"),
        })?;

        // Wrap single object/value in an array if it's not already an array
        let params_array = match params_value {
            serde_json::Value::Array(arr) => arr,
            _ => vec![params_value],
        };

        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let request = JsonRpcRequest::new(timestamp, method, params_array.into());
        self._request(request, ttl).await
    }

    pub async fn batch_call<T: DeserializeOwned>(&self, calls: Vec<CallTuple>) -> Result<JsonRpcResults<T>, JsonRpcError> {
        if calls.is_empty() {
            return Ok(Default::default());
        }
        let requests: Vec<JsonRpcRequest> = calls
            .iter()
            .enumerate()
            .map(|(index, (method, params))| JsonRpcRequest::new(index as u64 + 1, method, params.clone()))
            .collect();

        self.batch_request(requests).await
    }

    pub async fn batch_call_requests<T: JsonRpcRequestConvert, U: DeserializeOwned>(&self, calls: Vec<T>) -> Result<JsonRpcResults<U>, JsonRpcError> {
        let requests: Vec<JsonRpcRequest> = calls.iter().enumerate().map(|(index, request)| request.to_req(index as u64 + 1)).collect();
        self.batch_request(requests).await
    }

    pub async fn batch_request<T: DeserializeOwned>(&self, requests: Vec<JsonRpcRequest>) -> Result<JsonRpcResults<T>, JsonRpcError> {
        if requests.is_empty() {
            return Ok(Default::default());
        }

        let results: Vec<JsonRpcResult<T>> = self.client.post("", &requests, None).await?;
        if results.len() != requests.len() {
            return Err(JsonRpcError {
                message: "Batch call response length mismatch".into(),
                code: ERROR_INTERNAL_ERROR,
            });
        }

        Ok(JsonRpcResults(results))
    }

    async fn _request<T: DeserializeOwned>(&self, req: JsonRpcRequest, ttl: Option<u64>) -> Result<JsonRpcResult<T>, JsonRpcError> {
        // Build cache headers if TTL is provided
        let headers = ttl.map(|ttl_seconds| {
            let mut headers = std::collections::HashMap::new();
            headers.insert("Cache-Control".to_string(), format!("max-age={}", ttl_seconds));
            headers
        });

        let result: JsonRpcResult<T> = self.client.post("", &req, headers).await?;
        Ok(result)
    }
}

#[cfg(feature = "reqwest")]
impl JsonRpcClient<gem_client::ReqwestClient> {
    pub fn new_reqwest(url: String) -> Self {
        use gem_client::ReqwestClient;
        let reqwest_client = reqwest::Client::builder().build().expect("Failed to build reqwest client");
        let client = ReqwestClient::new(url, reqwest_client);
        Self { client }
    }
}

// Convenience functions for creating JsonRpcClient
#[cfg(feature = "reqwest")]
impl JsonRpcClient<gem_client::ReqwestClient> {
    pub fn new_default(url: String) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(Self::new_reqwest(url))
    }
}

// Module-level convenience function
#[cfg(feature = "reqwest")]
pub fn new_client(url: String) -> Result<JsonRpcClient<gem_client::ReqwestClient>, Box<dyn Error + Send + Sync>> {
    Ok(JsonRpcClient::new_reqwest(url))
}
