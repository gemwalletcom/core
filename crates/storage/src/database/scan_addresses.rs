use crate::{models::*, DatabaseClient};
use primitives::Chain;

use diesel::prelude::*;

impl DatabaseClient {
    pub fn get_scan_address(&mut self, _chain: Chain, value: &str) -> Result<ScanAddress, diesel::result::Error> {
        use crate::schema::scan_addresses::dsl::*;
        scan_addresses
            .filter(chain.eq(_chain.as_ref()))
            .filter(address.eq(value))
            .select(ScanAddress::as_select())
            .first(&mut self.connection)
    }

    pub fn get_scan_addresses(&mut self, queries: &[(Chain, &str)]) -> Result<Vec<ScanAddress>, diesel::result::Error> {
        use crate::schema::scan_addresses::dsl::*;
        use diesel::prelude::*;
        let conditions = queries
            .iter()
            .map(|(chain_value, address_value)| (chain.eq(chain_value.as_ref()), address.eq(address_value)));

        let mut query = scan_addresses.into_boxed();
        for (chain_filter, address_filter) in conditions {
            query = query.or_filter(chain_filter.and(address_filter));
        }

        query.select(ScanAddress::as_select()).load(&mut self.connection) // Returns a Vec<ScanAddress>
    }

    pub fn add_scan_address_types(&mut self, values: Vec<ScanAddressType>) -> Result<usize, diesel::result::Error> {
        use crate::schema::scan_addresses_types::dsl::*;
        diesel::insert_into(scan_addresses_types)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}
