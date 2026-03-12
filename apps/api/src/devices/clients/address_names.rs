use std::error::Error;

use primitives::{AddressName, ChainAddress};
use storage::{Database, ScanAddressesRepository};

#[derive(Clone)]
pub struct AddressNamesClient {
    database: Database,
}

impl AddressNamesClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn get_address_names(&self, requests: Vec<ChainAddress>) -> Result<Vec<AddressName>, Box<dyn Error + Send + Sync>> {
        let requests = requests.into_iter().filter(|request| !request.address.is_empty()).collect::<Vec<_>>();

        if requests.is_empty() {
            return Ok(vec![]);
        }

        let queries = requests.iter().map(|request| (request.chain, request.address.as_str())).collect::<Vec<_>>();

        Ok(self
            .database
            .scan_addresses()?
            .get_scan_addresses(&queries)?
            .into_iter()
            .filter_map(|x| x.as_primitive())
            .collect())
    }
}
