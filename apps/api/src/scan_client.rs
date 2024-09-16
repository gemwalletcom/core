use primitives::{Chain, ScanAddress};
use security_provider::{ScanRequest, ScanResult, SecurityProvider};
use std::error::Error;
use storage::DatabaseClient;

pub struct ScanClient {
    database: DatabaseClient,
    pub security_providers: Vec<Box<dyn SecurityProvider + Send + Sync>>,
}

impl ScanClient {
    pub async fn new(database_url: &str, security_providers: Vec<Box<dyn SecurityProvider + Send + Sync>>) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database, security_providers }
    }

    pub fn get_scan_address(&mut self, chain: Chain, address: &str) -> Result<ScanAddress, Box<dyn Error>> {
        Ok(self.database.get_scan_address(chain, address)?.as_primitive())
    }

    pub async fn scan_security(&mut self, scan_request: ScanRequest) -> Result<ScanResult, Box<dyn Error>> {
        let mut results = Vec::new();

        for provider in self.security_providers.iter() {
            let result = provider.scan(&scan_request.target, &scan_request.target_type).await;
            results.push(result);
        }

        // Combine results from multiple providers (you may want to implement a more sophisticated logic here)
        let combined_result = ScanResult {
            is_malicious: results.iter().any(|r| r.is_malicious),
            risk_score: results.iter().map(|r| r.risk_score).max().unwrap_or(0),
            details: results.iter().map(|r| r.details.clone()).collect::<Vec<_>>().join(" | "),
        };

        Ok(combined_result)
    }
}
