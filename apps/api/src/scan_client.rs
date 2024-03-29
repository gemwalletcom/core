use std::error::Error;

use primitives::{Chain, ScanAddress};
use storage::DatabaseClient;

pub struct ScanClient {
    database: DatabaseClient,
}

impl ScanClient {
    pub async fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub fn get_scan_address(
        &mut self,
        chain: Chain,
        address: &str,
    ) -> Result<ScanAddress, Box<dyn Error>> {
        Ok(self
            .database
            .get_scan_address(chain, address)?
            .as_primitive())
    }
}
