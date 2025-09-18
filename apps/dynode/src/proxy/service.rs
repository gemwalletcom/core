use crate::cache::{CacheProvider, CachedResponse, RequestCache};
use crate::config::{Domain, Url};
use crate::jsonrpc_types::{JsonRpcRequest, JsonRpcResponse, RequestType};
use crate::metrics::Metrics;
use crate::proxy::jsonrpc::JsonRpcHandler;
use crate::proxy::request_builder::RequestBuilder;
use crate::proxy::request_url::RequestUrl;
use crate::proxy::response_builder::{ProxyResponse, ResponseBuilder};
use gem_tracing::{info_with_fields, DurationMs};
use primitives::Chain;
use reqwest::header::{self, HeaderMap, HeaderName};
use reqwest::Method;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ProxyRequestService {
    pub domains: HashMap<String, NodeDomain>,
    pub domain_configs: HashMap<String, Domain>,
    pub metrics: Metrics,
    pub cache: RequestCache,
    pub client: reqwest::Client,
    pub keep_headers: Arc<[HeaderName]>,
}

#[derive(Debug, Clone)]
pub struct NodeDomain {
    pub url: Url,
}

impl ProxyRequestService {
    pub fn new(domains: HashMap<String, NodeDomain>, domain_configs: HashMap<String, Domain>, metrics: Metrics, cache: RequestCache) -> Self {
        let client = reqwest::Client::new();
        let keep_headers: Arc<[HeaderName]> = Arc::new([header::CONTENT_TYPE, header::CONTENT_ENCODING]);

        Self {
            domains,
            domain_configs,
            metrics,
            cache,
            client,
            keep_headers,
        }
    }

    pub async fn handle_request(
        &self,
        method: Method,
        headers: HeaderMap,
        body: Vec<u8>,
        path: String,
        path_with_query: String,
        host: String,
        user_agent: String,
    ) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let now = Instant::now();

        let metrics = self.metrics.clone();
        let cache = self.cache.clone();
        let client = self.client.clone();
        let keep_headers = self.keep_headers.clone();

        let (domain, domain_config) = match (self.domains.get(&host), self.domain_configs.get(&host)) {
            (Some(domain), Some(config)) => (domain.clone(), config.clone()),
            _ => {
                let response = ResponseBuilder::build_with_headers(b"domain not found".to_vec(), 404, "text/plain", HeaderMap::new())?;
                return Ok(response);
            }
        };

        let chain = domain_config.chain;
        let base_url = domain.url.clone();
        let overrides = base_url.urls_override.clone().unwrap_or_default();
        let request_url = RequestUrl::from_parts(base_url, overrides, &path_with_query);

        metrics.add_proxy_request(&host, &user_agent);

        let request_type = RequestType::from_request(method.as_str(), path_with_query.clone(), body.clone());

        match &request_type {
            RequestType::JsonRpc(_) => {
                info_with_fields!(
                    "Incoming request",
                    host = host.as_str(),
                    method = method.as_str(),
                    uri = path.as_str(),
                    rpc_method = &request_type.get_methods_list(),
                    user_agent = &user_agent,
                );
            }
            RequestType::Regular { .. } => {
                info_with_fields!(
                    "Incoming request",
                    host = host.as_str(),
                    method = method.as_str(),
                    uri = path.as_str(),
                    user_agent = &user_agent,
                );
            }
        }

        let cache_ttl = cache.should_cache_request(&chain, &request_type);
        let cache_key = if cache_ttl.is_some() {
            Some(request_type.cache_key(&host, &path_with_query))
        } else {
            None
        };

        let methods_for_metrics = request_type.get_methods_for_metrics();
        for method_name in &methods_for_metrics {
            metrics.add_proxy_request_by_method(&host, method_name);
        }

        if let Some(key) = cache_key.as_ref() {
            if let Some(result) = Self::try_cache_hit(&cache, chain, key, &request_type, &host, &path_with_query, &request_url, &metrics, now).await {
                return result;
            }
        }

        if let RequestType::JsonRpc(rpc_request) = &request_type {
            return JsonRpcHandler::handle_request(
                rpc_request,
                chain,
                &host,
                &path_with_query,
                &cache,
                &metrics,
                &request_url,
                &client,
                &method,
                now,
            )
            .await;
        }

        let response = Self::proxy_pass_get_data(method.clone(), headers, body, request_url.clone(), &client, &keep_headers).await?;
        let status = response.status().as_u16();

        let upstream_headers = ResponseBuilder::create_upstream_headers(request_url.url.host_str(), now.elapsed());
        let (processed_response, body_bytes) = Self::proxy_pass_response(response, &keep_headers, upstream_headers).await?;

        for method_name in &methods_for_metrics {
            metrics.add_proxy_response(
                &host,
                &path_with_query,
                method_name,
                request_url.url.host_str().unwrap_or_default(),
                status,
                now.elapsed().as_millis(),
            );
        }

        info_with_fields!(
            "Proxy response",
            host = request_url.url.host_str().unwrap_or_default(),
            status = status,
            latency = DurationMs(now.elapsed()),
        );

        if status == 200 {
            if let (Some(ttl), Some(key)) = (cache_ttl, cache_key.clone()) {
                tokio::spawn(Self::store_cache(
                    status,
                    ttl,
                    key,
                    body_bytes,
                    request_type.clone(),
                    chain,
                    host.clone(),
                    method.clone(),
                    path.clone(),
                    cache.clone(),
                    now,
                ));
            }
        }

        Ok(processed_response)
    }

    async fn try_cache_hit(
        cache: &RequestCache,
        chain: Chain,
        cache_key: &str,
        request_type: &RequestType,
        host: &str,
        path: &str,
        url: &RequestUrl,
        metrics: &Metrics,
        now: std::time::Instant,
    ) -> Option<Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>>> {
        if let Some(cached) = cache.get(&chain, cache_key).await {
            let methods_for_metrics = request_type.get_methods_for_metrics();
            for method_name in &methods_for_metrics {
                metrics.add_cache_hit(host, method_name);
            }

            info_with_fields!("Cache HIT", chain = chain.as_ref(), host = host, method = &methods_for_metrics.join(","));

            let upstream_headers = ResponseBuilder::create_upstream_headers(url.url.host_str(), now.elapsed());
            let status = cached.status;

            let response = match request_type {
                RequestType::JsonRpc(JsonRpcRequest::Single(original_call)) => {
                    let data = cached.to_jsonrpc_response(original_call);
                    ResponseBuilder::build_with_headers(data, cached.status, &cached.content_type, upstream_headers)
                }
                RequestType::Regular { .. } => Ok(ResponseBuilder::build_cached_with_headers(cached.clone(), upstream_headers)),
                RequestType::JsonRpc(JsonRpcRequest::Batch(_)) => return None,
            };

            for method_name in &methods_for_metrics {
                metrics.add_proxy_response(
                    host,
                    path,
                    method_name,
                    url.url.host_str().unwrap_or_default(),
                    status,
                    now.elapsed().as_millis(),
                );
            }

            Some(response)
        } else {
            let methods_for_metrics = request_type.get_methods_for_metrics();
            for method_name in &methods_for_metrics {
                metrics.add_cache_miss(host, method_name);
            }
            None
        }
    }

    async fn store_cache(
        status: u16,
        cache_ttl: u64,
        cache_key: String,
        body_bytes: Vec<u8>,
        request_type: RequestType,
        chain: Chain,
        host: String,
        method: Method,
        path: String,
        cache: RequestCache,
        request_start: Instant,
    ) {
        let content_type = request_type.content_type().to_string();
        let body_size = body_bytes.len();

        let cached = match request_type {
            RequestType::JsonRpc(_) => {
                let json_response = serde_json::from_slice::<JsonRpcResponse>(&body_bytes).expect("JSON-RPC response must be valid JSON");
                let result_bytes = serde_json::to_string(&json_response.result).unwrap_or_default().into_bytes();
                CachedResponse::new(result_bytes, status, content_type.clone(), cache_ttl)
            }
            RequestType::Regular { .. } => CachedResponse::new(body_bytes, status, content_type, cache_ttl),
        };

        cache.set(&chain, cache_key, cached, cache_ttl).await;

        info_with_fields!(
            "Cache SET",
            chain = chain.as_ref(),
            host = &host,
            method = method.as_str(),
            path = &path,
            ttl_seconds = cache_ttl,
            size_bytes = body_size,
            latency = DurationMs(request_start.elapsed()),
        );
    }

    async fn proxy_pass_response(
        response: reqwest::Response,
        keep_headers: &[HeaderName],
        additional_headers: HeaderMap,
    ) -> Result<(ProxyResponse, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
        let resp_headers = response.headers().clone();
        let status = response.status().as_u16();
        let body = response.bytes().await?.to_vec();

        let mut headers = Self::persist_headers(&resp_headers, keep_headers);
        headers.extend(additional_headers);

        Ok((ProxyResponse::new(status, headers, body.clone()), body))
    }

    async fn proxy_pass_get_data(
        method: Method,
        original_headers: HeaderMap,
        body: Vec<u8>,
        url: RequestUrl,
        client: &reqwest::Client,
        keep_headers: &[HeaderName],
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        let request = RequestBuilder::build_forwarded(&method, &url, body, &original_headers, keep_headers)?;
        Ok(client.execute(request).await?)
    }

    pub fn persist_headers(headers: &HeaderMap, list: &[HeaderName]) -> HeaderMap {
        RequestBuilder::filter_headers(headers, list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header;

    #[test]
    fn test_persist_headers_filters_correctly() {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
        headers.insert("x-keep", header::HeaderValue::from_static("1"));
        headers.insert("x-drop", header::HeaderValue::from_static("0"));

        let keep = [header::CONTENT_TYPE, HeaderName::from_static("x-keep")];
        let filtered = ProxyRequestService::persist_headers(&headers, &keep);

        assert!(filtered.get("x-drop").is_none());
        assert_eq!(filtered.get("x-keep").unwrap(), &header::HeaderValue::from_static("1"));
        assert_eq!(
            filtered.get(header::CONTENT_TYPE).unwrap(),
            &header::HeaderValue::from_static("application/json")
        );
    }
}
