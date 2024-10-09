use crate::models::SecurityResponse;
use async_trait::async_trait;
use security_provider::{ScanResult, ScanTarget, SecurityProvider, DEFAULT_TIMEOUT};
use serde_json::json;
use std::result::Result;

static PROVIDER_NAME: &str = "GoPlus";
const MALICIOUS_URL_KEYS: &[&str] = &["phishing_site"];

const MALICIOUS_ADDRESS_KEYS: &[&str] = &[
    "cybercrime",
    "money_laundering",
    "number_of_malicious_contracts_created",
    "gas_abuse",
    "financial_crime",
    "darkweb_transactions",
    "reinit",
    "phishing_activities",
    "fake_kyc",
    "blacklist_doubt",
    "fake_standard_interface",
    "data_source",
    "stealing_attack",
    "blackmail_activities",
    "sanctioned",
    "malicious_mining_activities",
    "mixer",
    "fake_token",
    "honeypot_related_address",
];

pub struct GoPlusProvider {
    client: reqwest::Client,
    url: String,
}

#[async_trait]
impl SecurityProvider for GoPlusProvider {
    fn new(url: &str, _api_key: &str) -> Self {
        GoPlusProvider {
            client: reqwest::Client::new(),
            url: url.into(),
        }
    }

    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    async fn scan(&self, target: &ScanTarget) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let url: String = match target {
            ScanTarget::Address(target) => format!("{}/api/v1/address_security/{}", self.url, target.address),
            ScanTarget::URL(_) => format!("{}/api/v1/phishing_site", self.url),
        };
        let query = match target {
            ScanTarget::Address(target) => [("chain_id", target.chain.network_id())],
            ScanTarget::URL(url) => [("url", url.as_str())],
        };

        let response = self
            .client
            .get(&url)
            .query(&query)
            .timeout(DEFAULT_TIMEOUT)
            .send()
            .await?
            .json::<SecurityResponse>()
            .await?;
        if response.code != 1 {
            return Err(response.message.into());
        }

        let mut is_malicious = false;
        let mut reason: Option<String> = None;

        let keys: &[&str] = match target {
            ScanTarget::Address(_) => MALICIOUS_ADDRESS_KEYS,
            ScanTarget::URL(_) => MALICIOUS_URL_KEYS,
        };

        if let Some(result) = response.result {
            let safe_value = json!("0");
            let malicious_value = String::from("1");
            for key in keys {
                let value = result.get(key).unwrap_or(&safe_value);
                if *value.to_string() == malicious_value {
                    is_malicious = true;
                    reason = Some(format!("Category: {}", key));
                    break;
                }
            }
        }

        Ok(ScanResult {
            is_malicious,
            reason,
            provider: self.name().into(),
            metadata: None,
        })
    }
}
