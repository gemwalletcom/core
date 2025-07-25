use primitives::{ScanTransaction, ScanTransactionPayload};
use rocket::futures::future;
use security_provider::{AddressTarget, ScanProvider, ScanResult, TokenTarget};
use std::error::Error;
use storage::DatabaseClient;

pub struct ScanClient {
    database: DatabaseClient,
    pub security_providers: Vec<Box<dyn ScanProvider + Send + Sync>>,
}

impl ScanClient {
    pub async fn new(database_url: &str, security_providers: Vec<Box<dyn ScanProvider + Send + Sync>>) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database, security_providers }
    }

    pub async fn get_scan_transaction(&mut self, payload: ScanTransactionPayload) -> Result<ScanTransaction, Box<dyn Error + Send + Sync>> {
        let local_scan = self.get_scan_transaction_local(payload.clone())?;
        if local_scan.is_malicious {
            return Ok(local_scan);
        }

        let target = AddressTarget {
            chain: payload.origin.chain,
            address: payload.origin.address.clone(),
        };
        let providers_scan = self.scan_address_providers(target).await?;

        //TODO: Store into DB / if is_malicious

        Ok(ScanTransaction {
            is_malicious: providers_scan.iter().any(|scan| scan.is_malicious),
            is_memo_required: local_scan.is_memo_required,
        })
    }

    fn get_scan_transaction_local(&mut self, payload: ScanTransactionPayload) -> Result<ScanTransaction, Box<dyn Error + Send + Sync>> {
        let queries = [
            (payload.origin.chain, payload.origin.address.as_str()),
            (payload.target.chain, payload.target.address.as_str()),
        ];
        let addresses = self.database.scan_addresses().get_scan_addresses(&queries)?;
        let is_malicious = addresses.iter().any(|address| address.is_fraudulent);
        let is_memo_required = addresses.iter().any(|address| address.is_memo_required);

        Ok(ScanTransaction {
            is_malicious,
            is_memo_required,
        })
    }

    pub async fn scan_address_providers(&mut self, target: AddressTarget) -> Result<Vec<ScanResult<AddressTarget>>, Box<dyn Error + Send + Sync>> {
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
    pub async fn scan_token(&mut self, chain: primitives::Chain, token_id: &str) -> Result<Vec<ScanResult<TokenTarget>>, Box<dyn Error + Send + Sync>> {
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

    pub async fn get_scan_address(&mut self, address: &str) -> Result<Vec<primitives::ScanAddress>, Box<dyn Error + Send + Sync>> {
        let scan_addresses = self.database.scan_addresses().get_scan_addresses_by_addresses(vec![address.to_string()])?;
        Ok(scan_addresses.into_iter().map(|addr| addr.as_scan_address_primitive()).collect())
    }
}
