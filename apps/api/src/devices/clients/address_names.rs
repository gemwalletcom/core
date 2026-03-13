use std::collections::{HashMap, HashSet};
use std::error::Error;

use primitives::{AddressName, AddressType, Asset, AssetId, ChainAddress, VerificationStatus};
use storage::{AssetsRepository, Database, ScanAddressesRepository};

#[derive(Clone)]
pub struct AddressNamesClient {
    database: Database,
}

impl AddressNamesClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn get_address_names(&self, requests: Vec<ChainAddress>) -> Result<Vec<AddressName>, Box<dyn Error + Send + Sync>> {
        let requests: Vec<ChainAddress> = requests.into_iter().filter(|request| !request.address.is_empty()).collect();
        if requests.is_empty() {
            return Ok(vec![]);
        }

        let queries = requests.iter().map(|request| (request.chain, request.address.as_str())).collect::<Vec<_>>();
        let scan_names = self
            .database
            .scan_addresses()?
            .get_scan_addresses(&queries)?
            .into_iter()
            .filter_map(|row| row.as_primitive())
            .map(|name| (ChainAddress::new(name.chain, name.address.clone()), name))
            .collect::<HashMap<_, _>>();
        let asset_ids = requests
            .iter()
            .map(|request| AssetId::from(request.chain, Some(request.address.clone())).to_string())
            .collect::<Vec<_>>();
        let asset_names = self
            .database
            .assets()?
            .get_assets(asset_ids)?
            .into_iter()
            .filter_map(asset_entry)
            .collect::<HashMap<_, _>>();

        Ok(map_requests(requests, &scan_names, &asset_names))
    }
}

fn map_requests(requests: Vec<ChainAddress>, scan_names: &HashMap<ChainAddress, AddressName>, asset_names: &HashMap<ChainAddress, AddressName>) -> Vec<AddressName> {
    requests
        .into_iter()
        .filter_map(|request| asset_names.get(&request).or_else(|| scan_names.get(&request)).cloned())
        .scan(HashSet::new(), |seen, name| {
            seen.insert(ChainAddress::new(name.chain, name.address.clone())).then_some(name)
        })
        .collect()
}

fn asset_entry(asset: Asset) -> Option<(ChainAddress, AddressName)> {
    let address = asset.token_id?;

    Some((
        ChainAddress::new(asset.chain, address.clone()),
        AddressName {
            chain: asset.chain,
            address,
            name: asset.name,
            address_type: Some(AddressType::Contract),
            status: VerificationStatus::Verified,
        },
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::map_requests;
    use primitives::{AddressName, AddressType, Chain, ChainAddress, VerificationStatus};

    #[test]
    fn test_map_requests_prefers_asset_then_scan() {
        let asset_request = ChainAddress::new(Chain::Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string());
        let scan_request = ChainAddress::new(Chain::Ethereum, "0x123".to_string());
        let asset_name = AddressName::mock("0xdAC17F958D2ee523a2206206994597C13D831ec7", "USDT", AddressType::Contract, VerificationStatus::Verified);
        let scan_name = AddressName::mock("0x123", "Legacy Name", AddressType::Address, VerificationStatus::Unverified);

        let scan_names = HashMap::from([(asset_request.clone(), scan_name.clone()), (scan_request.clone(), scan_name.clone())]);
        let asset_names = HashMap::from([(asset_request.clone(), asset_name.clone())]);

        assert_eq!(map_requests(vec![asset_request, scan_request], &scan_names, &asset_names), vec![asset_name, scan_name]);
    }
}
