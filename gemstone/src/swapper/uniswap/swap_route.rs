use crate::swapper::SwapRoute;
use gem_evm::{address::EthereumAddress, uniswap::path::BasePair};
use primitives::AssetId;

pub fn get_intermediaries(token_in: &EthereumAddress, token_out: &EthereumAddress, base_pair: &BasePair) -> Vec<EthereumAddress> {
    let array = base_pair.to_array();
    get_intermediaries_by_array(token_in, token_out, &array)
}

pub fn get_intermediaries_by_array(token_in: &EthereumAddress, token_out: &EthereumAddress, array: &[EthereumAddress]) -> Vec<EthereumAddress> {
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
    fee_tier: &str,
    gas_estimate: Option<String>,
) -> Vec<SwapRoute> {
    if let Some(intermediary) = intermediary {
        vec![
            SwapRoute {
                input: token_in.clone(),
                output: intermediary.clone(),
                route_data: fee_tier.to_string(),
                gas_limit: gas_estimate.clone(),
            },
            SwapRoute {
                input: intermediary.clone(),
                output: token_out.clone(),
                route_data: fee_tier.to_string(),
                gas_limit: None,
            },
        ]
    } else {
        vec![SwapRoute {
            input: token_in.clone(),
            output: token_out.clone(),
            route_data: fee_tier.to_string(),
            gas_limit: gas_estimate.clone(),
        }]
    }
}
