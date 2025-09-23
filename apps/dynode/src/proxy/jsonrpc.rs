use crate::cache::{CacheProvider, CachedResponse, RequestCache};
use crate::jsonrpc_types::{JsonRpcCall, JsonRpcError, JsonRpcErrorResponse, JsonRpcRequest, JsonRpcResponse, JsonRpcResult};
use crate::metrics::Metrics;
use crate::proxy::constants::JSON_CONTENT_TYPE;
use crate::proxy::request_builder::RequestBuilder;
use crate::proxy::request_url::RequestUrl;
use crate::proxy::response_builder::ResponseBuilder;
use gem_tracing::{DurationMs, info_with_fields};
use primitives::Chain;
use reqwest::Method;
use reqwest::header::HeaderMap;

use crate::proxy::ProxyResponse;

pub struct JsonRpcHandler;

impl JsonRpcHandler {
    pub async fn handle_request(
        request: &JsonRpcRequest,
        chain: Chain,
        host: &str,
        path: &str,
        cache: &RequestCache,
        metrics: &Metrics,
        url: &RequestUrl,
        client: &reqwest::Client,
        method: &Method,
        start_time: std::time::Instant,
    ) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let calls = request.get_calls();

        for call in &calls {
            metrics.add_proxy_request_by_method(host, &call.method);
        }

        let (cached_responses, uncached_indices) = Self::check_cache(&calls, chain, host, path, cache, metrics).await;

        let upstream_responses = if uncached_indices.is_empty() {
            Vec::new()
        } else {
            let uncached_calls: Vec<&JsonRpcCall> = uncached_indices.iter().map(|&i| calls[i]).collect();
            Self::fetch_responses(&uncached_calls, chain, host, path, cache, url, client, method, start_time).await?
        };

        let responses = Self::build_responses(&calls, &cached_responses, &upstream_responses, uncached_indices.clone());

        for call in &calls {
            metrics.add_proxy_response(
                host,
                path,
                &call.method,
                url.url.host_str().unwrap_or_default(),
                200,
                start_time.elapsed().as_millis(),
            );
        }

        info_with_fields!(
            "Proxy response",
            host = url.url.host_str().unwrap_or_default(),
            status = 200,
            latency = DurationMs(start_time.elapsed()),
        );

        let upstream_headers = ResponseBuilder::create_upstream_headers(url.url.host_str(), start_time.elapsed());

        match request {
            JsonRpcRequest::Single(_) => Self::build_json_response_with_headers(&responses[0], upstream_headers),
            JsonRpcRequest::Batch(_) => Self::build_json_response_with_headers(&responses, upstream_headers),
        }
    }

    async fn check_cache(
        calls: &[&JsonRpcCall],
        chain: Chain,
        host: &str,
        path: &str,
        cache: &RequestCache,
        metrics: &Metrics,
    ) -> (Vec<Option<CachedResponse>>, Vec<usize>) {
        let mut cached_responses = Vec::new();
        let mut uncached_indices = Vec::new();

        for (i, call) in calls.iter().enumerate() {
            let cache_key = call.cache_key(host, path);
            if cache.should_cache_call(&chain, call).is_some() {
                if let Some(cached) = cache.get(&chain, &cache_key).await {
                    cached_responses.push(Some(cached));
                    metrics.add_cache_hit(host, &call.method);

                    info_with_fields!("Cache HIT", chain = chain.as_ref(), host = host, method = call.method.as_str(),);
                } else {
                    cached_responses.push(None);
                    uncached_indices.push(i);
                    metrics.add_cache_miss(host, &call.method);
                }
            } else {
                cached_responses.push(None);
                uncached_indices.push(i);
                metrics.add_cache_miss(host, &call.method);
            }
        }

        (cached_responses, uncached_indices)
    }

    async fn fetch_responses(
        calls: &[&JsonRpcCall],
        chain: Chain,
        host: &str,
        path: &str,
        cache: &RequestCache,
        url: &RequestUrl,
        client: &reqwest::Client,
        method: &Method,
        start_time: std::time::Instant,
    ) -> Result<Vec<JsonRpcResult>, Box<dyn std::error::Error + Send + Sync>> {
        let body = if calls.len() == 1 {
            serde_json::to_vec(&calls[0])?
        } else {
            serde_json::to_vec(&calls)?
        };

        let req = RequestBuilder::build_jsonrpc(url, method, body)?;

        let response = client.execute(req).await?;
        let status = response.status().as_u16();
        let body_bytes = response.bytes().await?.to_vec();

        let responses = if status == 200 {
            if calls.len() == 1 {
                vec![serde_json::from_slice(&body_bytes)?]
            } else {
                serde_json::from_slice(&body_bytes)?
            }
        } else {
            Vec::new()
        };

        Self::cache_responses(&responses, calls, chain, host, path, cache, start_time).await;

        Ok(responses)
    }

    async fn cache_responses(
        responses: &[JsonRpcResult],
        calls: &[&JsonRpcCall],
        chain: Chain,
        host: &str,
        path: &str,
        cache: &RequestCache,
        now: std::time::Instant,
    ) {
        for (i, response) in responses.iter().enumerate() {
            if let (Some(call), JsonRpcResult::Success(success)) = (calls.get(i), response)
                && let Some(ttl) = cache.should_cache_call(&chain, call)
            {
                let result_bytes = serde_json::to_string(&success.result).unwrap_or_default().into_bytes();
                let size_bytes = result_bytes.len();
                let cached = CachedResponse::new(result_bytes, 200, JSON_CONTENT_TYPE.to_string(), ttl);
                let cache_key = call.cache_key(host, path);
                cache.set(&chain, cache_key, cached, ttl).await;

                info_with_fields!(
                    "Cache SET",
                    chain = chain.as_ref(),
                    host = host,
                    method = call.method.as_str(),
                    ttl_seconds = ttl,
                    size_bytes = size_bytes,
                    latency = DurationMs(now.elapsed()),
                );
            }
        }
    }

    pub(crate) fn build_responses(calls: &[&JsonRpcCall], cached: &[Option<CachedResponse>], upstream: &[JsonRpcResult], _: Vec<usize>) -> Vec<JsonRpcResult> {
        let mut upstream_idx = 0;
        calls
            .iter()
            .enumerate()
            .map(|(i, call)| {
                if let Some(Some(cache)) = cached.get(i) {
                    let result = serde_json::from_slice(&cache.body).unwrap_or_default();
                    JsonRpcResult::Success(JsonRpcResponse { result, id: call.id })
                } else if let Some(response) = upstream.get(upstream_idx) {
                    upstream_idx += 1;
                    response.clone()
                } else {
                    JsonRpcResult::Error(JsonRpcErrorResponse {
                        error: JsonRpcError {
                            code: -32603,
                            message: "Internal error".to_string(),
                            data: Some(serde_json::json!({"reason": "No response received"})),
                        },
                        id: call.id,
                    })
                }
            })
            .collect()
    }

    fn build_json_response_with_headers<T: serde::Serialize>(data: &T, headers: HeaderMap) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let response_body = serde_json::to_vec(data)?;
        ResponseBuilder::build_with_headers(response_body, 200, JSON_CONTENT_TYPE, headers)
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
    fn test_build_responses_prefers_cache_then_upstream() {
        let call1 = make_call(1, "eth_blockNumber");
        let call2 = make_call(2, "eth_gasPrice");
        let calls = vec![&call1, &call2];

        let cached_body = serde_json::to_vec(&json!("0x123")).unwrap();
        let cached = CachedResponse::new(cached_body, 200, "application/json".into(), 60);
        let cached_vec = vec![Some(cached), None];

        let upstream = vec![JsonRpcResult::Success(JsonRpcResponse { result: json!("0x456"), id: 2 })];

        let out = JsonRpcHandler::build_responses(&calls, &cached_vec, &upstream, vec![1]);

        assert!(matches!(&out[0], JsonRpcResult::Success(JsonRpcResponse { result, id }) if result == &json!("0x123") && *id == 1));
        assert!(matches!(&out[1], JsonRpcResult::Success(JsonRpcResponse { result, id }) if result == &json!("0x456") && *id == 2));
    }
}
