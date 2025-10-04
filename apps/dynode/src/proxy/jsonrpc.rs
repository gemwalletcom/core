use crate::cache::{CacheProvider, CachedResponse, RequestCache};
use crate::jsonrpc_types::{JsonRpcCall, JsonRpcRequest, JsonRpcResponse, JsonRpcResult};
use crate::metrics::Metrics;
use crate::proxy::constants::JSON_CONTENT_TYPE;
use crate::proxy::proxy_request::ProxyRequest;
use crate::proxy::request_builder::RequestBuilder;
use crate::proxy::request_url::RequestUrl;
use crate::proxy::response_builder::ResponseBuilder;
use gem_tracing::{DurationMs, info_with_fields};
use reqwest::header::HeaderMap;
use reqwest::{Method, StatusCode};

use crate::proxy::ProxyResponse;

pub struct JsonRpcHandler;

impl JsonRpcHandler {
    pub async fn handle_request(
        rpc_request: &JsonRpcRequest,
        request: &ProxyRequest,
        cache: &RequestCache,
        metrics: &Metrics,
        url: &RequestUrl,
        client: &reqwest::Client,
    ) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        match rpc_request {
            JsonRpcRequest::Single(call) => Self::handle_single_request(call, request, cache, metrics, url, client).await,
            JsonRpcRequest::Batch(calls) => Self::handle_batch_request(calls, request, metrics, url, client).await,
        }
    }

    async fn handle_single_request(
        call: &JsonRpcCall,
        request: &ProxyRequest,
        cache: &RequestCache,
        metrics: &Metrics,
        url: &RequestUrl,
        client: &reqwest::Client,
    ) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        metrics.add_proxy_request_by_method(&request.host, &call.method);

        let cache_key = call.cache_key(&request.host, &request.path_with_query);
        if let Some(_ttl) = cache.should_cache_call(&request.chain, call) {
            if let Some(cached) = cache.get(&request.chain, &cache_key).await {
                metrics.add_cache_hit(&request.host, &call.method);
                info_with_fields!(
                    "Cache HIT",
                    chain = request.chain.as_ref(),
                    host = request.host.as_str(),
                    method = call.method.as_str()
                );

                let result = serde_json::from_slice(&cached.body).unwrap_or_default();
                let response = JsonRpcResult::Success(JsonRpcResponse {
                    jsonrpc: call.jsonrpc.clone(),
                    result,
                    id: Some(call.id),
                });

                metrics.add_proxy_response(
                    &request.host,
                    &request.path_with_query,
                    &call.method,
                    url.url.host_str().unwrap_or_default(),
                    StatusCode::OK.as_u16(),
                    request.elapsed().as_millis(),
                );

                let upstream_headers = ResponseBuilder::create_upstream_headers(url.url.host_str(), request.elapsed());
                return Self::build_json_response(&response, upstream_headers, StatusCode::OK.as_u16());
            } else {
                metrics.add_cache_miss(&request.host, &call.method);
            }
        } else {
            metrics.add_cache_miss(&request.host, &call.method);
        }

        let (response, response_status) = Self::fetch_single_response(call, request, cache, url, client).await?;

        metrics.add_proxy_response(
            &request.host,
            &request.path_with_query,
            &call.method,
            url.url.host_str().unwrap_or_default(),
            response_status,
            request.elapsed().as_millis(),
        );

        match &response {
            JsonRpcResult::Success(_) => {
                info_with_fields!(
                    "Proxy response",
                    chain = request.chain.as_ref(),
                    host = request.host,
                    remote_host = url.url.host_str().unwrap_or_default(),
                    status = response_status,
                    latency = DurationMs(request.elapsed()),
                );
            }
            JsonRpcResult::Error(error_response) => {
                info_with_fields!(
                    "Proxy response",
                    chain = request.chain.as_ref(),
                    host = request.host,
                    remote_host = url.url.host_str().unwrap_or_default(),
                    status = response_status,
                    latency = DurationMs(request.elapsed()),
                    error_code = error_response.error.code,
                    error_message = error_response.error.message.as_str(),
                );
            }
        }

        let upstream_headers = ResponseBuilder::create_upstream_headers(url.url.host_str(), request.elapsed());
        Self::build_json_response(&response, upstream_headers, response_status)
    }

    async fn handle_batch_request(
        calls: &[JsonRpcCall],
        request: &ProxyRequest,
        metrics: &Metrics,
        url: &RequestUrl,
        client: &reqwest::Client,
    ) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        for call in calls {
            metrics.add_proxy_request_by_method(&request.host, &call.method);
            metrics.add_cache_miss(&request.host, &call.method);
        }

        let (responses, response_status) = Self::fetch_batch_responses(calls, url, client, &request.method).await?;

        for call in calls {
            metrics.add_proxy_response(
                &request.host,
                &request.path_with_query,
                &call.method,
                url.url.host_str().unwrap_or_default(),
                response_status,
                request.elapsed().as_millis(),
            );
        }

        info_with_fields!(
            "Proxy response",
            chain = request.chain.as_ref(),
            host = request.host,
            remote_host = url.url.host_str().unwrap_or_default(),
            status = response_status,
            latency = DurationMs(request.elapsed()),
        );

        let upstream_headers = ResponseBuilder::create_upstream_headers(url.url.host_str(), request.elapsed());
        Self::build_json_response(&responses, upstream_headers, response_status)
    }

    async fn fetch_single_response(
        call: &JsonRpcCall,
        request: &ProxyRequest,
        cache: &RequestCache,
        url: &RequestUrl,
        client: &reqwest::Client,
    ) -> Result<(JsonRpcResult, u16), Box<dyn std::error::Error + Send + Sync>> {
        let body = serde_json::to_vec(&call)?;
        let req = RequestBuilder::build_jsonrpc(url, &request.method, body)?;

        let response = client.execute(req).await?;
        let status = response.status().as_u16();
        let body_bytes = response.bytes().await?.to_vec();

        let result: JsonRpcResult = serde_json::from_slice(&body_bytes)?;

        if status == StatusCode::OK.as_u16() {
            if let (JsonRpcResult::Success(success), Some(ttl)) = (&result, cache.should_cache_call(&request.chain, call)) {
                let result_bytes = serde_json::to_string(&success.result).unwrap_or_default().into_bytes();
                let size_bytes = result_bytes.len();
                let cached = CachedResponse::new(result_bytes, StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string(), ttl);
                let cache_key = call.cache_key(&request.host, &request.path_with_query);
                cache.set(&request.chain, cache_key, cached, ttl).await;

                info_with_fields!(
                    "Cache SET",
                    chain = request.chain.as_ref(),
                    host = request.host.as_str(),
                    method = call.method.as_str(),
                    ttl_seconds = ttl,
                    size_bytes = size_bytes,
                    latency = DurationMs(request.elapsed()),
                );
            }
        } else {
            match &result {
                JsonRpcResult::Error(error_response) => {
                    info_with_fields!(
                        "JSON RPC error response",
                        chain = request.chain.as_ref(),
                        host = request.host.as_str(),
                        method = call.method.as_str(),
                        remote_host = url.url.host_str().unwrap_or(""),
                        status = status,
                        latency = DurationMs(request.elapsed()),
                        error_code = error_response.error.code,
                        error_message = error_response.error.message.as_str()
                    );
                }
                JsonRpcResult::Success(_) => {
                    info_with_fields!(
                        "JSON RPC success response",
                        chain = request.chain.as_ref(),
                        host = request.host.as_str(),
                        method = call.method.as_str(),
                        remote_host = url.url.host_str().unwrap_or(""),
                        status = status,
                        latency = DurationMs(request.elapsed())
                    );
                }
            }
        }

        Ok((result, status))
    }

    async fn fetch_batch_responses(
        calls: &[JsonRpcCall],
        url: &RequestUrl,
        client: &reqwest::Client,
        method: &Method,
    ) -> Result<(serde_json::Value, u16), Box<dyn std::error::Error + Send + Sync>> {
        let body = serde_json::to_vec(&calls)?;
        let req = RequestBuilder::build_jsonrpc(url, method, body)?;

        let response = client.execute(req).await?;
        let status = response.status().as_u16();
        let body_bytes = response.bytes().await?.to_vec();
        let responses: serde_json::Value = serde_json::from_slice(&body_bytes)?;
        Ok((responses, status))
    }

    fn build_json_response<T: serde::Serialize>(data: &T, headers: HeaderMap, status: u16) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let response_body = serde_json::to_vec(data)?;
        ResponseBuilder::build_with_headers(response_body, status, JSON_CONTENT_TYPE, headers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_call(id: u64, method: &str) -> JsonRpcCall {
        JsonRpcCall {
            jsonrpc: "2.0".into(),
            method: method.into(),
            params: json!([]),
            id,
        }
    }

    #[test]
    fn test_single_and_batch_request_distinction() {
        let single_call = make_call(1, "eth_blockNumber");
        let batch_calls = vec![make_call(1, "eth_blockNumber"), make_call(2, "eth_gasPrice")];

        let single_request = JsonRpcRequest::Single(single_call);
        let batch_request = JsonRpcRequest::Batch(batch_calls);

        assert!(matches!(single_request, JsonRpcRequest::Single(_)));
        assert!(matches!(batch_request, JsonRpcRequest::Batch(_)));
    }
}
