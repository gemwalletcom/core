use gem_client::ReqwestClient;
use gem_tracing::error_with_fields;
use primitives::{ScanTransaction, ScanTransactionPayload};
use rocket::futures::future;
use security_provider::providers::goplus::GoPlusProvider;
use security_provider::providers::hashdit::HashDitProvider;
use security_provider::{AddressTarget, ScanProvider, ScanResult, TokenTarget};
use settings::Settings;
use std::error::Error;
use std::sync::Arc;
use storage::{Database, ScanAddressesRepository};

pub struct ScanProviderFactory {}

impl ScanProviderFactory {
    pub fn create_providers(settings: &Settings) -> Vec<Box<dyn ScanProvider + Send + Sync>> {
        let client = gem_client::builder().timeout(settings.scan.timeout).build().unwrap();

        vec![
            Box::new(GoPlusProvider::new(
                ReqwestClient::new(settings.scan.goplus.url.clone(), client.clone()),
                &settings.scan.goplus.key.public,
            )),
            Box::new(HashDitProvider::new(
                ReqwestClient::new(settings.scan.hashdit.url.clone(), client.clone()),
                &settings.scan.hashdit.key.public,
                &settings.scan.hashdit.key.secret,
            )),
        ]
    }
}

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
        let addresses = self.database.scan_addresses()?.get_scan_addresses(&queries)?;
        let is_malicious = addresses.iter().any(|address| address.is_fraudulent);
        let is_memo_required = addresses.iter().any(|address| address.is_memo_required);

        Ok(ScanTransaction { is_malicious, is_memo_required })
    }

    pub async fn scan_address_providers(&self, target: AddressTarget) -> Result<Vec<ScanResult<AddressTarget>>, Box<dyn Error + Send + Sync>> {
        let results = future::join_all(self.security_providers.iter().map(|provider| provider.scan_address(&target)))
            .await
            .into_iter()
            .filter_map(|result| match result {
                Err(e) => {
                    error_with_fields!("error scanning", e.as_ref(),);
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
                    error_with_fields!("error scanning token", e.as_ref(),);
                    None
                }
                Ok(result) => Some(result),
            })
            .collect();
        Ok(results)
    }
}
