use reqwest::{Method, header::HeaderMap};
use std::time::Instant;
use primitives::Chain;
use crate::jsonrpc_types::RequestType;

#[derive(Debug, Clone)]
pub struct ProxyRequest {
    pub method: Method,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
    pub path: String,
    pub path_with_query: String,
    pub host: String,
    pub user_agent: String,
    pub chain: Chain,
    pub request_start: Instant,
}

impl ProxyRequest {
    pub fn new(
        method: Method,
        headers: HeaderMap,
        body: Vec<u8>,
        path: String,
        path_with_query: String,
        host: String,
        user_agent: String,
        chain: Chain,
    ) -> Self {
        Self {
            method,
            headers,
            body,
            path,
            path_with_query,
            host,
            user_agent,
            chain,
            request_start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.request_start.elapsed()
    }

    pub fn request_type(&self) -> RequestType {
        RequestType::from_request(self.method.as_str(), self.path_with_query.clone(), self.body.clone())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn request_creation() {
        let ctx = ProxyRequest::new(
            Method::GET,
            HeaderMap::new(),
            vec![],
            "/test".to_string(),
            "/test?param=1".to_string(),
            "example.com".to_string(),
            "test-agent".to_string(),
            Chain::Ethereum,
        );

        assert_eq!(ctx.method, Method::GET);
        assert_eq!(ctx.path, "/test");
        assert_eq!(ctx.host, "example.com");
        assert_eq!(ctx.user_agent, "test-agent");
        assert_eq!(ctx.chain, Chain::Ethereum);
    }

    #[test]
    fn elapsed_time() {
        let ctx = ProxyRequest::new(
            Method::GET,
            HeaderMap::new(),
            vec![],
            "/test".to_string(),
            "/test".to_string(),
            "example.com".to_string(),
            "test-agent".to_string(),
            Chain::Ethereum,
        );

        thread::sleep(Duration::from_millis(1));

        let elapsed = ctx.elapsed();
        assert!(elapsed.as_millis() > 0);
    }
}