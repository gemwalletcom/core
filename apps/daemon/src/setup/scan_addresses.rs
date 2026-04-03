use gem_evm::{
    across::deployment::AcrossDeployment,
    uniswap::deployment::{
        get_uniswap_permit2_by_chain,
        v3::{
            get_aerodrome_router_deployment_by_chain, get_oku_deployment_by_chain, get_pancakeswap_router_deployment_by_chain, get_uniswap_router_deployment_by_chain,
            get_wagmi_router_deployment_by_chain,
        },
        v4::get_uniswap_deployment_by_chain,
    },
};
use gem_tracing::info_with_fields;
use primitives::{Chain, ScanAddress, SwapProvider};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use storage::{Database, ScanAddressesRepository};

pub fn setup_scan_addresses(database: &Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut values = HashMap::new();

    for chain in Chain::all() {
        let uniswap_permit2 = get_uniswap_permit2_by_chain(&chain);
        let uniswap_v3 = get_uniswap_router_deployment_by_chain(&chain);
        let uniswap_v4 = get_uniswap_deployment_by_chain(&chain);
        let pancakeswap = get_pancakeswap_router_deployment_by_chain(&chain);
        let oku = get_oku_deployment_by_chain(&chain);
        let wagmi = get_wagmi_router_deployment_by_chain(&chain);
        let aerodrome = get_aerodrome_router_deployment_by_chain(&chain);

        if let Some(address) = uniswap_permit2 {
            values
                .entry((chain, address.to_string()))
                .or_insert_with(|| ScanAddress::contract(chain, address, SwapProvider::UniswapV3.name()));
        }

        for (provider, address) in [
            (SwapProvider::UniswapV3, uniswap_v3.as_ref().map(|deployment| deployment.universal_router)),
            (SwapProvider::UniswapV4, uniswap_v4.as_ref().map(|deployment| deployment.universal_router)),
            (SwapProvider::PancakeswapV3, pancakeswap.as_ref().map(|deployment| deployment.universal_router)),
            (SwapProvider::Oku, oku.as_ref().map(|deployment| deployment.universal_router)),
            (SwapProvider::Wagmi, wagmi.as_ref().map(|deployment| deployment.universal_router)),
            (SwapProvider::Aerodrome, aerodrome.as_ref().map(|deployment| deployment.universal_router)),
        ] {
            if let Some(address) = address {
                values
                    .entry((chain, address.to_string()))
                    .or_insert_with(|| ScanAddress::contract(chain, address, provider.name()));
            }
        }

        for (provider, address) in [
            (SwapProvider::PancakeswapV3, pancakeswap.as_ref().map(|deployment| deployment.permit2)),
            (SwapProvider::Oku, oku.as_ref().map(|deployment| deployment.permit2)),
            (SwapProvider::Wagmi, wagmi.as_ref().map(|deployment| deployment.permit2)),
        ] {
            if let Some(address) = address {
                values
                    .entry((chain, address.to_string()))
                    .or_insert_with(|| ScanAddress::contract(chain, address, provider.name()));
            }
        }

        if let Some(deployment) = AcrossDeployment::deployment_by_chain(&chain) {
            values
                .entry((chain, deployment.spoke_pool.to_string()))
                .or_insert_with(|| ScanAddress::contract(chain, deployment.spoke_pool, SwapProvider::Across.name()));
        }
    }

    let count = values.len();
    let addresses = values.keys().map(|(_, address)| address.clone()).collect();
    let existing = database
        .scan_addresses()?
        .get_scan_addresses_by_addresses(addresses)?
        .into_iter()
        .map(|row| (row.chain.0, row.address))
        .collect::<HashSet<_>>();
    let values = values
        .into_iter()
        .filter_map(|(key, value)| (!existing.contains(&key)).then_some(value))
        .collect::<Vec<_>>();
    let inserted = if values.is_empty() { 0 } else { database.scan_addresses()?.add_scan_addresses(values)? };

    info_with_fields!("setup", step = "scan addresses", count = count, inserted = inserted);
    Ok(())
}
