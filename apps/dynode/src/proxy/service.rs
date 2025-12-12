use crate::cache::{CacheProvider, CachedResponse, RequestCache};
use crate::config::{ChainConfig, HeadersConfig, Url};
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
use reqwest::header::{HeaderMap, HeaderName};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ProxyRequestService {
    pub metrics: Metrics,
    pub cache: RequestCache,
    pub client: reqwest::Client,
    pub forward_headers: Arc<[HeaderName]>,
    pub headers_config: HeadersConfig,
}

#[derive(Debug, Clone)]
pub struct NodeDomain {
    pub url: Url,
    pub config: ChainConfig,
}

impl NodeDomain {
    pub fn new(url: Url, config: ChainConfig) -> Self {
        Self { url, config }
    }
}

impl ProxyRequestService {
    pub fn new(metrics: Metrics, cache: RequestCache, client: reqwest::Client, headers_config: HeadersConfig) -> Self {
        let forward_headers: Arc<[HeaderName]> = headers_config
            .forward
            .iter()
            .filter_map(|s| HeaderName::from_str(s).ok())
            .collect::<Vec<_>>()
            .into();

        Self {
            metrics,
            cache,
            client,
            forward_headers,
            headers_config,
        }
    }

    fn build_headers(&self, host: &str, original: &HeaderMap) -> HeaderMap {
        let mut headers = RequestBuilder::filter_headers(original, &self.forward_headers);

        if let Some(names) = self.headers_config.get_domain_headers(host) {
            for name in names {
                if let Ok(key) = HeaderName::from_str(name)
                    && let Some(value) = original.get(&key)
                {
                    headers.insert(key, value.clone());
                }
            }
        }

        headers
    }

    pub async fn handle_request(&self, request: ProxyRequest, node_domain: &NodeDomain) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let chain = request.chain;
        let request_type = request.request_type();

        let rpc_method = match &request_type {
            RequestType::JsonRpc(JsonRpcRequest::Single(call)) => Some(call.method.as_str()),
            _ => None,
        };

        let resolved_url = node_domain.config.resolve_url(&node_domain.url, rpc_method, Some(&request.path));
        let url = RequestUrl::from_parts(resolved_url, &request.path_with_query);
        let headers = self.build_headers(url.url.host_str().unwrap_or_default(), &request.headers);

        self.metrics.add_proxy_request(request.chain.as_ref(), &request.user_agent);

        match &request_type {
            RequestType::JsonRpc(_) => {
                info_with_fields!(
                    "Incoming request",
                    chain = request.chain.as_ref(),
                    method = request.method.as_str(),
                    uri = request.path.as_str(),
                    rpc_method = &request_type.get_methods_list(),
                    user_agent = &request.user_agent,
                );
            }
            RequestType::Regular { .. } => {
                info_with_fields!(
                    "Incoming request",
                    chain = request.chain.as_ref(),
                    method = request.method.as_str(),
                    uri = request.path.as_str(),
                    user_agent = &request.user_agent,
                );
            }
        }

        let cache_ttl = self.cache.should_cache_request(&chain, &request_type);
        let cache_key = cache_ttl.and_then(|_| request_type.cache_key(&request.host, &request.path_with_query));

        let methods_for_metrics = request_type.get_methods_for_metrics();
        self.metrics
            .add_proxy_request_batch(request.chain.as_ref(), &request.user_agent, &methods_for_metrics);

        if let Some(key) = &cache_key
            && let Some(result) = Self::try_cache_hit(&self.cache, key, &request, &url, &self.metrics).await
        {
            return result;
        }

        if let RequestType::JsonRpc(rpc_request) = &request_type {
            return JsonRpcHandler::handle_request(rpc_request, &request, &self.cache, &self.metrics, &url, &self.client, &headers).await;
        }

        let response = Self::proxy_pass_get_data(request.method.clone(), request.body.clone(), url.clone(), &self.client, headers).await?;
        let status = response.status().as_u16();

        let upstream_headers = ResponseBuilder::create_upstream_headers(url.url.host_str(), request.elapsed());
        let (processed_response, body_bytes) = Self::proxy_pass_response(response, &self.forward_headers, upstream_headers).await?;

        for method_name in &methods_for_metrics {
            self.metrics.add_proxy_response(
                request.chain.as_ref(),
                &request.path_with_query,
                method_name,
                url.url.host_str().unwrap_or_default(),
                status,
                request.elapsed().as_millis(),
            );
        }

        info_with_fields!(
            "Proxy response",
            chain = request.chain.as_ref(),
            remote_host = url.url.host_str().unwrap_or_default(),
            method = request.method.as_str(),
            uri = request.path.as_str(),
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
                metrics.add_cache_hit(request.chain.as_ref(), method_name);
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
                    request.chain.as_ref(),
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
                metrics.add_cache_miss(request.chain.as_ref(), method_name);
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
        forward_headers: &[HeaderName],
        additional_headers: HeaderMap,
    ) -> Result<(ProxyResponse, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
        let resp_headers = response.headers().clone();
        let status = response.status().as_u16();
        let body = response.bytes().await?.to_vec();

        let mut headers = RequestBuilder::filter_headers(&resp_headers, forward_headers);
        headers.extend(additional_headers);

        Ok((ProxyResponse::new(status, headers, body.clone()), body))
    }

    async fn proxy_pass_get_data(
        method: Method,
        body: Vec<u8>,
        url: RequestUrl,
        client: &reqwest::Client,
        headers: HeaderMap,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        let request = RequestBuilder::build(&method, &url, body, headers)?;
        Ok(client.execute(request).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::RequestCache;
    use crate::config::{CacheConfig, HeadersConfig, MetricsConfig};
    use crate::metrics::Metrics;
    use crate::proxy::constants::JSON_CONTENT_TYPE;
    use reqwest::header;
    use std::collections::HashMap;

    fn create_service(headers_config: HeadersConfig) -> ProxyRequestService {
        ProxyRequestService::new(
            Metrics::new(MetricsConfig::default()),
            RequestCache::new(CacheConfig::default()),
            reqwest::Client::new(),
            headers_config,
        )
    }

    #[test]
    fn test_build_headers_with_domain_config() {
        let mut domains = HashMap::new();
        domains.insert("example.com".to_string(), vec![header::USER_AGENT.to_string()]);

        let service = create_service(HeadersConfig {
            forward: vec![header::CONTENT_TYPE.to_string()],
            domains,
        });

        let mut original = HeaderMap::new();
        original.insert(header::CONTENT_TYPE, header::HeaderValue::from_static(JSON_CONTENT_TYPE));
        original.insert(header::USER_AGENT, header::HeaderValue::from_static("TestAgent/1.0"));
        original.insert("x-drop", header::HeaderValue::from_static("dropped"));

        let headers = service.build_headers("example.com", &original);

        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), JSON_CONTENT_TYPE);
        assert_eq!(headers.get(header::USER_AGENT).unwrap(), "TestAgent/1.0");
        assert!(headers.get("x-drop").is_none());
    }

    #[test]
    fn test_build_headers_without_domain_config() {
        let service = create_service(HeadersConfig {
            forward: vec![header::CONTENT_TYPE.to_string()],
            domains: HashMap::new(),
        });

        let mut original = HeaderMap::new();
        original.insert(header::CONTENT_TYPE, header::HeaderValue::from_static(JSON_CONTENT_TYPE));
        original.insert(header::USER_AGENT, header::HeaderValue::from_static("TestAgent/1.0"));

        let headers = service.build_headers("example.com", &original);

        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), JSON_CONTENT_TYPE);
        assert!(headers.get(header::USER_AGENT).is_none());
    }
}
