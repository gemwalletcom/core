use crate::types::{JsonRpcError, JsonRpcRequest, JsonRpcRequestConvert, JsonRpcResult, ERROR_INTERNAL_ERROR};
use gem_client::{Client, ClientError};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::SystemTime;

pub type CallTuple = (String, Value);

#[derive(Clone, Debug)]
pub struct JsonRpcClient<C: Client + Clone> {
    url: String,
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
    pub fn new(url: String, client: C) -> Self {
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

    pub async fn call_with_cache<T: JsonRpcRequestConvert, U: DeserializeOwned>(&self, call: &T, _ttl: Option<u64>) -> Result<JsonRpcResult<U>, JsonRpcError> {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let req = call.to_req(timestamp);
        self._request_raw(req).await
    }

    pub async fn call_method_with_param<T, U>(&self, method: &str, params: T, _ttl: Option<u64>) -> Result<JsonRpcResult<U>, JsonRpcError>
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
        self._request_raw(request).await
    }

    pub async fn batch_call<T: DeserializeOwned>(&self, calls: Vec<CallTuple>) -> Result<Vec<JsonRpcResult<T>>, JsonRpcError> {
        let requests: Vec<JsonRpcRequest> = calls
            .iter()
            .enumerate()
            .map(|(index, (method, params))| JsonRpcRequest::new(index as u64 + 1, method, params.clone()))
            .collect();

        self.batch_request(requests).await
    }

    pub async fn batch_call_requests<T: JsonRpcRequestConvert, U: DeserializeOwned>(&self, calls: Vec<T>) -> Result<Vec<JsonRpcResult<U>>, JsonRpcError> {
        let requests: Vec<JsonRpcRequest> = calls.iter().enumerate().map(|(index, request)| request.to_req(index as u64 + 1)).collect();
        self.batch_request(requests).await
    }

    pub async fn batch_request<T: DeserializeOwned>(&self, requests: Vec<JsonRpcRequest>) -> Result<Vec<JsonRpcResult<T>>, JsonRpcError> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        let path = if self.url.is_empty() { "" } else { &self.url };
        let results: Vec<JsonRpcResult<T>> = self.client.post(path, &requests, None).await?;
        if results.len() != requests.len() {
            return Err(JsonRpcError {
                message: "Batch call response length mismatch".into(),
                code: ERROR_INTERNAL_ERROR,
            });
        }

        Ok(results)
    }

    async fn _request<T: DeserializeOwned>(&self, req: JsonRpcRequest) -> Result<T, JsonRpcError> {
        let result = self._request_raw(req).await?;
        match result {
            JsonRpcResult::Value(value) => Ok(value.result),
            JsonRpcResult::Error(error) => Err(error.error),
        }
    }

    async fn _request_raw<T: DeserializeOwned>(&self, req: JsonRpcRequest) -> Result<JsonRpcResult<T>, JsonRpcError> {
        let path = if self.url.is_empty() { "" } else { &self.url };
        let result: JsonRpcResult<T> = self.client.post(path, &req, None).await?;
        Ok(result)
    }
}

#[cfg(feature = "reqwest")]
impl JsonRpcClient<gem_client::ReqwestClient> {
    pub fn new_reqwest(url: String) -> Self {
        use gem_client::ReqwestClient;
        let reqwest_client = reqwest::Client::new();
        let client = ReqwestClient::new(url.clone(), reqwest_client);
        Self { url: "".to_string(), client }
    }
}

// Convenience functions for creating JsonRpcClient
#[cfg(feature = "reqwest")]
impl JsonRpcClient<gem_client::ReqwestClient> {
    pub fn new_default(url: String) -> Result<Self, anyhow::Error> {
        Ok(Self::new_reqwest(url))
    }
}

// Module-level convenience function
#[cfg(feature = "reqwest")]
pub fn new_client(url: String) -> Result<JsonRpcClient<gem_client::ReqwestClient>, anyhow::Error> {
    Ok(JsonRpcClient::new_reqwest(url))
}
