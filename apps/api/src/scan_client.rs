use security_provider::{AddressTarget, Metadata, ScanRequest, ScanResult, ScanTarget, SecurityProvider};
use std::error::Error;
use storage::DatabaseClient;

static PROVIDER_NAME: &str = "Gem";

pub struct ScanClient {
    database: DatabaseClient,
    pub security_providers: Vec<Box<dyn SecurityProvider + Send + Sync>>,
}

impl ScanClient {
    pub async fn new(database_url: &str, security_providers: Vec<Box<dyn SecurityProvider + Send + Sync>>) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database, security_providers }
    }

    pub fn get_scan_address(&mut self, target: &AddressTarget) -> Result<ScanResult, Box<dyn Error + Send + Sync>> {
        let scan_address = self.database.get_scan_address(target.chain, &target.address)?.as_primitive();
        Ok(ScanResult {
            is_malicious: scan_address.is_fraudulent,
            metadata: Some(Metadata {
                name: Some(scan_address.name.unwrap_or_default()),
                verified: scan_address.is_verified,
                required_memo: scan_address.is_memo_required,
            }),
            provider: PROVIDER_NAME.into(),
        })
    }

    pub async fn scan_security(&mut self, scan_request: ScanRequest) -> Result<Vec<ScanResult>, Box<dyn Error + Send + Sync>> {
        let mut results = Vec::new();

        // Check internal db first
        if let ScanTarget::Address(target) = &scan_request.target {
            let result = self.get_scan_address(target);
            if let Ok(result) = result {
                results.push(result);
            }
        }

        // Iterate over security providers
        for provider in self.security_providers.iter() {
            let result = provider.scan(&scan_request.target).await;
            match result {
                Err(e) => {
                    println!("{} error scanning: {}", provider.name(), e);
                }
                Ok(result) => {
                    results.push(result);
                }
            }
        }

        Ok(results)
    }
}
