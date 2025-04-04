use alloy_primitives::Bytes;
use gem_evm::{
    address::EthereumAddress,
    uniswap::{
        path::{build_direct_pair, build_pairs, BasePair, TokenPair},
        FeeTier,
    },
};

use crate::swapper::{
    uniswap::swap_route::{get_intermediaries, RouteData},
    SwapRoute, SwapperError,
};

pub fn build_paths(token_in: &EthereumAddress, token_out: &EthereumAddress, fee_tiers: &[FeeTier], base_pair: &BasePair) -> Vec<Vec<(Vec<TokenPair>, Bytes)>> {
    let mut paths: Vec<Vec<(Vec<TokenPair>, Bytes)>> = vec![];
    let direct_paths: Vec<_> = fee_tiers
        .iter()
        .map(|fee_tier| {
            (
                vec![TokenPair {
                    token_in: token_in.clone(),
                    token_out: token_out.clone(),
                    fee_tier: fee_tier.clone(),
                }],
                build_direct_pair(token_in, token_out, fee_tier.clone() as u32),
            )
        })
        .collect();
    paths.push(direct_paths);

    let intermediaries = get_intermediaries(token_in, token_out, base_pair);
    intermediaries.iter().for_each(|intermediary| {
        let token_pairs: Vec<Vec<TokenPair>> = fee_tiers
            .iter()
            .map(|fee_tier| TokenPair::new_two_hop(token_in, intermediary, token_out, fee_tier))
            .collect();
        let pair_paths: Vec<_> = token_pairs.iter().map(|token_pairs| (token_pairs.to_vec(), build_pairs(token_pairs))).collect();
        paths.push(pair_paths);
    });
    paths
}

pub fn build_paths_with_routes(routes: &[SwapRoute]) -> Result<Bytes, SwapperError> {
    if routes.is_empty() {
        return Err(SwapperError::InvalidRoute);
    }
    let route_data: RouteData = serde_json::from_str(&routes[0].route_data).map_err(|_| SwapperError::InvalidRoute)?;
    let fee_tier = FeeTier::try_from(route_data.fee_tier.as_str()).map_err(|_| SwapperError::InvalidAmount("invalid fee tier".into()))?;
    let token_pairs: Vec<TokenPair> = routes
        .iter()
        .map(|route| TokenPair {
            token_in: route.input.clone().into(),
            token_out: route.output.clone().into(),
            fee_tier: fee_tier.clone(),
        })
        .collect();
    let paths = build_pairs(&token_pairs);
    Ok(paths)
}
