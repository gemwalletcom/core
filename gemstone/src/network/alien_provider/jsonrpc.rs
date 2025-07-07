use super::{mime::*, AlienError, AlienHttpMethod, AlienProvider, AlienTarget};
use gem_jsonrpc::types::{JsonRpcError, JsonRpcRequest, JsonRpcRequestConvert, JsonRpcResult};
use primitives::Chain;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::{fmt::Debug, sync::Arc};

impl AlienTarget {
    pub fn from_json_rpc_request(request: &JsonRpcRequest, url: &str) -> Result<Self, AlienError> {
        let headers = HashMap::from([(CONTENT_TYPE.into(), JSON.into())]);
        let body = serde_json::to_vec(request).map_err(|e| AlienError::RequestError {
            msg: format!("Failed to serialize RPC request: {e}"),
        })?;
        Ok(AlienTarget {
            url: url.into(),
            method: AlienHttpMethod::Post,
            headers: Some(headers),
            body: Some(body),
        })
    }
}

impl From<JsonRpcError> for AlienError {
    fn from(err: JsonRpcError) -> Self {
        Self::ResponseError { msg: err.message }
    }
}

impl From<AlienError> for JsonRpcError {
    fn from(err: AlienError) -> Self {
        Self {
            code: -1,
            message: err.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcResponse<T> {
    pub id: u64,
    pub result: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcErrorResponse {
    pub id: u64,
    pub error: JsonRpcError,
}

pub fn batch_into_target<T>(requests: &T, endpoint: &str) -> AlienTarget
where
    T: ?Sized + Serialize,
{
    let headers = HashMap::from([(CONTENT_TYPE.into(), JSON.into())]);
    let bytes = serde_json::to_vec(requests).unwrap();
    AlienTarget {
        url: endpoint.into(),
        method: AlienHttpMethod::Post,
        headers: Some(headers),
        body: Some(bytes),
    }
}

#[derive(Debug)]
pub struct JsonRpcClient {
    provider: Arc<dyn AlienProvider>,
    endpoint: String,
}

impl JsonRpcClient {
    pub fn new(provider: Arc<dyn AlienProvider>, endpoint: String) -> Self {
        Self { provider, endpoint }
    }

    pub fn new_with_chain(provider: Arc<dyn AlienProvider>, chain: Chain) -> Self {
        let endpoint = provider.get_endpoint(chain).unwrap();
        Self::new(provider, endpoint)
    }

    pub async fn call<T, U>(&self, call: &T) -> Result<JsonRpcResult<U>, JsonRpcError>
    where
        T: JsonRpcRequestConvert,
        U: DeserializeOwned,
    {
        self.call_with_cache(call, None).await
    }

    pub async fn call_method_with_param<T, U>(&self, method: &str, params: T, ttl: Option<u64>) -> Result<JsonRpcResult<U>, AlienError>
    where
        T: Serialize,
        U: DeserializeOwned,
    {
        let params_value = serde_json::to_value(params).map_err(|e| AlienError::RequestError {
            msg: format!("Failed to serialize RPC params: {e}"),
        })?;

        // Wrap single object/value in an array if it's not already an array
        let params_array = match params_value {
            serde_json::Value::Array(arr) => arr,
            _ => vec![params_value],
        };

        let request = JsonRpcRequest::new(1, method, params_array.into());
        let mut target = AlienTarget::from_json_rpc_request(&request, &self.endpoint)?;
        if let Some(ttl) = ttl {
            target = target.set_cache_ttl(ttl);
        }
        let response_data = self.provider.request(target).await?;

        // Deserialize into the JsonRpcResult enum first
        let rpc_result: JsonRpcResult<U> = serde_json::from_slice(&response_data).map_err(|e| AlienError::ResponseError {
            msg: format!("Failed to parse JSON-RPC response: {e}"),
        })?;

        Ok(rpc_result)
    }

    pub async fn call_with_cache<T, U>(&self, call: &T, ttl: Option<u64>) -> Result<JsonRpcResult<U>, JsonRpcError>
    where
        T: JsonRpcRequestConvert,
        U: DeserializeOwned,
    {
        let request = call.to_req(1);
        let mut target = batch_into_target(&request, &self.endpoint);
        if let Some(ttl) = ttl {
            target = target.set_cache_ttl(ttl);
        }
        let data = self.provider.request(target).await?;
        let result: JsonRpcResult<U> = serde_json::from_slice(&data).map_err(|err| AlienError::ResponseError { msg: err.to_string() })?;
        Ok(result)
    }

    pub async fn batch_call<T, U>(&self, calls: Vec<T>) -> Result<Vec<JsonRpcResult<U>>, AlienError>
    where
        T: JsonRpcRequestConvert,
        U: DeserializeOwned,
    {
        let requests: Vec<JsonRpcRequest> = calls.iter().enumerate().map(|(index, request)| request.to_req(index as u64 + 1)).collect();

        let targets = vec![batch_into_target(&requests, &self.endpoint)];

        let data_array = self.provider.batch_request(targets).await?;
        let data = data_array.first().ok_or(AlienError::ResponseError { msg: "No result".into() })?;

        let results: Vec<JsonRpcResult<U>> = serde_json::from_slice(data).map_err(|err| AlienError::ResponseError { msg: err.to_string() })?;
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_batch_into_target() {
        let requests = vec![
            JsonRpcRequest::new(1, "eth_gasPrice", json!([])),
            JsonRpcRequest::new(2, "eth_blockNumber", json!(vec!["0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5", "latest"])),
            JsonRpcRequest::new(3, "eth_chainId", json!([])),
        ];
        let endpoint = "http://localhost:8080";
        let target = batch_into_target(&requests, endpoint);

        assert_eq!(target.url, endpoint);
        assert_eq!(target.method, AlienHttpMethod::Post);
        assert_eq!(target.headers.as_ref().unwrap().get(CONTENT_TYPE).unwrap(), "application/json");
        assert_eq!(
            String::from_utf8(target.body.unwrap()).unwrap(),
            r#"[{"jsonrpc":"2.0","id":1,"method":"eth_gasPrice","params":[]},{"jsonrpc":"2.0","id":2,"method":"eth_blockNumber","params":["0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5","latest"]},{"jsonrpc":"2.0","id":3,"method":"eth_chainId","params":[]}]"#
        );
    }

    #[test]
    fn test_decode_json_rpc_error_response() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": 3,
                "message": "execution reverted: revert: toAddress_outOfBounds"
            }
        }"#;
        let result = serde_json::from_str::<JsonRpcResult<String>>(json).unwrap();
        if let JsonRpcResult::Error(value) = result {
            assert_eq!(value.id, 1);
            assert_eq!(value.error.code, 3);
            assert_eq!(value.error.message, "execution reverted: revert: toAddress_outOfBounds");
        } else {
            panic!("unexpected response: {:?}", result);
        }
    }

    #[test]
    fn test_decode_json_rpc_response() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x21e3bb1a6"
        }"#;
        let result = serde_json::from_str::<JsonRpcResult<String>>(json).unwrap();
        if let JsonRpcResult::Value(value) = result {
            assert_eq!(value.id, 1);
            assert_eq!(value.result, "0x21e3bb1a6");
        } else {
            panic!("unexpected response: {:?}", result);
        }
    }
}
