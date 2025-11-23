use primitives::{ScanTransaction, ScanTransactionPayload};
use rocket::futures::future;
use security_provider::{AddressTarget, ScanProvider, ScanResult, TokenTarget};
use std::error::Error;
use std::sync::Arc;
use storage::Database;

#[derive(Clone)]
pub struct ScanClient {
    database: Database,
    pub security_providers: Vec<Arc<dyn ScanProvider + Send + Sync>>,
}

impl ScanClient {
    pub fn new(database: Database, security_providers: Vec<Box<dyn ScanProvider + Send + Sync>>) -> Self {
        let security_providers = security_providers.into_iter().map(Arc::from).collect();
        Self { database, security_providers }
    }

    pub async fn get_scan_transaction(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, Box<dyn Error + Send + Sync>> {
        let local_scan = self.get_scan_transaction_local(payload.clone())?;
        if local_scan.is_malicious {
            return Ok(local_scan);
        }

        let target = AddressTarget {
            chain: payload.origin.asset_id.chain,
            address: payload.origin.address.clone(),
        };
        let providers_scan = self.scan_address_providers(target).await?;

        Ok(ScanTransaction {
            is_malicious: providers_scan.iter().any(|scan| scan.is_malicious),
            is_memo_required: local_scan.is_memo_required,
        })
    }

    fn get_scan_transaction_local(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, Box<dyn Error + Send + Sync>> {
        let queries = [
            (payload.origin.asset_id.chain, payload.origin.address.as_str()),
            (payload.target.asset_id.chain, payload.target.address.as_str()),
        ];
        let addresses = self.database.client()?.scan_addresses().get_scan_addresses(&queries)?;
        let is_malicious = addresses.iter().any(|address| address.is_fraudulent);
        let is_memo_required = addresses.iter().any(|address| address.is_memo_required);

        Ok(ScanTransaction {
            is_malicious,
            is_memo_required,
        })
    }

    pub async fn scan_address_providers(&self, target: AddressTarget) -> Result<Vec<ScanResult<AddressTarget>>, Box<dyn Error + Send + Sync>> {
        let results = future::join_all(self.security_providers.iter().map(|provider| provider.scan_address(&target)))
            .await
            .into_iter()
            .filter_map(|result| match result {
                Err(e) => {
                    println!("error scanning: {e}");
                    None
                }
                Ok(result) => Some(result),
            })
            .collect();
        Ok(results)
    }

    #[allow(dead_code)]
    pub async fn scan_token(&self, chain: primitives::Chain, token_id: &str) -> Result<Vec<ScanResult<TokenTarget>>, Box<dyn Error + Send + Sync>> {
        let target = TokenTarget {
            token_id: token_id.to_string(),
            chain,
        };

        let results = future::join_all(self.security_providers.iter().map(|provider| provider.scan_token(&target)))
            .await
            .into_iter()
            .filter_map(|result| match result {
                Err(e) => {
                    println!("error scanning token: {e}");
                    None
                }
                Ok(result) => Some(result),
            })
            .collect();
        Ok(results)
    }

    pub async fn get_scan_address(&self, address: &str) -> Result<Vec<primitives::ScanAddress>, Box<dyn Error + Send + Sync>> {
        let scan_addresses = self
            .database
            .client()?
            .scan_addresses()
            .get_scan_addresses_by_addresses(vec![address.to_string()])?;
        Ok(scan_addresses.into_iter().map(|addr| addr.as_scan_address_primitive()).collect())
    }
}
