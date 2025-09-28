use std::io::Cursor;
use std::net::IpAddr;
use std::str::FromStr;

use dynode::config::{MetricsConfig, NodeConfig};
use dynode::metrics::Metrics;
use dynode::monitoring::NodeService;
use dynode::proxy::ProxyResponse;
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
use url::Url;

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

async fn process_proxy(method: Method, request: &Request<'_>, data: Data<'_>, node_service: &NodeService) -> Result<ProxyResponse, Status> {
    let limit = BODY_READ_LIMIT_MB.mebibytes();
    let mut stream = data.open(limit);
    let mut body_vec = Vec::new();
    stream.read_to_end(&mut body_vec).await.map_err(|_| Status::InternalServerError)?;

    let mut headers = HeaderMap::new();
    for header in request.headers().iter() {
        let name = HeaderName::from_bytes(header.name().as_str().as_bytes());
        let value = HeaderValue::from_str(header.value());

        if let (Ok(name), Ok(value)) = (name, value) {
            headers.append(name, value);
        }
    }

    let host_header = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .ok_or(Status::BadRequest)?
        .to_string();
    let host = normalize_host(&host_header);

    let user_agent = headers.get("user-agent").and_then(|h| h.to_str().ok()).unwrap_or_default().to_string();

    let path = request.uri().path().to_string();
    let path_with_query = request.uri().to_string();

    let proxy_service = node_service.get_proxy_request().await;

    match proxy_service
        .handle_request(method, headers, body_vec, path, path_with_query, host, user_agent)
        .await
    {
        Ok(proxy_response) => Ok(proxy_response),
        Err(err) => {
            error_with_fields!("Proxy request failed", err.as_ref(),);
            Err(Status::InternalServerError)
        }
    }
}

fn normalize_host(raw_host: &str) -> String {
    let candidate = format!("http://{}", raw_host);
    Url::parse(&candidate)
        .ok()
        .and_then(|url| url.host_str().map(str::to_string))
        .unwrap_or_else(|| raw_host.to_string())
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

    let metrics_config = MetricsConfig {
        user_agent_patterns: config.metrics.user_agent_patterns.clone(),
    };
    let metrics = Metrics::new(metrics_config);
    let node_service = NodeService::new(config.domains_map(), metrics.clone(), config.cache.clone(), config.monitoring.clone());
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
        .mount("/", proxy_routes());

    let metrics_server = rocket::custom(Config::figment().merge(("address", metrics_address)).merge(("port", config.metrics.port)))
        .manage(metrics)
        .mount("/", rocket::routes![metrics_endpoint]);

    rocket::tokio::try_join!(proxy_server.launch(), metrics_server.launch())?;

    Ok(())
}
