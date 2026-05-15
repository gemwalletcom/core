use async_trait::async_trait;
use std::{collections::HashMap, error::Error, fmt, sync::Arc};

use crate::{
    alien,
    rpc::{HttpMethod, Target},
};

#[async_trait]
pub trait GrpcTransport: Send + Sync + fmt::Debug {
    async fn unary(&self, endpoint: &str, path: &str, body: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>>;
}

fn unary_target(endpoint: &str, path: &str, body: Vec<u8>) -> Target {
    Target {
        url: format!("{}{}", endpoint.trim_end_matches('/'), path),
        method: HttpMethod::Post,
        headers: Some(grpc_headers()),
        body: Some(body),
    }
}

fn ensure_success_status(status: Option<u16>) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(status) = status
        && !(200..300).contains(&status)
    {
        return Err(format!("gRPC HTTP error: status {status}").into());
    }
    Ok(())
}

fn grpc_headers() -> HashMap<String, String> {
    HashMap::from([
        ("Content-Type".into(), "application/grpc+proto".into()),
        ("Accept".into(), "application/grpc+proto".into()),
        ("TE".into(), "trailers".into()),
    ])
}

#[derive(Clone)]
pub struct AlienGrpcTransport {
    provider: Arc<dyn alien::RpcProvider>,
}

impl AlienGrpcTransport {
    pub fn new(provider: Arc<dyn alien::RpcProvider>) -> Self {
        Self { provider }
    }
}

impl fmt::Debug for AlienGrpcTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AlienGrpcTransport").finish_non_exhaustive()
    }
}

#[async_trait]
impl GrpcTransport for AlienGrpcTransport {
    async fn unary(&self, endpoint: &str, path: &str, body: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let response = self.provider.request(unary_target(endpoint, path, body)).await?;
        ensure_success_status(response.status)?;
        Ok(response.data)
    }
}

#[cfg(feature = "reqwest")]
#[derive(Clone, Debug)]
pub struct ReqwestGrpcTransport {
    client: reqwest::Client,
}

#[cfg(feature = "reqwest")]
impl ReqwestGrpcTransport {
    pub fn new() -> Self {
        Self { client: reqwest::Client::new() }
    }

    pub fn new_with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[cfg(feature = "reqwest")]
impl Default for ReqwestGrpcTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "reqwest")]
#[async_trait]
impl GrpcTransport for ReqwestGrpcTransport {
    async fn unary(&self, endpoint: &str, path: &str, body: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let response = self
            .client
            .post(format!("{}{}", endpoint.trim_end_matches('/'), path))
            .header("Content-Type", "application/grpc+proto")
            .header("Accept", "application/grpc+proto")
            .header("TE", "trailers")
            .body(body)
            .send()
            .await?;
        let status = response.status().as_u16();
        let bytes = response.bytes().await?.to_vec();
        ensure_success_status(Some(status))?;
        Ok(bytes)
    }
}
