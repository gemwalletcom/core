use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;

use dynode::config::load_config;
use dynode::metrics::Metrics;
use dynode::monitoring::{NodeMonitor, NodeService};
use dynode::proxy::{ProxyRequestBuilder, ProxyResponse};
use primitives::Chain;
use dynode::response::{ErrorResponse, ProxyRocketResponse};
use gem_tracing::{error_with_fields, info_with_fields};
use reqwest::Method;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use rocket::config::Config;
use rocket::data::{Data, ToByteUnit};
use rocket::http::{Method as RocketMethod, Status};
use rocket::outcome::Outcome as RequestOutcome;
use rocket::response::content::RawText;
use rocket::route::{Handler, Outcome, Route};
use rocket::tokio::io::AsyncReadExt;
use rocket::{Request, State};

const BODY_READ_LIMIT_MB: u64 = 32;

#[derive(Clone)]
struct ProxyHandler;

#[rocket::async_trait]
impl Handler for ProxyHandler {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        let state_outcome = request.guard::<&State<NodeService>>().await;
        let node_service = match state_outcome {
            RequestOutcome::Success(state) => state,
            RequestOutcome::Error((status, _)) | RequestOutcome::Forward(status) => {
                return Outcome::from(request, ErrorResponse::new(status, "Failed to access node service".to_string()));
            }
        };

        let method = match Method::from_bytes(request.method().as_str().as_bytes()) {
            Ok(method) => method,
            Err(_) => return Outcome::from(request, ErrorResponse::new(Status::BadRequest, "Invalid HTTP method".to_string())),
        };

        let uri = request.uri().to_string();
        let chain = match resolve_chain(&uri) {
            Some(chain) => chain,
            None => return Outcome::from(request, ErrorResponse::new(Status::BadRequest, "Invalid chain".to_string())),
        };

        match process_proxy(chain, method, request, data, node_service.inner()).await {
            Ok(response) => Outcome::from(request, ProxyRocketResponse(response)),
            Err(err) => Outcome::from(request, err),
        }
    }
}

fn proxy_routes() -> Vec<Route> {
    let methods = [
        RocketMethod::Get,
        RocketMethod::Post,
        RocketMethod::Put,
        RocketMethod::Patch,
        RocketMethod::Delete,
        RocketMethod::Options,
        RocketMethod::Head,
    ];

    methods.into_iter().map(|method| Route::new(method, "/<path..>", ProxyHandler)).collect()
}

#[rocket::get("/metrics")]
async fn metrics_endpoint(metrics: &State<Metrics>) -> RawText<String> {
    RawText(metrics.get_metrics())
}

#[rocket::get("/health")]
async fn health_endpoint() -> Status {
    Status::Ok
}

#[rocket::get("/")]
async fn root_endpoint() -> Status {
    Status::Ok
}

async fn process_proxy(chain: Chain, method: Method, request: &Request<'_>, data: Data<'_>, node_service: &NodeService) -> Result<ProxyResponse, ErrorResponse> {
    let body = read_request_body(data).await?;
    let headers = build_header_map(request)?;
    let uri = request.uri().to_string();

    let proxy_request = match ProxyRequestBuilder::build(method.clone(), headers, body, uri, chain) {
        Ok(req) => req,
        Err(status) => {
            let msg = "Failed to build request".to_string();
            return Err(ErrorResponse::new(status, msg));
        }
    };

    node_service.handle_request(proxy_request).await.map_err(|err| {
        let error_msg = err.to_string();
        error_with_fields!("Proxy request failed", err.as_ref(),);
        ErrorResponse::new(Status::InternalServerError, error_msg)
    })
}

async fn read_request_body(data: Data<'_>) -> Result<Vec<u8>, ErrorResponse> {
    let limit = BODY_READ_LIMIT_MB.mebibytes();
    let mut stream = data.open(limit);
    let mut body = Vec::new();
    stream
        .read_to_end(&mut body)
        .await
        .map_err(|_| ErrorResponse::new(Status::InternalServerError, "Failed to read request body".to_string()))?;
    Ok(body)
}

fn build_header_map(request: &Request<'_>) -> Result<HeaderMap, ErrorResponse> {
    let mut headers = HeaderMap::new();
    for header in request.headers().iter() {
        let name = HeaderName::from_bytes(header.name().as_str().as_bytes())
            .map_err(|_| ErrorResponse::new(Status::BadRequest, format!("Invalid header name: {}", header.name())))?;
        let value =
            HeaderValue::from_str(header.value()).map_err(|_| ErrorResponse::new(Status::BadRequest, format!("Invalid header value for {}", header.name())))?;
        headers.append(name, value);
    }
    Ok(headers)
}

fn resolve_chain(path: &str) -> Option<Chain> {
    let chain_str = path.trim_start_matches('/').split('/').next()?;
    Chain::from_str(chain_str).ok()
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (config, chains) = load_config()?;

    let node_address = IpAddr::from_str(config.address.as_str())?;
    let metrics_address = IpAddr::from_str(config.metrics.address.as_str())?;

    let metrics_config = dynode::config::MetricsConfig {
        prefix: config.metrics.prefix.clone(),
        user_agent_patterns: config.metrics.user_agent_patterns.clone(),
    };
    let metrics = Metrics::new(metrics_config);
    let node_service = NodeService::new(
        chains,
        metrics.clone(),
        config.cache.clone(),
        config.monitoring.clone(),
        config.retry.clone(),
        config.request.clone(),
    );
    if node_service.monitoring_config.enabled {
        let monitor = NodeMonitor::new(
            node_service.chains.clone(),
            Arc::clone(&node_service.nodes),
            Arc::clone(&node_service.metrics),
            node_service.monitoring_config.clone(),
        );

        rocket::tokio::spawn(async move {
            monitor.start_monitoring().await;
        });
    }

    info_with_fields!(
        "Server started",
        node_address = &format!("{}:{}", node_address, config.port),
        metrics_address = &format!("{}:{}", metrics_address, config.metrics.port),
    );

    let proxy_server = rocket::custom(Config::figment().merge(("address", node_address)).merge(("port", config.port)))
        .manage(node_service)
        .mount("/", proxy_routes())
        .mount("/", rocket::routes![health_endpoint, root_endpoint]);

    let metrics_server = rocket::custom(Config::figment().merge(("address", metrics_address)).merge(("port", config.metrics.port)))
        .manage(metrics)
        .mount("/", rocket::routes![metrics_endpoint]);

    rocket::tokio::try_join!(proxy_server.launch(), metrics_server.launch())?;

    Ok(())
}
