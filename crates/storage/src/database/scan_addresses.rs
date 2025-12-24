use crate::{DatabaseClient, models::*};
use primitives::Chain;

use diesel::prelude::*;

pub(crate) trait ScanAddressesStore {
    fn get_scan_address(&mut self, _chain: Chain, value: &str) -> Result<ScanAddressRow, diesel::result::Error>;
    fn get_scan_addresses(&mut self, queries: &[(Chain, &str)]) -> Result<Vec<ScanAddressRow>, diesel::result::Error>;
    fn get_scan_addresses_by_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<ScanAddressRow>, diesel::result::Error>;
    fn add_scan_address_types(&mut self, values: Vec<ScanAddressTypeRow>) -> Result<usize, diesel::result::Error>;
    fn add_scan_addresses(&mut self, values: Vec<NewScanAddressRow>) -> Result<usize, diesel::result::Error>;
}

impl ScanAddressesStore for DatabaseClient {
    fn get_scan_address(&mut self, _chain: Chain, value: &str) -> Result<ScanAddressRow, diesel::result::Error> {
        use crate::schema::scan_addresses::dsl::*;
        scan_addresses
            .filter(chain.eq(_chain.as_ref()))
            .filter(address.eq(value))
            .select(ScanAddressRow::as_select())
            .first(&mut self.connection)
    }

    fn get_scan_addresses(&mut self, queries: &[(Chain, &str)]) -> Result<Vec<ScanAddressRow>, diesel::result::Error> {
        use crate::schema::scan_addresses::dsl::*;
        use diesel::prelude::*;
        let conditions = queries
            .iter()
            .map(|(chain_value, address_value)| (chain.eq(chain_value.as_ref()), address.eq(address_value)));

        let mut query = scan_addresses.into_boxed();
        for (chain_filter, address_filter) in conditions {
            query = query.or_filter(chain_filter.and(address_filter));
        }

        query.select(ScanAddressRow::as_select()).load(&mut self.connection) // Returns a Vec<ScanAddressRow>
    }

    fn get_scan_addresses_by_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<ScanAddressRow>, diesel::result::Error> {
        use crate::schema::scan_addresses::dsl::*;
        scan_addresses
            .filter(address.eq_any(addresses))
            .select(ScanAddressRow::as_select())
            .load(&mut self.connection)
    }

    fn add_scan_address_types(&mut self, values: Vec<ScanAddressTypeRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::scan_addresses_types::dsl::*;
        diesel::insert_into(scan_addresses_types)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    fn add_scan_addresses(&mut self, values: Vec<NewScanAddressRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::scan_addresses::dsl::*;
        diesel::insert_into(scan_addresses)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }
}
