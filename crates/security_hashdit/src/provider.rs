use async_trait::async_trait;
use hmac::{Hmac, Mac};
use rand::Rng;
use security_provider::{ScanResult, SecurityProvider};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

pub struct HashDitProvider {
    app_id: String,
    app_secret: String,
    // Add any other necessary fields
}

impl HashDitProvider {
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

    fn new(app_id: &str, app_secret: &str) -> Self {
        HashDitProvider {
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

    async fn scan(&self, target: &str, target_type: &str) -> ScanResult {
        // Generate timestamp and nonce
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs().to_string();
        let nonce: String = rand::thread_rng().gen_range(0..9999999).to_string();
        let method = "POST";
        let url = "/security-api/public/app/v1/detect";
        let query = "";
        let body = target; // Assuming target is the request body

        // Generate message for signature
        let msg_for_sig = self.generate_msg_for_sig(&timestamp, &nonce, method, url, query, body);
        let sig = self.compute_sig(&msg_for_sig);

        // Implement HashDit-specific scanning logic
        ScanResult {
            is_malicious: false,
            risk_score: 0,
            details: format!("HashDit scan completed. Signature: {}", sig),
        }
    }
}
