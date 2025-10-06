use crate::monitoring::NodeService;
use crate::proxy::proxy_request::ProxyRequest;
use reqwest::{Method, header::HeaderMap};
use rocket::http::Status;
use url::Url;

pub struct ProxyRequestBuilder;

impl ProxyRequestBuilder {
    pub fn build(method: Method, headers: HeaderMap, body: Vec<u8>, uri: String, node_service: &NodeService) -> Result<ProxyRequest, Status> {
        let host = Self::extract_host(&headers)?;
        let user_agent = Self::extract_user_agent(&headers);
        let path = Self::extract_path(&uri);
        let resolution = node_service.resolve_chain(&host, &path).ok_or(Status::BadRequest)?;
        let (path, path_with_query) = Self::prepare_paths(&uri, resolution.is_path_based());

        Ok(ProxyRequest::new(
            method,
            headers,
            body,
            path,
            path_with_query,
            host,
            user_agent,
            resolution.chain(),
        ))
    }

    fn extract_host(headers: &HeaderMap) -> Result<String, Status> {
        let host_header = headers.get(reqwest::header::HOST).and_then(|h| h.to_str().ok()).ok_or(Status::BadRequest)?;

        Ok(Self::parse_hostname(host_header))
    }

    fn extract_user_agent(headers: &HeaderMap) -> String {
        headers
            .get(reqwest::header::USER_AGENT)
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default()
            .to_string()
    }

    fn extract_path(uri: &str) -> String {
        uri.split('?').next().unwrap_or(uri).to_string()
    }

    fn prepare_paths(uri: &str, path_based_routing: bool) -> (String, String) {
        if path_based_routing {
            (Self::remove_chain_from_path(&Self::extract_path(uri)), Self::remove_chain_from_path(uri))
        } else {
            (Self::extract_path(uri), uri.to_string())
        }
    }

    fn parse_hostname(host_header: &str) -> String {
        let candidate = format!("http://{}", host_header);
        Url::parse(&candidate)
            .ok()
            .and_then(|url| url.host_str().map(str::to_string))
            .unwrap_or_else(|| host_header.to_string())
    }

    fn remove_chain_from_path(uri: &str) -> String {
        let (path_part, query_part) = uri.split_once('?').unwrap_or((uri, ""));

        let remaining = path_part
            .trim_start_matches('/')
            .split_once('/')
            .map(|(_, rest)| format!("/{}", rest))
            .unwrap_or_else(|| "/".to_string());

        if query_part.is_empty() {
            remaining
        } else {
            format!("{}?{}", remaining, query_part)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_chain_from_path() {
        assert_eq!(
            ProxyRequestBuilder::remove_chain_from_path("/tron/wallet/getchainparameters"),
            "/wallet/getchainparameters"
        );
        assert_eq!(ProxyRequestBuilder::remove_chain_from_path("/ethereum/v1/some/path"), "/v1/some/path");
        assert_eq!(ProxyRequestBuilder::remove_chain_from_path("/bitcoin"), "/");
        assert_eq!(ProxyRequestBuilder::remove_chain_from_path("/solana?query=1"), "/?query=1");
        assert_eq!(
            ProxyRequestBuilder::remove_chain_from_path("/chain/path?foo=bar&baz=qux"),
            "/path?foo=bar&baz=qux"
        );
    }

    #[test]
    fn test_parse_hostname() {
        assert_eq!(ProxyRequestBuilder::parse_hostname("example.com"), "example.com");
        assert_eq!(ProxyRequestBuilder::parse_hostname("example.com:8080"), "example.com");
        assert_eq!(ProxyRequestBuilder::parse_hostname("localhost:3000"), "localhost");
    }
}
