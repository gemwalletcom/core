use crate::api::HashDitApi;
use crate::models::DetectResponse;

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use reqwest_enum::target::Target;
use security_provider::{ScanResult, ScanTarget, SecurityProvider};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};
type HmacSha256 = Hmac<Sha256>;

static PROVIDER_NAME: &str = "HashDit";

pub struct HashDitProvider {
    client: reqwest::Client,
    app_id: String,
    app_secret: String,
}

impl HashDitProvider {
    fn generate_msg_for_sig(&self, timestamp: &str, nonce: &str, method: &str, url: &str, query: &str, body: &[u8]) -> String {
        let body = if body.is_empty() { "" } else { std::str::from_utf8(body).unwrap_or_default() };
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

    fn build_request(&self, target: HashDitApi) -> reqwest::RequestBuilder {
        let url = format!("{}{}", target.base_url(), target.path());
        let mut request = self.client.request(target.method().into(), url);

        let query = target.query();
        let query_str = target.query_string();
        let body = target.body().to_bytes();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            .to_string();
        let nonce: String = uuid::Uuid::new_v4().to_string().replace("-", "");
        let method = target.method().to_string();

        // Generate message for signature
        let msg_for_sig = self.generate_msg_for_sig(&timestamp, &nonce, &method, target.path(), &query_str, &body);
        let sig = self.compute_sig(&msg_for_sig);

        request = request.query(&query);
        request = request.header("Content-Type", "application/json;charset=UTF-8");
        request = request.header("X-Signature-appid", &self.app_id);
        request = request.header("X-Signature-signature", sig);
        request = request.header("X-Signature-timestamp", timestamp);
        request = request.header("X-Signature-nonce", nonce);
        request = request.body(body);
        request
    }

    fn new(app_id: &str, app_secret: &str) -> Self {
        HashDitProvider {
            client: reqwest::Client::new(),
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
        }
    }
}

#[async_trait]
impl SecurityProvider for HashDitProvider {
    fn new(app_id: &str, app_secret: &str) -> Self {
        // Initialize HashDit client
        HashDitProvider::new(app_id, app_secret)
    }

    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    async fn scan(&self, target: &ScanTarget) -> std::result::Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let api = match target {
            ScanTarget::Address(address_target) => HashDitApi::DetectAddress(address_target.address.clone(), address_target.chain.network_id().into()),
            ScanTarget::URL(scan_url) => HashDitApi::DetectURL(scan_url.clone()),
        };
        let request = self.build_request(api);
        let response = self.client.execute(request.build()?).await?;
        let mut is_malicious = false;
        let mut reason: Option<String> = None;

        let body = response.json::<DetectResponse>().await?;
        if let Some(error_data) = body.error_data {
            return Err(Box::from(error_data));
        }
        if let Some(data) = body.data {
            if data.has_result {
                // 0 - Very Low Risk
                // 1 - Some Risk
                // 2 - Low Risk
                // 3 - Medium Risk
                // 4 - High Risk
                // 5 - Significant Risk
                is_malicious = data.risk_level >= 3;
                reason = Some(format!("Risk level: {}", data.risk_level));
            } else {
                return Err(Box::from("No data found"));
            }
        }

        // Implement HashDit-specific scanning logic
        Ok(ScanResult {
            is_malicious,
            reason,
            provider: PROVIDER_NAME.into(),
            metadata: None,
        })
    }
}
