use std::io::Cursor;
use std::net::IpAddr;
use std::str::FromStr;

use dynode::config::NodeConfig;
use dynode::metrics::Metrics;
use dynode::monitoring::NodeService;
use dynode::proxy::{ProxyRequestBuilder, ProxyResponse};
use gem_tracing::{error_with_fields, info_with_fields};
use reqwest::Method;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use rocket::config::Config;
use rocket::data::{Data, ToByteUnit};
use rocket::http::{Method as RocketMethod, Status};
use rocket::outcome::Outcome as RequestOutcome;
use rocket::response::{Responder, Response, content::RawText};
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
            RequestOutcome::Error((status, _)) | RequestOutcome::Forward(status) => return Outcome::error(status),
        };

        let method = match Method::from_bytes(request.method().as_str().as_bytes()) {
            Ok(method) => method,
            Err(_) => return Outcome::error(Status::BadRequest),
        };

        match process_proxy(method, request, data, node_service.inner()).await {
            Ok(response) => Outcome::from(request, ProxyRocketResponse(response)),
            Err(status) => Outcome::error(status),
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

async fn process_proxy(method: Method, request: &Request<'_>, data: Data<'_>, node_service: &NodeService) -> Result<ProxyResponse, Status> {
    let body = read_request_body(data).await?;
    let headers = build_header_map(request)?;
    let uri = request.uri().to_string();

    let proxy_request = ProxyRequestBuilder::build(method, headers, body, uri, node_service)?;

    node_service.handle_request(proxy_request).await.map_err(|err| {
        error_with_fields!("Proxy request failed", err.as_ref(),);
        Status::InternalServerError
    })
}

async fn read_request_body(data: Data<'_>) -> Result<Vec<u8>, Status> {
    let limit = BODY_READ_LIMIT_MB.mebibytes();
    let mut stream = data.open(limit);
    let mut body = Vec::new();
    stream.read_to_end(&mut body).await.map_err(|_| Status::InternalServerError)?;
    Ok(body)
}

fn build_header_map(request: &Request<'_>) -> Result<HeaderMap, Status> {
    let mut headers = HeaderMap::new();
    for header in request.headers().iter() {
        let name = HeaderName::from_bytes(header.name().as_str().as_bytes()).map_err(|_| Status::BadRequest)?;
        let value = HeaderValue::from_str(header.value()).map_err(|_| Status::BadRequest)?;
        headers.append(name, value);
    }
    Ok(headers)
}

fn build_response(proxy: ProxyResponse) -> Response<'static> {
    let ProxyResponse { status, headers, body } = proxy;

    let mut builder = Response::build();
    let status = Status::from_code(status).unwrap_or(Status::Ok);
    builder.status(status);

    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            builder.raw_header(name.as_str().to_string(), value_str.to_string());
        }
    }

    let body_len = body.len();
    builder.sized_body(body_len, Cursor::new(body));
    builder.finalize()
}

struct ProxyRocketResponse(ProxyResponse);

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for ProxyRocketResponse {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        Ok(build_response(self.0))
    }
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = NodeConfig::new()?;

    let node_address = IpAddr::from_str(config.address.as_str())?;
    let metrics_address = IpAddr::from_str(config.metrics.address.as_str())?;

    let metrics_config = dynode::config::MetricsConfig {
        prefix: config.metrics.prefix.clone(),
        user_agent_patterns: config.metrics.user_agent_patterns.clone(),
    };
    let metrics = Metrics::new(metrics_config);
    let node_service = NodeService::new(
        config.domains_map(),
        metrics.clone(),
        config.cache.clone(),
        config.monitoring.clone(),
        config.retry.clone(),
        config.request.clone(),
    );
    let node_service_clone = node_service.clone();

    rocket::tokio::spawn(async move {
        node_service_clone.start_monitoring().await;
    });

    info_with_fields!(
        "Server started",
        node_address = &format!("{}:{}", node_address, config.port),
        metrics_address = &format!("{}:{}", metrics_address, config.metrics.port),
    );

    let proxy_server = rocket::custom(Config::figment().merge(("address", node_address)).merge(("port", config.port)))
        .manage(node_service)
        .mount("/", proxy_routes())
        .mount("/", rocket::routes![health_endpoint]);

    let metrics_server = rocket::custom(Config::figment().merge(("address", metrics_address)).merge(("port", config.metrics.port)))
        .manage(metrics)
        .mount("/", rocket::routes![metrics_endpoint]);

    rocket::tokio::try_join!(proxy_server.launch(), metrics_server.launch())?;

    Ok(())
}
