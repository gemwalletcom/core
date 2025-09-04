use crate::providers::hashdit::models::DetectResponse;
use crate::{mapper, AddressTarget, ScanProvider, ScanResult, TokenTarget};
use async_trait::async_trait;
use gem_client::{Client, ClientError, ReqwestClient};
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
type HmacSha256 = Hmac<Sha256>;

static PROVIDER_NAME: &str = "HashDit";

pub struct HashDitProvider {
    client: ReqwestClient,
    app_id: String,
    app_secret: String,
}

impl HashDitProvider {
    pub fn new(client: ReqwestClient, app_id: &str, app_secret: &str) -> Self {
        HashDitProvider {
            client,
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
        }
    }

    fn generate_msg_for_sig(&self, timestamp: &str, nonce: &str, method: &str, url: &str, query: &str, body: &str) -> String {
        if !query.is_empty() {
            format!("{};{};{};{};{};{};{}", self.app_id, timestamp, nonce, method, url, query, body)
        } else {
            format!("{};{};{};{};{};{}", self.app_id, timestamp, nonce, method, url, body)
        }
    }

    fn compute_sig(&self, msg_for_sig: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.app_secret.as_bytes()).expect("HMAC can take key of any size");
        mac.update(msg_for_sig.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        hex::encode(code_bytes)
    }

    async fn send_request<T: serde::Serialize + Send + Sync>(&self, business: &str, body: &T) -> Result<DetectResponse, ClientError> {
        let query = HashMap::from([("business".to_string(), business.to_string())]);
        let query_str = query.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<String>>().join("&");
        let method = "POST";
        let path = "/security-api/public/app/v1/detect";

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            .to_string();
        let nonce: String = uuid::Uuid::new_v4().to_string().replace('-', "");

        let body_str = serde_json::to_string(body).unwrap_or_default();
        let msg_for_sig = self.generate_msg_for_sig(&timestamp, &nonce, method, path, &query_str, &body_str);
        let sig = self.compute_sig(&msg_for_sig);

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json;charset=UTF-8".to_string());
        headers.insert("X-Signature-appid".to_string(), self.app_id.clone());
        headers.insert("X-Signature-signature".to_string(), sig);
        headers.insert("X-Signature-timestamp".to_string(), timestamp);
        headers.insert("X-Signature-nonce".to_string(), nonce);

        let url = format!("{}?{}", path, query_str);
        self.client.post(&url, body, Some(headers)).await
    }

    fn parse_response(response: DetectResponse) -> Result<(bool, Option<String>), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(error_data) = response.error_data {
            return Err(Box::from(error_data));
        }

        let mut is_malicious = false;
        let mut reason: Option<String> = None;

        if let Some(data) = response.data {
            let has_result = data.has_result.unwrap_or_else(|| data.risk_level.is_some());
            if has_result {
                let level = data.risk_level.unwrap_or(0);
                // 3 - Medium Risk
                is_malicious = level >= 3;
                reason = Some(format!("Risk level: {}", level));
            } else {
                is_malicious = false;
                reason = Some("No data found".to_string());
            }
        }

        Ok((is_malicious, reason))
    }

    async fn _scan<T: Clone + Send + Sync + 'static>(
        &self,
        target: &T,
        business: &str,
        body: &Value,
    ) -> Result<ScanResult<T>, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.send_request(business, body).await?;
        let (is_malicious, reason) = Self::parse_response(response)?;
        Ok(ScanResult {
            target: target.clone(),
            is_malicious,
            reason,
            provider: PROVIDER_NAME.into(),
        })
    }
}

#[async_trait]
impl ScanProvider for HashDitProvider {
    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    async fn scan_address(&self, target: &AddressTarget) -> Result<ScanResult<AddressTarget>, Box<dyn std::error::Error + Send + Sync>> {
        let body = json!({
            "chain_id": mapper::chain_to_provider_id(target.chain),
            "address": target.address,
        });
        self._scan(target, "gem_wallet_address_detection", &body).await
    }

    async fn scan_token(&self, target: &TokenTarget) -> Result<ScanResult<TokenTarget>, Box<dyn std::error::Error + Send + Sync>> {
        let body = json!({
            "chain_id": mapper::chain_to_provider_id(target.chain),
            "address": target.token_id,
        });
        self._scan(target, "gem_wallet_token_detection", &body).await
    }

    async fn scan_url(&self, target: &str) -> Result<ScanResult<String>, Box<dyn std::error::Error + Send + Sync>> {
        let body = json!({
            "url": target,
        });
        let _response = self.send_request("gem_wallet_url_detection", &body).await?;

        unimplemented!()
    }
}
