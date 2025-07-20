use std::error::Error;

use crate::database::scan_addresses::ScanAddressesStore;
use crate::models::{NewScanAddress, ScanAddress as StorageScanAddress, ScanAddressType};
use crate::DatabaseClient;
use primitives::{Chain, ScanAddress};

pub trait ScanAddressesRepository {
    fn get_scan_address(&mut self, _chain: Chain, value: &str) -> Result<StorageScanAddress, Box<dyn Error + Send + Sync>>;
    fn get_scan_addresses(&mut self, queries: &[(Chain, &str)]) -> Result<Vec<StorageScanAddress>, Box<dyn Error + Send + Sync>>;
    fn get_scan_addresses_by_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<StorageScanAddress>, Box<dyn Error + Send + Sync>>;
    fn add_scan_address_types(&mut self, values: Vec<ScanAddressType>) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn add_scan_addresses(&mut self, values: Vec<ScanAddress>) -> Result<usize, Box<dyn Error + Send + Sync>>;
}

impl ScanAddressesRepository for DatabaseClient {
    fn get_scan_address(&mut self, _chain: Chain, value: &str) -> Result<StorageScanAddress, Box<dyn Error + Send + Sync>> {
        Ok(ScanAddressesStore::get_scan_address(self, _chain, value)?)
    }

    fn get_scan_addresses(&mut self, queries: &[(Chain, &str)]) -> Result<Vec<StorageScanAddress>, Box<dyn Error + Send + Sync>> {
        Ok(ScanAddressesStore::get_scan_addresses(self, queries)?)
    }

    fn get_scan_addresses_by_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<StorageScanAddress>, Box<dyn Error + Send + Sync>> {
        Ok(ScanAddressesStore::get_scan_addresses_by_addresses(self, addresses)?)
    }

    fn add_scan_address_types(&mut self, values: Vec<ScanAddressType>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(ScanAddressesStore::add_scan_address_types(self, values)?)
    }

    fn add_scan_addresses(&mut self, values: Vec<ScanAddress>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let new_addresses = values
            .into_iter()
            .map(NewScanAddress::from_primitive)
            .collect();
        Ok(ScanAddressesStore::add_scan_addresses(self, new_addresses)?)
    }
}
