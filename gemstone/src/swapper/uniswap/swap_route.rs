use crate::swapper::SwapperRoute;
use alloy_primitives::Address;
use gem_evm::uniswap::path::BasePair;
use primitives::AssetId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteData {
    pub fee_tier: String,
    pub min_amount_out: String,
}

pub fn get_intermediaries(token_in: &Address, token_out: &Address, base_pair: &BasePair) -> Vec<Address> {
    let array = base_pair.path_building_array();
    get_intermediaries_by_array(token_in, token_out, &array)
}

pub fn get_intermediaries_by_array(token_in: &Address, token_out: &Address, array: &[Address]) -> Vec<Address> {
    array
        .iter()
        .filter(|intermediary| *intermediary != token_in && *intermediary != token_out)
        .cloned()
        .collect()
}

pub fn build_swap_route(
    token_in: &AssetId,
    intermediary: Option<&AssetId>,
    token_out: &AssetId,
    route_data: &RouteData,
    gas_estimate: Option<String>,
) -> Vec<SwapperRoute> {
    let data = serde_json::to_string(route_data).unwrap();
    if let Some(intermediary) = intermediary {
        vec![
            SwapperRoute {
                input: token_in.clone(),
                output: intermediary.clone(),
                route_data: data.clone(),
                gas_limit: gas_estimate.clone(),
            },
            SwapperRoute {
                input: intermediary.clone(),
                output: token_out.clone(),
                route_data: data,
                gas_limit: None,
            },
        ]
    } else {
        vec![SwapperRoute {
            input: token_in.clone(),
            output: token_out.clone(),
            route_data: data,
            gas_limit: gas_estimate.clone(),
        }]
    }
}
