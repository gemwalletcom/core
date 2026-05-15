use alloy_primitives::{Address, Bytes};
use gem_evm::uniswap::{
    FeeTier,
    path::{BasePair, TokenPair, build_pairs, validate_pairs},
};

use crate::{
    Route, SwapperError, eth_address,
    uniswap::swap_route::{RouteData, get_intermediaries},
};

fn path_for_pairs(token_pairs: Vec<TokenPair>) -> (Vec<TokenPair>, Bytes) {
    let path = build_pairs(&token_pairs);
    (token_pairs, path)
}

pub fn build_paths(token_in: &Address, token_out: &Address, fee_tiers: &[FeeTier], base_pair: &BasePair) -> Vec<Vec<(Vec<TokenPair>, Bytes)>> {
    let direct_paths: Vec<_> = fee_tiers
        .iter()
        .map(|fee_tier| {
            path_for_pairs(vec![TokenPair {
                token_in: *token_in,
                token_out: *token_out,
                fee_tier: *fee_tier,
            }])
        })
        .collect();

    let intermediaries = get_intermediaries(token_in, token_out, base_pair);
    std::iter::once(direct_paths)
        .chain(intermediaries.iter().map(|intermediary| {
            fee_tiers
                .iter()
                .flat_map(|first_fee_tier| {
                    fee_tiers
                        .iter()
                        .map(move |second_fee_tier| path_for_pairs(TokenPair::new_two_hop_with_fees(token_in, intermediary, token_out, *first_fee_tier, *second_fee_tier)))
                })
                .collect()
        }))
        .collect()
}

pub fn build_paths_with_routes(routes: &[Route]) -> Result<Bytes, SwapperError> {
    if routes.is_empty() {
        return Err(SwapperError::InvalidRoute);
    }
    let token_pairs: Vec<TokenPair> = routes
        .iter()
        .map(|route| {
            let route_data: RouteData = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
            let fee_tier = FeeTier::try_from(route_data.fee_tier.as_str()).map_err(|_| SwapperError::ComputeQuoteError("invalid fee tier".into()))?;
            Ok(TokenPair {
                token_in: eth_address::parse_asset_id(&route.input)?,
                token_out: eth_address::parse_asset_id(&route.output)?,
                fee_tier,
            })
        })
        .collect::<Result<Vec<_>, SwapperError>>()?;
    if !validate_pairs(&token_pairs) {
        return Err(SwapperError::InvalidRoute);
    }
    let paths = build_pairs(&token_pairs);
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uniswap::swap_route::build_swap_route;
    use alloy_primitives::{address, hex::encode_prefixed as HexEncode};
    use primitives::Chain;

    #[test]
    fn test_build_paths_uses_mixed_fee_two_hop() {
        let token_in = address!("0x1111111111111111111111111111111111111111");
        let intermediary = address!("0x2222222222222222222222222222222222222222");
        let token_out = address!("0x3333333333333333333333333333333333333333");
        let fee_tiers = vec![FeeTier::Hundred, FeeTier::ThreeThousand];
        let base_pair = BasePair {
            native: intermediary,
            stables: vec![],
            alternatives: vec![],
        };

        let paths = build_paths(&token_in, &token_out, &fee_tiers, &base_pair);

        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].len(), 2);
        assert_eq!(paths[1].len(), 4);
        let fees: Vec<(FeeTier, FeeTier)> = paths[1].iter().map(|path| (path.0[0].fee_tier, path.0[1].fee_tier)).collect();
        assert_eq!(
            fees,
            vec![
                (FeeTier::Hundred, FeeTier::Hundred),
                (FeeTier::Hundred, FeeTier::ThreeThousand),
                (FeeTier::ThreeThousand, FeeTier::Hundred),
                (FeeTier::ThreeThousand, FeeTier::ThreeThousand),
            ]
        );
    }

    #[test]
    fn test_build_paths_with_routes_uses_per_hop_fee_tiers() {
        let token_in = address!("0x1111111111111111111111111111111111111111");
        let intermediary = address!("0x2222222222222222222222222222222222222222");
        let token_out = address!("0x3333333333333333333333333333333333333333");
        let token_pairs = TokenPair::new_two_hop_with_fees(&token_in, &intermediary, &token_out, FeeTier::Hundred, FeeTier::ThreeThousand);
        let routes = build_swap_route(Chain::Optimism, &token_pairs, "100").unwrap();

        let path = build_paths_with_routes(&routes).unwrap();

        assert_eq!(HexEncode(path), HexEncode(build_pairs(&token_pairs)));
    }
}
