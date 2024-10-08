extern crate rocket;
use primitives::{SecurityMetadata, SecurityResponse};
use rocket::{futures::future, serde::json::Json, tokio::sync::Mutex, State};
use security_provider::{AddressTarget, ScanRequest, ScanResult, ScanTarget, SecurityProvider};
use std::error::Error;
use storage::DatabaseClient;

static PROVIDER_NAME: &str = "Gem";
static REASON: &str = "Moderation";

#[post("/scan/security", data = "<scan_request>")]
pub async fn scan(scan_request: Json<ScanRequest>, client: &State<Mutex<SecurityScanClient>>) -> Json<SecurityResponse> {
    let result = client.lock().await.scan_security(scan_request.0).await.unwrap();
    Json(result)
}

pub struct SecurityScanClient {
    database: DatabaseClient,
    pub security_providers: Vec<Box<dyn SecurityProvider + Send + Sync>>,
}

impl SecurityScanClient {
    pub async fn new(database_url: &str, security_providers: Vec<Box<dyn SecurityProvider + Send + Sync>>) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database, security_providers }
    }

    pub fn get_scan_address(&mut self, target: &AddressTarget) -> Result<ScanResult, Box<dyn Error + Send + Sync>> {
        let scan_address = self.database.get_scan_address(target.chain, &target.address)?;
        Ok(ScanResult {
            is_malicious: scan_address.is_fraudulent,
            reason: Some(REASON.into()),
            metadata: Some(SecurityMetadata {
                name: scan_address.name.unwrap_or_default(),
                verified: scan_address.is_verified,
                required_memo: scan_address.is_memo_required,
            }),
            provider: PROVIDER_NAME.into(),
        })
    }

    pub async fn scan_security(&mut self, scan_request: ScanRequest) -> Result<SecurityResponse, Box<dyn Error + Send + Sync>> {
        let mut results: Vec<ScanResult> = vec![];
        let mut metadata: Option<SecurityMetadata> = None;

        // Check internal db first
        if let ScanTarget::Address(target) = &scan_request.target {
            let result = self.get_scan_address(target);
            if let Ok(result) = result {
                metadata = result.metadata.clone();
                results.push(result);
            }
        }

        let scanned = future::join_all(self.security_providers.iter().map(|provider| provider.scan(&scan_request.target)))
            .await
            .into_iter()
            .filter_map(|result| match result {
                Err(e) => {
                    println!("error scanning: {}", e);
                    None
                }
                Ok(result) => Some(result),
            });
        results.extend(scanned);

        Ok(SecurityResponse {
            malicious: results.iter().any(|result| result.is_malicious),
            reason: results
                .iter()
                .filter(|result| result.is_malicious)
                .map(|result| result.reason.clone().unwrap_or_default())
                .collect::<Vec<_>>()
                .join("|"),
            provider: results
                .iter()
                .filter(|result| result.is_malicious)
                .map(|result| result.provider.clone())
                .collect::<Vec<_>>()
                .join("|"),
            metadata,
        })
    }
}
