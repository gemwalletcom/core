use std::error::Error;

use crate::database::scan_addresses::ScanAddressesStore;
use crate::models::{ScanAddress, ScanAddressType};
use crate::DatabaseClient;
use primitives::Chain;

pub trait ScanAddressesRepository {
    fn get_scan_address(&mut self, _chain: Chain, value: &str) -> Result<ScanAddress, Box<dyn Error + Send + Sync>>;
    fn get_scan_addresses(&mut self, queries: &[(Chain, &str)]) -> Result<Vec<ScanAddress>, Box<dyn Error + Send + Sync>>;
    fn add_scan_address_types(&mut self, values: Vec<ScanAddressType>) -> Result<usize, Box<dyn Error + Send + Sync>>;
}

impl ScanAddressesRepository for DatabaseClient {
    fn get_scan_address(&mut self, _chain: Chain, value: &str) -> Result<ScanAddress, Box<dyn Error + Send + Sync>> {
        Ok(ScanAddressesStore::get_scan_address(self, _chain, value)?)
    }

    fn get_scan_addresses(&mut self, queries: &[(Chain, &str)]) -> Result<Vec<ScanAddress>, Box<dyn Error + Send + Sync>> {
        Ok(ScanAddressesStore::get_scan_addresses(self, queries)?)
    }

    fn add_scan_address_types(&mut self, values: Vec<ScanAddressType>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(ScanAddressesStore::add_scan_address_types(self, values)?)
    }
}
