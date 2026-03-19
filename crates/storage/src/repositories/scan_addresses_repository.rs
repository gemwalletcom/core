use crate::database::scan_addresses::ScanAddressesStore;
use crate::models::{NewScanAddressRow, ScanAddressRow};
use crate::sql_types::ChainRow;
use crate::{DatabaseClient, DatabaseError};
use primitives::{Chain, ScanAddress};
use std::collections::{HashMap, HashSet};

pub trait ScanAddressesRepository {
    fn get_scan_address(&mut self, _chain: Chain, value: &str) -> Result<ScanAddressRow, DatabaseError>;
    fn get_scan_addresses(&mut self, queries: &[(Chain, &str)]) -> Result<Vec<ScanAddressRow>, DatabaseError>;
    fn get_scan_addresses_by_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<ScanAddressRow>, DatabaseError>;
    fn add_scan_addresses(&mut self, values: Vec<ScanAddress>) -> Result<usize, DatabaseError>;
}

impl ScanAddressesRepository for DatabaseClient {
    fn get_scan_address(&mut self, chain: Chain, value: &str) -> Result<ScanAddressRow, DatabaseError> {
        let rows = ScanAddressesStore::get_scan_addresses_by_addresses(self, vec![value.to_string()])?;
        select_scan_address(chain, value, rows).ok_or_else(|| DatabaseError::not_found("ScanAddress", format!("{}/{}", chain.as_ref(), value)))
    }

    fn get_scan_addresses(&mut self, queries: &[(Chain, &str)]) -> Result<Vec<ScanAddressRow>, DatabaseError> {
        let addresses = queries.iter().map(|(_, address)| (*address).to_string()).collect::<HashSet<_>>().into_iter().collect();
        let rows = ScanAddressesStore::get_scan_addresses_by_addresses(self, addresses)?;

        Ok(select_scan_addresses(queries, rows))
    }

    fn get_scan_addresses_by_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<ScanAddressRow>, DatabaseError> {
        Ok(ScanAddressesStore::get_scan_addresses_by_addresses(self, addresses)?)
    }

    fn add_scan_addresses(&mut self, values: Vec<ScanAddress>) -> Result<usize, DatabaseError> {
        let new_addresses = values.into_iter().map(NewScanAddressRow::from_primitive).collect();
        Ok(ScanAddressesStore::add_scan_addresses(self, new_addresses)?)
    }
}

fn select_scan_address(chain: Chain, address: &str, rows: Vec<ScanAddressRow>) -> Option<ScanAddressRow> {
    let rows = rows.into_iter().filter(|row| row.address == address).collect::<Vec<_>>();
    select_scan_address_row(chain, &rows)
}

fn select_scan_addresses(queries: &[(Chain, &str)], rows: Vec<ScanAddressRow>) -> Vec<ScanAddressRow> {
    let mut rows_by_address = HashMap::<String, Vec<ScanAddressRow>>::new();
    for row in rows {
        rows_by_address.entry(row.address.clone()).or_default().push(row);
    }

    queries
        .iter()
        .filter_map(|(chain, address)| rows_by_address.get(*address).and_then(|rows| select_scan_address_row(*chain, rows)))
        .collect()
}

fn select_scan_address_row(chain: Chain, rows: &[ScanAddressRow]) -> Option<ScanAddressRow> {
    rows.iter().find(|row| row.chain.0 == chain).cloned().or_else(|| {
        rows.first().cloned().map(|mut row| {
            row.chain = ChainRow::from(chain);
            row
        })
    })
}

#[cfg(test)]
mod tests {
    use super::{select_scan_address, select_scan_addresses};
    use crate::models::ScanAddressRow;
    use primitives::Chain;

    #[test]
    fn test_select_scan_address_prefers_exact_chain_match() {
        let rows = vec![
            ScanAddressRow::mock(1, Chain::Ethereum, "0x123", Some("Ethereum")),
            ScanAddressRow::mock(2, Chain::Arbitrum, "0x123", Some("Arbitrum")),
        ];

        let result = select_scan_address(Chain::Arbitrum, "0x123", rows).unwrap();

        assert_eq!(result.chain.0, Chain::Arbitrum);
        assert_eq!(result.name, Some("Arbitrum".to_string()));
    }

    #[test]
    fn test_select_scan_address_falls_back_to_other_chain() {
        let rows = vec![ScanAddressRow::mock(1, Chain::Ethereum, "0x123", Some("1inch"))];

        let result = select_scan_address(Chain::Arbitrum, "0x123", rows).unwrap();

        assert_eq!(result.chain.0, Chain::Arbitrum);
        assert_eq!(result.name, Some("1inch".to_string()));
    }

    #[test]
    fn test_select_scan_addresses_resolves_each_query_independently() {
        let rows = vec![
            ScanAddressRow::mock(1, Chain::Ethereum, "0x123", Some("1inch")),
            ScanAddressRow::mock(2, Chain::Polygon, "0x456", Some("Polygon")),
            ScanAddressRow::mock(3, Chain::Arbitrum, "0x456", Some("Arbitrum")),
        ];
        let queries = vec![(Chain::Arbitrum, "0x123"), (Chain::Arbitrum, "0x456")];

        let result = select_scan_addresses(&queries, rows);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].chain.0, Chain::Arbitrum);
        assert_eq!(result[0].name, Some("1inch".to_string()));
        assert_eq!(result[1].chain.0, Chain::Arbitrum);
        assert_eq!(result[1].name, Some("Arbitrum".to_string()));
    }
}
