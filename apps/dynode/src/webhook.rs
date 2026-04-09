use gem_auth::create_device_token;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{ChainRequest, ChainRequestProtocol, ChainRequestType, TransactionId};
use settings_chain::BroadcastProviders;
use std::time::Duration;

use crate::config::WebhookConfig;
use crate::jsonrpc_types::{JsonRpcRequest, RequestType};
use crate::proxy::proxy_request::ProxyRequest;

#[derive(Debug, Clone)]
pub struct DynodeBroadcastWebhookClient {
    enabled: bool,
    url: String,
    jwt_secret: String,
    expiry: Duration,
    client: reqwest::Client,
}

impl DynodeBroadcastWebhookClient {
    pub fn new(config: WebhookConfig, jwt_secret: String) -> Result<Self, reqwest::Error> {
        Ok(Self {
            enabled: config.enabled,
            url: config.url,
            jwt_secret,
            expiry: config.expiry,
            client: reqwest::Client::builder().timeout(config.timeout).build()?,
        })
    }

    #[cfg(test)]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            url: String::new(),
            jwt_secret: String::new(),
            expiry: Duration::from_secs(60),
            client: reqwest::Client::new(),
        }
    }

    pub fn notify_broadcast(&self, request: &ProxyRequest, response_status: u16, response_body: &[u8], broadcast_providers: &BroadcastProviders) {
        if !self.should_notify(request, response_status, broadcast_providers) {
            return;
        }

        let Some(payload) = self.extract_payload(request, response_body, broadcast_providers) else {
            return;
        };

        self.spawn_notify(payload, request.id.clone());
    }

    fn should_notify(&self, request: &ProxyRequest, response_status: u16, broadcast_providers: &BroadcastProviders) -> bool {
        self.enabled && !self.url.is_empty() && is_broadcast_request(request, broadcast_providers) && is_success_status(response_status)
    }

    fn extract_payload(&self, request: &ProxyRequest, response_body: &[u8], broadcast_providers: &BroadcastProviders) -> Option<TransactionId> {
        let response = parse_response_body(request, response_body)?;
        let identifier = broadcast_providers.decode_transaction_broadcast(request.chain, response)?;
        Some(TransactionId::new(request.chain, identifier))
    }

    fn spawn_notify(&self, payload: TransactionId, request_id: String) {
        let (token, _) = match create_device_token("dynode", &self.jwt_secret, self.expiry) {
            Ok(result) => result,
            Err(err) => {
                error_with_fields!("broadcast webhook token creation failed", &err, request_id = request_id.as_str());
                return;
            }
        };

        let url = self.url.clone();
        let client = self.client.clone();

        tokio::spawn(Self::deliver(client, url, token, payload, request_id));
    }

    async fn deliver(client: reqwest::Client, url: String, token: String, payload: TransactionId, request_id: String) {
        let transaction_id = payload.to_string();

        match client.post(&url).bearer_auth(&token).json(&payload).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info_with_fields!("broadcast webhook delivered", transaction_id = transaction_id.as_str(), request_id = request_id.as_str(),);
                } else {
                    info_with_fields!(
                        "broadcast webhook delivery failed",
                        transaction_id = transaction_id.as_str(),
                        request_id = request_id.as_str(),
                        status = response.status().as_u16(),
                    );
                }
            }
            Err(err) => {
                error_with_fields!(
                    "broadcast webhook request failed",
                    &err,
                    transaction_id = transaction_id.as_str(),
                    request_id = request_id.as_str(),
                );
            }
        }
    }
}

fn is_success_status(status: u16) -> bool {
    (200..300).contains(&status)
}

fn parse_response_body<'a>(request: &ProxyRequest, response_body: &'a [u8]) -> Option<&'a str> {
    match std::str::from_utf8(response_body) {
        Ok(response) => Some(response),
        Err(err) => {
            error_with_fields!("broadcast webhook decode failed", &err, chain = request.chain.as_ref(), request_id = request.id.as_str(),);
            None
        }
    }
}

fn is_broadcast_request(request: &ProxyRequest, broadcast_providers: &BroadcastProviders) -> bool {
    let chain_request = match request.request_type() {
        RequestType::JsonRpc(JsonRpcRequest::Single(call)) => ChainRequest::new(ChainRequestProtocol::JsonRpc, call.method.as_str(), request.path.as_str(), &request.body),
        RequestType::Regular { .. } => ChainRequest::new(ChainRequestProtocol::Http, request.method.as_str(), request.path.as_str(), &request.body),
        RequestType::JsonRpc(JsonRpcRequest::Batch(_)) => return false,
    };

    broadcast_providers.classify_request(request.chain, chain_request) == ChainRequestType::Broadcast
}

#[cfg(test)]
mod tests {
    use reqwest::header::HeaderMap;

    use super::*;
    use crate::proxy::proxy_request::ProxyRequest;
    use primitives::Chain;
    use settings_chain::BroadcastProviders;

    fn make_request(chain: Chain, method: reqwest::Method, path: &str, body: &[u8]) -> ProxyRequest {
        ProxyRequest::new(
            method,
            HeaderMap::new(),
            body.to_vec(),
            path.to_string(),
            path.to_string(),
            "example.com".to_string(),
            "test-agent".to_string(),
            chain,
        )
    }

    fn broadcast_providers() -> BroadcastProviders {
        BroadcastProviders::from_chains([Chain::Ethereum, Chain::Tron])
    }

    #[test]
    fn test_detect_broadcast_jsonrpc_single() {
        let request = make_request(
            Chain::Ethereum,
            reqwest::Method::POST,
            "/rpc",
            br#"{"jsonrpc":"2.0","method":"eth_sendRawTransaction","params":["0xdeadbeef"],"id":1}"#,
        );

        assert!(is_broadcast_request(&request, &broadcast_providers()));
    }

    #[test]
    fn test_detect_broadcast_batch_jsonrpc_skipped() {
        let request = make_request(
            Chain::Ethereum,
            reqwest::Method::POST,
            "/rpc",
            br#"[{"jsonrpc":"2.0","method":"eth_sendRawTransaction","params":["0x1"],"id":1},{"jsonrpc":"2.0","method":"eth_sendRawTransaction","params":["0x2"],"id":2}]"#,
        );

        assert!(!is_broadcast_request(&request, &broadcast_providers()));
    }

    #[test]
    fn test_detect_broadcast_http_path() {
        let request = make_request(Chain::Tron, reqwest::Method::POST, "/wallet/broadcasttransaction", br#"{"txID":"abc"}"#);

        assert!(is_broadcast_request(&request, &broadcast_providers()));
    }
}
