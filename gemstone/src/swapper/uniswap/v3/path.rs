use alloy_primitives::{Address, Bytes};
use gem_evm::uniswap::{
    path::{build_direct_pair, build_pairs, BasePair, TokenPair},
    FeeTier,
};

use crate::swapper::{
    eth_address,
    uniswap::swap_route::{get_intermediaries, RouteData},
    SwapperRoute, SwapperError,
};

pub fn build_paths(token_in: &Address, token_out: &Address, fee_tiers: &[FeeTier], base_pair: &BasePair) -> Vec<Vec<(Vec<TokenPair>, Bytes)>> {
    let mut paths: Vec<Vec<(Vec<TokenPair>, Bytes)>> = vec![];
    let direct_paths: Vec<_> = fee_tiers
        .iter()
        .map(|fee_tier| {
            (
                vec![TokenPair {
                    token_in: *token_in,
                    token_out: *token_out,
                    fee_tier: *fee_tier,
                }],
                build_direct_pair(token_in, token_out, *fee_tier),
            )
        })
        .collect();
    paths.push(direct_paths);

    let intermediaries = get_intermediaries(token_in, token_out, base_pair);
    intermediaries.iter().for_each(|intermediary| {
        let token_pairs: Vec<Vec<TokenPair>> = fee_tiers
            .iter()
            .map(|fee_tier| TokenPair::new_two_hop(token_in, intermediary, token_out, *fee_tier))
            .collect();
        let pair_paths: Vec<_> = token_pairs.iter().map(|token_pairs| (token_pairs.to_vec(), build_pairs(token_pairs))).collect();
        paths.push(pair_paths);
    });
    paths
}

pub fn build_paths_with_routes(routes: &[SwapperRoute]) -> Result<Bytes, SwapperError> {
    if routes.is_empty() {
        return Err(SwapperError::InvalidRoute);
    }
    let route_data: RouteData = serde_json::from_str(&routes[0].route_data).map_err(|_| SwapperError::InvalidRoute)?;
    let fee_tier = FeeTier::try_from(route_data.fee_tier.as_str()).map_err(|_| SwapperError::InvalidAmount("invalid fee tier".into()))?;
    let token_pairs: Vec<TokenPair> = routes
        .iter()
        .map(|route| TokenPair {
            token_in: eth_address::parse_asset_id(&route.input).unwrap(),
            token_out: eth_address::parse_asset_id(&route.output).unwrap(),
            fee_tier,
        })
        .collect();
    let paths = build_pairs(&token_pairs);
    Ok(paths)
}
