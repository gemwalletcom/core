use crate::{Route, SwapperError};
use alloy_primitives::Address;
use gem_evm::uniswap::path::{BasePair, TokenPair};
use primitives::{AssetId, Chain};
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

pub fn build_swap_route(chain: Chain, token_pairs: &[TokenPair], min_amount_out: &str) -> Result<Vec<Route>, SwapperError> {
    if token_pairs.is_empty() {
        return Err(SwapperError::InvalidRoute);
    }

    token_pairs
        .iter()
        .map(|pair| {
            let route_data = RouteData {
                fee_tier: (pair.fee_tier as u32).to_string(),
                min_amount_out: min_amount_out.to_string(),
            };
            Ok(Route {
                input: AssetId::from(chain, Some(pair.token_in.to_checksum(None))),
                output: AssetId::from(chain, Some(pair.token_out.to_checksum(None))),
                route_data: serde_json::to_string(&route_data).map_err(|_| SwapperError::InvalidRoute)?,
            })
        })
        .collect()
}
