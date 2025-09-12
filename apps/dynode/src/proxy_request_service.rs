use crate::cache::CacheProvider;
use crate::request_types::{JsonRpcRequest, JsonRpcResponse, RequestType};
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
use std::fmt::format;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

type HttpClient = Client<HttpsConnector<HttpConnector>, Full<Bytes>>;

use crate::cache::{CachedResponse, RequestCache};
use crate::config::{Domain, Url};
use crate::metrics::Metrics;
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
        let now = Instant::now();
        let headers = req.headers().clone();

        let host = headers.get("host").expect("invalid host").to_str().unwrap_or_default();

        let user_agent = headers.get("user-agent").and_then(|x| x.to_str().ok()).unwrap_or_default();

        let (domain, domain_config) = if let (Some(domain), Some(domain_config)) = (self.domains.get(host), self.domain_configs.get(host)) {
            (domain, domain_config.clone())
        } else {
            return async move { Ok(Response::builder().status(404).body(Full::new(Bytes::from("domain not found"))).unwrap()) }.boxed();
        };

        let url = domain.url.clone();
        let url = RequestUrl::from_uri(url.clone(), url.urls_override.clone().unwrap_or_default(), req.uri());

        self.metrics.add_proxy_request(host, user_agent);

        let metrics = self.metrics.clone();
        let host = host.to_string();
        let cache = self.cache.clone();
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let path_with_query = req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or(req.uri().path()).to_string();
        let user_agent_str = user_agent.to_string();
        let client = self.client.clone();
        let keep_headers = self.keep_headers.clone();

        async move {
            let chain = domain_config.chain;
            let (parts, incoming_body) = req.into_parts();
            let body = incoming_body.collect().await?.to_bytes();

            let request_type = RequestType::from_request(method.as_str(), path_with_query.clone(), body.clone());

            let cache_ttl = cache.should_cache_request(&chain, &request_type);

            let cache_key = if cache_ttl.is_some() {
                Some(request_type.cache_key(&host, &path_with_query))
            } else {
                None
            };

            let methods_for_metrics = request_type.get_methods_for_metrics();
            let methods_display = request_type.get_methods_list();

            for method in &methods_for_metrics {
                metrics.add_proxy_request_by_method(host.as_str(), method);
            }

            if let Some(ref key) = cache_key {
                if let Some(cached) = cache.get(&chain, key).await {
                    for method in &methods_for_metrics {
                        metrics.add_cache_hit(host.as_str(), method);
                    }

                    let response = match &request_type {
                        RequestType::JsonRpc(JsonRpcRequest::Single(original_call)) => {
                            let data = cached.to_jsonrpc_response(original_call);
                            let mut response = Response::new(Full::new(data));
                            *response.status_mut() = hyper::StatusCode::from_u16(cached.status).unwrap_or(hyper::StatusCode::OK);

                            response
                                .headers_mut()
                                .insert(header::CONTENT_TYPE, Self::get_content_type_header(&cached.content_type));

                            Ok(response)
                        }
                        RequestType::Regular { .. } => Ok(Self::cached_response_sync(cached.clone())),
                    };

                    let latency_ms = now.elapsed().as_millis();
                    let latency_str;
                    let latency_ref = if latency_ms < 10 {
                        // Use static strings for common small values
                        match latency_ms {
                            0 => "0ms",
                            1 => "1ms",
                            2 => "2ms",
                            3 => "3ms",
                            4 => "4ms",
                            5 => "5ms",
                            6 => "6ms",
                            7 => "7ms",
                            8 => "8ms",
                            9 => "9ms",
                            _ => {
                                latency_str = format!("{}ms", latency_ms);
                                &latency_str
                            }
                        }
                    } else {
                        latency_str = format!("{}ms", latency_ms);
                        &latency_str
                    };

                    info_with_context(
                        "Cache HIT",
                        &[
                            ("chain", chain.as_ref()),
                            ("host", host.as_str()),
                            ("method", method.as_str()),
                            ("path", &path),
                            ("rpc_method", &methods_display),
                            ("latency", latency_ref),
                        ],
                    );

                    for method in &methods_for_metrics {
                        metrics.add_proxy_response(
                            host.as_str(),
                            method,
                            url.uri.host().unwrap_or_default(),
                            cached.status,
                            0, // Cache hit latency is essentially 0
                        );
                    }

                    return response;
                }
            }

            let context = vec![
                ("host", host.as_str()),
                ("method", method.as_str()),
                ("uri", path.as_str()),
                ("rpc_method", &methods_display),
                ("user_agent", &user_agent_str),
            ];
            info_with_context("Incoming request", &context);

            if cache_key.is_some() {
                for method in &methods_for_metrics {
                    metrics.add_cache_miss(host.as_str(), method);
                }
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

            for method in &methods_for_metrics {
                metrics.add_proxy_response(host.as_str(), method, url.uri.host().unwrap_or_default(), status, now.elapsed().as_millis());
            }

            let (processed_response, body_bytes) = Self::proxy_pass_response(response, &keep_headers).await?;

            if status == 200 {
                if let (Some(ttl), Some(key)) = (cache_ttl, cache_key) {
                    tokio::spawn(Self::store_cache(
                        status,
                        ttl,
                        key,
                        body_bytes.clone(),
                        request_type.clone(),
                        chain,
                        host.clone(),
                        method.clone(),
                        path.clone(),
                        cache.clone(),
                    ));
                }
            }

            Ok(processed_response)
        }
        .boxed()
    }
}

impl ProxyRequestService {
    fn get_content_type_header(content_type_str: &str) -> header::HeaderValue {
        content_type_str.parse().unwrap_or_else(|_| "application/json".parse().unwrap())
    }

    async fn store_cache(
        status: u16,
        cache_ttl: u64,
        cache_key: String,
        body_bytes: Bytes,
        request_type: RequestType,
        chain: primitives::Chain,
        host: String,
        method: hyper::Method,
        path: String,
        cache: RequestCache,
    ) {
        let now = Instant::now();
        let content_type = request_type.content_type().to_string();

        let body_size = body_bytes.len();

        let cached = match &request_type {
            RequestType::JsonRpc(_) => {
                let json_response = serde_json::from_slice::<JsonRpcResponse>(&body_bytes).expect("JSON-RPC response must be valid JSON");
                let result_bytes = Bytes::from(serde_json::to_string(&json_response.result).unwrap_or_default());
                CachedResponse::new(result_bytes, status, content_type, cache_ttl)
            }
            RequestType::Regular { .. } => CachedResponse::new(body_bytes.clone(), status, content_type, cache_ttl),
        };

        let ttl_str = cache_ttl.to_string();
        let size_str = body_size.to_string();
        let methods_display = request_type.get_methods_list();
        let latency_str = format!("{}ms", now.elapsed().as_millis());
        cache.set(&chain, cache_key, cached, cache_ttl).await;

        let context = vec![
            ("chain", chain.as_ref()),
            ("host", &host),
            ("method", method.as_str()),
            ("path", &path),
            ("rpc_method", &methods_display),
            ("ttl_seconds", &ttl_str),
            ("size_bytes", &size_str),
            ("latency_ms", &latency_str),
        ];

        info_with_context("Cache SET", &context);
    }

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

    fn cached_response_sync(cached: CachedResponse) -> Response<Full<Bytes>> {
        let mut response = Response::new(Full::from(cached.body));

        // Most cached responses are 200, avoid parsing if possible
        if cached.status == 200 {
            *response.status_mut() = hyper::StatusCode::OK;
        } else {
            *response.status_mut() = hyper::StatusCode::from_u16(cached.status).unwrap_or(hyper::StatusCode::OK);
        }

        // Most responses are application/json, avoid parsing if possible
        if cached.content_type == "application/json" {
            response
                .headers_mut()
                .insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
        } else {
            response
                .headers_mut()
                .insert(header::CONTENT_TYPE, Self::get_content_type_header(&cached.content_type));
        }

        response
    }
}
