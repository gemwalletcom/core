use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AbuseIPDBClient {
    client: reqwest::Client,
    url: String,
    api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbuseIPDBResponse {
    pub data: AbuseIPDBData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbuseIPDBData {
    pub ip_address: String,
    pub is_public: bool,
    pub ip_version: i32,
    pub is_whitelisted: Option<bool>,
    pub abuse_confidence_score: i32,
    pub country_code: String,
    pub usage_type: Option<String>,
    pub is_tor: bool,
    pub total_reports: i32,
}

const BLOCKED_USAGE_KEYWORDS: [&str; 4] = ["Data Center", "Web Hosting", "Transit", "Content Delivery Network"];

impl AbuseIPDBData {
    pub fn is_suspicious(&self) -> bool {
        if self.abuse_confidence_score >= 10 {
            return true;
        }
        if self.is_tor {
            return true;
        }
        if self.total_reports > 0 {
            return true;
        }
        if let Some(usage_type) = &self.usage_type {
            for keyword in BLOCKED_USAGE_KEYWORDS {
                if usage_type.contains(keyword) {
                    return true;
                }
            }
        }
        false
    }
}

impl AbuseIPDBClient {
    pub fn new(url: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
            api_key,
        }
    }

    pub async fn check_ip(&self, ip_address: &str) -> Result<AbuseIPDBData, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/check", self.url);
        let response = self
            .client
            .get(&url)
            .header("Key", &self.api_key)
            .header("Accept", "application/json")
            .query(&[("ipAddress", ip_address)])
            .send()
            .await?
            .json::<AbuseIPDBResponse>()
            .await?;

        Ok(response.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_data(abuse_confidence_score: i32, is_tor: bool, total_reports: i32, usage_type: Option<&str>) -> AbuseIPDBData {
        AbuseIPDBData {
            ip_address: "1.2.3.4".to_string(),
            is_public: true,
            ip_version: 4,
            is_whitelisted: None,
            abuse_confidence_score,
            country_code: "US".to_string(),
            usage_type: usage_type.map(|s| s.to_string()),
            is_tor,
            total_reports,
        }
    }

    #[test]
    fn test_is_suspicious() {
        assert!(!create_data(0, false, 0, None).is_suspicious());
        assert!(!create_data(9, false, 0, Some("Fixed Line ISP")).is_suspicious());
        assert!(create_data(10, false, 0, None).is_suspicious());
        assert!(create_data(0, true, 0, None).is_suspicious());
        assert!(create_data(0, false, 1, None).is_suspicious());
        assert!(create_data(0, false, 0, Some("Data Center/Web Hosting/Transit")).is_suspicious());
        assert!(create_data(0, false, 0, Some("Content Delivery Network")).is_suspicious());
    }
}
