use crate::cache::{CacheProvider, CachedResponse, RequestCache};
use crate::config::{Domain, Url};
use crate::jsonrpc_types::{JsonRpcRequest, JsonRpcResponse, RequestType};
use crate::metrics::Metrics;
use crate::proxy::jsonrpc::JsonRpcHandler;
use crate::proxy::proxy_request::ProxyRequest;
use crate::proxy::request_builder::RequestBuilder;
use crate::proxy::request_url::RequestUrl;
use crate::proxy::response_builder::{ProxyResponse, ResponseBuilder};
use gem_tracing::{DurationMs, info_with_fields};
use reqwest::Method;
use reqwest::StatusCode;
use reqwest::header::{self, HeaderMap, HeaderName};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ProxyRequestService {
    pub domains: Arc<RwLock<HashMap<String, NodeDomain>>>,
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
    pub fn new(
        domains: Arc<RwLock<HashMap<String, NodeDomain>>>,
        domain_configs: HashMap<String, Domain>,
        metrics: Metrics,
        cache: RequestCache,
        client: reqwest::Client,
    ) -> Self {
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

    pub async fn handle_request(&self, request: ProxyRequest, node_domain: &NodeDomain) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let chain = request.chain;
        let url = RequestUrl::from_parts(node_domain.url.clone(), &request.path_with_query);
        let request_type = request.request_type();

        self.metrics.add_proxy_request(&request.host, &request.user_agent);

        match &request_type {
            RequestType::JsonRpc(_) => {
                info_with_fields!(
                    "Incoming request",
                    host = request.host.as_str(),
                    method = request.method.as_str(),
                    uri = request.path.as_str(),
                    rpc_method = &request_type.get_methods_list(),
                    user_agent = &request.user_agent,
                );
            }
            RequestType::Regular { .. } => {
                info_with_fields!(
                    "Incoming request",
                    host = request.host.as_str(),
                    method = request.method.as_str(),
                    uri = request.path.as_str(),
                    user_agent = &request.user_agent,
                );
            }
        }

        let cache_ttl = self.cache.should_cache_request(&chain, &request_type);
        let cache_key = cache_ttl.map(|_| request_type.cache_key(&request.host, &request.path_with_query));

        let methods_for_metrics = request_type.get_methods_for_metrics();
        self.metrics.add_proxy_request_batch(&request.host, &request.user_agent, &methods_for_metrics);

        if let Some(key) = &cache_key
            && let Some(result) = Self::try_cache_hit(&self.cache, key, &request, &url, &self.metrics).await
        {
            return result;
        }

        if let RequestType::JsonRpc(rpc_request) = &request_type {
            return JsonRpcHandler::handle_request(rpc_request, &request, &self.cache, &self.metrics, &url, &self.client).await;
        }

        let response = Self::proxy_pass_get_data(
            request.method.clone(),
            request.headers.clone(),
            request.body.clone(),
            url.clone(),
            &self.client,
            &self.keep_headers,
        )
        .await?;
        let status = response.status().as_u16();

        let upstream_headers = ResponseBuilder::create_upstream_headers(url.url.host_str(), request.elapsed());
        let (processed_response, body_bytes) = Self::proxy_pass_response(response, &self.keep_headers, upstream_headers).await?;

        for method_name in &methods_for_metrics {
            self.metrics.add_proxy_response(
                &request.host,
                &request.path_with_query,
                method_name,
                url.url.host_str().unwrap_or_default(),
                status,
                request.elapsed().as_millis(),
            );
        }

        info_with_fields!(
            "Proxy response",
            host = request.host,
            remote_host = url.url.host_str().unwrap_or_default(),
            status = status,
            latency = DurationMs(request.elapsed()),
        );

        if status == StatusCode::OK.as_u16()
            && let (Some(ttl), Some(key)) = (cache_ttl, cache_key)
        {
            let request_clone = request.clone();
            let cache_clone = self.cache.clone();
            tokio::spawn(async move {
                if let Err(err) = Self::store_cache(status, ttl, key, body_bytes, request_clone, cache_clone).await {
                    // Log cache storage error but don't fail the request
                    gem_tracing::error_with_fields!("Failed to store cache", err.as_ref(),);
                }
            });
        }

        Ok(processed_response)
    }

    async fn try_cache_hit(
        cache: &RequestCache,
        cache_key: &str,
        request: &ProxyRequest,
        url: &RequestUrl,
        metrics: &Metrics,
    ) -> Option<Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>>> {
        if let Some(cached) = cache.get(&request.chain, cache_key).await {
            let request_type = request.request_type();
            let methods_for_metrics = request_type.get_methods_for_metrics();
            for method_name in &methods_for_metrics {
                metrics.add_cache_hit(&request.host, method_name);
            }

            info_with_fields!(
                "Cache HIT",
                chain = request.chain.as_ref(),
                host = &request.host,
                method = &methods_for_metrics.join(",")
            );

            let upstream_headers = ResponseBuilder::create_upstream_headers(url.url.host_str(), request.elapsed());
            let status = cached.status;

            let response = match request_type {
                RequestType::JsonRpc(JsonRpcRequest::Single(original_call)) => {
                    let data = cached.to_jsonrpc_response(&original_call);
                    ResponseBuilder::build_with_headers(data, cached.status, &cached.content_type, upstream_headers)
                }
                RequestType::Regular { .. } => Ok(ResponseBuilder::build_cached_with_headers(cached.clone(), upstream_headers)),
                RequestType::JsonRpc(JsonRpcRequest::Batch(_)) => return None,
            };

            for method_name in &methods_for_metrics {
                metrics.add_proxy_response(
                    &request.host,
                    &request.path_with_query,
                    method_name,
                    url.url.host_str().unwrap_or_default(),
                    status,
                    request.elapsed().as_millis(),
                );
            }

            Some(response)
        } else {
            let request_type = request.request_type();
            let methods_for_metrics = request_type.get_methods_for_metrics();
            for method_name in &methods_for_metrics {
                metrics.add_cache_miss(&request.host, method_name);
            }
            None
        }
    }

    async fn store_cache(
        status: u16,
        cache_ttl: u64,
        cache_key: String,
        body_bytes: Vec<u8>,
        request: ProxyRequest,
        cache: RequestCache,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request_type = request.request_type();
        let content_type = request_type.content_type().to_string();
        let body_size = body_bytes.len();

        let cached = match request_type {
            RequestType::JsonRpc(_) => {
                let json_response = serde_json::from_slice::<JsonRpcResponse>(&body_bytes)?;
                let result_bytes = serde_json::to_string(&json_response.result).unwrap_or_default().into_bytes();
                CachedResponse::new(result_bytes, status, content_type.clone(), cache_ttl)
            }
            RequestType::Regular { .. } => CachedResponse::new(body_bytes, status, content_type, cache_ttl),
        };

        cache.set(&request.chain, cache_key, cached, cache_ttl).await;

        info_with_fields!(
            "Cache SET",
            chain = request.chain.as_ref(),
            host = &request.host,
            method = request.method.as_str(),
            path = &request.path,
            ttl_seconds = cache_ttl,
            size_bytes = body_size,
            latency = DurationMs(request.elapsed()),
        );

        Ok(())
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
