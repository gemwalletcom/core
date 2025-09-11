use crate::cache::CacheProvider;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::header::{self, HeaderName};
use hyper::service::Service;
use hyper::HeaderMap;

use futures::FutureExt;
use hyper::{body::Incoming as IncomingBody, Request, Response};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

type HttpClient = Client<HttpsConnector<HttpConnector>, Full<Bytes>>;

use crate::cache::{CachedResponse, RequestCache};
use crate::config::{Domain, Url};
use crate::metrics::Metrics;
use crate::request_parser::extract_rpc_method;
use crate::request_url::RequestUrl;
use gem_tracing::info_with_context;

#[derive(Debug, Clone)]
pub struct ProxyRequestService {
    pub domains: HashMap<String, NodeDomain>,
    pub domain_configs: HashMap<String, Domain>,
    pub metrics: Metrics,
    pub cache: RequestCache,
    pub client: HttpClient,
    pub keep_headers: Arc<[HeaderName]>,
}

#[derive(Debug, Clone)]
pub struct NodeDomain {
    pub url: Url,
}

impl ProxyRequestService {
    pub fn new(domains: HashMap<String, NodeDomain>, domain_configs: HashMap<String, Domain>, metrics: Metrics, cache: RequestCache) -> Self {
        let https = HttpsConnector::new();
        let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build(https);

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
}

impl Service<Request<IncomingBody>> for ProxyRequestService {
    type Response = Response<Full<Bytes>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        let headers = req.headers().clone();

        let host = headers.get("host").expect("invalid host").to_str().unwrap_or_default();

        let user_agent = headers.get("user-agent").and_then(|x| x.to_str().ok()).unwrap_or_default();

        match self.domains.get(host) {
            Some(domain) => {
                let url = domain.url.clone();
                let url = RequestUrl::from_uri(url.clone(), url.urls_override.clone().unwrap_or_default(), req.uri());

                self.metrics.add_proxy_request(host, user_agent);

                let metrics = self.metrics.clone();
                let host = host.to_string();
                let cache = self.cache.clone();
                let domain_config = self.domain_configs.get(&host).cloned();
                let method = req.method().clone();
                let path = req.uri().path().to_string();
                let path_with_query = req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or(req.uri().path()).to_string();
                let user_agent_str = user_agent.to_string();
                let client = self.client.clone();
                let keep_headers = self.keep_headers.clone();

                async move {
                    let now = Instant::now();

                    if let Some(domain_config) = domain_config {
                        let chain = domain_config.chain;
                        let (parts, incoming_body) = req.into_parts();
                        let body = incoming_body.collect().await?.to_bytes();

                        let cache_ttl = cache.should_cache(&chain, &path_with_query, method.as_str(), Some(&body));
                        let cache_key = if cache_ttl.is_some() {
                            Some(RequestCache::create_cache_key(&host, &path_with_query, method.as_str(), Some(&body)))
                        } else {
                            None
                        };

                        let rpc_method = if method == hyper::Method::POST { extract_rpc_method(&body) } else { None };
                        let method_label = rpc_method.as_deref().unwrap_or(&path);
                        metrics.add_proxy_request_by_method(host.as_str(), method_label);

                        if let Some(ref key) = cache_key {
                            if let Some(cached) = cache.get(&chain, key).await {
                                metrics.add_cache_hit(host.as_str(), method_label);

                                info_with_context(
                                    "Cache HIT",
                                    &[
                                        ("chain", chain.as_ref()),
                                        ("host", host.as_str()),
                                        ("method", method.as_str()),
                                        ("path", &path),
                                        ("rpc_method", method_label),
                                    ],
                                );

                                metrics.add_proxy_response(host.as_str(), method_label, url.uri.host().unwrap_or_default(), cached.status, 0);
                                return Self::cached_response(cached).await;
                            }
                        }

                        let context = vec![
                            ("host", host.as_str()),
                            ("method", method.as_str()),
                            ("uri", path.as_str()),
                            ("rpc_method", method_label),
                            ("user_agent", &user_agent_str),
                        ];
                        info_with_context("Incoming request", &context);

                        if cache_key.is_some() {
                            metrics.add_cache_miss(host.as_str(), method_label);
                        }

                        let new_req = Request::builder()
                            .method(parts.method)
                            .uri(parts.uri)
                            .body(Full::new(body.clone()))
                            .expect("failed to build request");
                        let new_req = {
                            let mut r = new_req;
                            *r.headers_mut() = parts.headers;
                            r
                        };

                        let response = Self::proxy_pass_get_data(new_req, url.clone(), &client, &keep_headers).await?;
                        let status = response.status().as_u16();

                        info_with_context(
                            "Proxy response",
                            &[
                                ("host", url.uri.host().unwrap_or_default()),
                                ("status", &response.status().to_string()),
                                ("latency", &format!("{}ms", now.elapsed().as_millis())),
                            ],
                        );

                        metrics.add_proxy_response(
                            host.as_str(),
                            method_label,
                            url.uri.host().unwrap_or_default(),
                            status,
                            now.elapsed().as_millis(),
                        );

                        let (processed_response, body_bytes) = Self::proxy_pass_response(response, &keep_headers).await?;

                        if status == 200 && cache_ttl.is_some() && cache_key.is_some() {
                            if let (Some(ttl), Some(key)) = (cache_ttl, cache_key) {
                                let content_type = processed_response
                                    .headers()
                                    .get(header::CONTENT_TYPE)
                                    .and_then(|v| v.to_str().ok())
                                    .map(|s| s.to_string());

                                let body_size = body_bytes.len();
                                let cached_resp = CachedResponse {
                                    body: body_bytes,
                                    status,
                                    content_type,
                                    ttl_seconds: ttl,
                                };

                                let ttl_str = ttl.to_string();
                                let size_str = body_size.to_string();

                                let context = vec![
                                    ("chain", chain.as_ref()),
                                    ("host", host.as_str()),
                                    ("method", method.as_str()),
                                    ("path", path.as_str()),
                                    ("rpc_method", method_label),
                                    ("ttl_seconds", &ttl_str),
                                    ("size_bytes", &size_str),
                                ];

                                info_with_context("Cache SET", &context);

                                cache.set(&chain, key, cached_resp, ttl).await;
                            }
                        }

                        Ok(processed_response)
                    } else {
                        info_with_context(
                            "Incoming request",
                            &[("host", &host), ("method", method.as_str()), ("uri", &path), ("user_agent", &user_agent_str)],
                        );

                        metrics.add_proxy_request_by_method(&host, &path);

                        let (parts, body) = req.into_parts();
                        let body_bytes = body.collect().await?.to_bytes();
                        let full_body_req = Request::from_parts(parts, Full::new(body_bytes));
                        let response = Self::proxy_pass_get_data(full_body_req, url.clone(), &client, &keep_headers).await?;

                        info_with_context(
                            "Proxy response",
                            &[
                                ("host", url.uri.host().unwrap_or_default()),
                                ("status", &response.status().to_string()),
                                ("latency", &format!("{}ms", now.elapsed().as_millis())),
                            ],
                        );

                        metrics.add_proxy_response(
                            host.as_str(),
                            url.uri.path(),
                            url.uri.host().unwrap_or_default(),
                            response.status().as_u16(),
                            now.elapsed().as_millis(),
                        );

                        Self::proxy_pass_response(response, &keep_headers).await.map(|(resp, _)| resp)
                    }
                }
                .boxed()
            }
            _ => async move { Ok(Response::builder().body(Full::new(Bytes::from("unsupported domain"))).unwrap()) }.boxed(),
        }
    }
}

impl ProxyRequestService {
    async fn proxy_pass_response(
        response: Response<IncomingBody>,
        keep_headers: &[HeaderName],
    ) -> Result<(Response<Full<Bytes>>, Bytes), Box<dyn std::error::Error + Send + Sync>> {
        let resp_headers = response.headers().clone();
        let status = response.status();
        let body = response.collect().await?.to_bytes();

        let mut new_response = Response::new(Full::from(body.clone()));
        *new_response.status_mut() = status;
        *new_response.headers_mut() = Self::persist_headers(&resp_headers, keep_headers);

        Ok((new_response, body))
    }

    async fn proxy_pass_get_data(
        original_request: Request<Full<Bytes>>,
        url: RequestUrl,
        client: &HttpClient,
        keep_headers: &[HeaderName],
    ) -> Result<Response<IncomingBody>, Box<dyn std::error::Error + Send + Sync>> {
        let original_headers = original_request.headers().clone();
        let mut request = Request::builder()
            .method(original_request.method())
            .uri(url.clone().uri)
            .body(original_request.into_body())
            .expect("invalid request params");

        let mut new_headers = Self::persist_headers(&original_headers, keep_headers);
        for (key, value) in url.params.clone() {
            new_headers.append(HeaderName::from_str(&key).unwrap(), value.clone().parse().unwrap());
        }
        *request.headers_mut() = new_headers;

        Ok(client.request(request).await?)
    }

    pub fn persist_headers(headers: &HeaderMap, list: &[HeaderName]) -> HeaderMap {
        headers
            .iter()
            .filter_map(|(k, v)| if list.contains(&k) { Some((k.clone(), v.clone())) } else { None })
            .collect()
    }

    async fn cached_response(cached: CachedResponse) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut response = Response::new(Full::from(cached.body));
        *response.status_mut() = hyper::StatusCode::from_u16(cached.status).unwrap_or(hyper::StatusCode::OK);
        if let Some(content_type) = cached.content_type {
            response.headers_mut().insert(header::CONTENT_TYPE, content_type.parse().unwrap());
        }
        Ok(response)
    }
}
