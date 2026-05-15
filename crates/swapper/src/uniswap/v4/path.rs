use alloy_primitives::{Address, Bytes};
use gem_evm::uniswap::{
    FeeTier,
    contracts::v4::{IV4Quoter::QuoteExactParams, PathKey, PoolKey},
    path::TokenPair,
};

use crate::{Route, SwapperError, error::INVALID_ADDRESS, eth_address, uniswap::swap_route::RouteData};

// return (currency0, currency1)
fn sort_addresses(token_in: &Address, token_out: &Address) -> (Address, Address) {
    if token_in.0 < token_out.0 { (*token_in, *token_out) } else { (*token_out, *token_in) }
}

pub fn build_pool_key(token_in: &Address, token_out: &Address, fee_tier: &FeeTier) -> (PoolKey, bool) {
    let (currency0, currency1) = sort_addresses(token_in, token_out);
    let zero_for_one = currency0.0 == token_in.0;
    let fee = fee_tier.as_u24();
    let tick_spacing = fee_tier.default_tick_spacing();
    (
        PoolKey {
            currency0,
            currency1,
            fee,
            tickSpacing: tick_spacing,
            hooks: Address::ZERO,
        },
        zero_for_one,
    )
}

pub fn build_pool_keys(token_in: &Address, token_out: &Address, fee_tiers: &[FeeTier]) -> Vec<(Vec<TokenPair>, PoolKey)> {
    fee_tiers
        .iter()
        .map(|fee_tier| {
            let (pool_key, _) = build_pool_key(token_in, token_out, fee_tier);
            (
                vec![TokenPair {
                    token_in: *token_in,
                    token_out: *token_out,
                    fee_tier: *fee_tier,
                }],
                pool_key,
            )
        })
        .collect()
}

pub fn build_quote_exact_params(
    amount_in: u128,
    token_in: &Address,
    token_out: &Address,
    fee_tiers: &[FeeTier],
    intermediaries: &[Address],
) -> Vec<Vec<(Vec<TokenPair>, QuoteExactParams)>> {
    intermediaries
        .iter()
        .map(|intermediary| {
            fee_tiers
                .iter()
                .flat_map(|first_fee_tier| {
                    fee_tiers.iter().map(move |second_fee_tier| {
                        let token_pairs = TokenPair::new_two_hop_with_fees(token_in, intermediary, token_out, *first_fee_tier, *second_fee_tier);
                        let quote_exact_params = QuoteExactParams {
                            exactCurrency: token_pairs[0].token_in,
                            path: token_pairs
                                .iter()
                                .map(|token_pair| PathKey {
                                    intermediateCurrency: token_pair.token_out,
                                    fee: token_pair.fee_tier.as_u24(),
                                    tickSpacing: token_pair.fee_tier.default_tick_spacing(),
                                    hooks: Address::ZERO,
                                    hookData: Bytes::new(),
                                })
                                .collect(),
                            exactAmount: amount_in,
                        };

                        (token_pairs, quote_exact_params)
                    })
                })
                .collect()
        })
        .collect()
}

impl TryFrom<&Route> for PathKey {
    type Error = SwapperError;

    fn try_from(value: &Route) -> Result<Self, Self::Error> {
        let token_id = value
            .output
            .token_id
            .as_ref()
            .ok_or_else(|| SwapperError::ComputeQuoteError(format!("{}: {}", INVALID_ADDRESS, value.output)))?;
        let currency = eth_address::parse_str(token_id)?;

        let route_data: RouteData = serde_json::from_str(&value.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let fee_tier = FeeTier::try_from(route_data.fee_tier.as_str()).map_err(|_| SwapperError::ComputeQuoteError("invalid fee tier".into()))?;
        Ok(PathKey {
            intermediateCurrency: currency,
            fee: fee_tier.as_u24(),
            tickSpacing: fee_tier.default_tick_spacing(),
            hooks: Address::ZERO,
            hookData: Bytes::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uniswap::swap_route::build_swap_route;
    use alloy_primitives::address;
    use primitives::Chain;

    #[test]
    fn test_build_quote_exact_params_uses_mixed_fee_two_hop() {
        let token_in = address!("0x1111111111111111111111111111111111111111");
        let intermediary = address!("0x2222222222222222222222222222222222222222");
        let token_out = address!("0x3333333333333333333333333333333333333333");
        let fee_tiers = vec![FeeTier::Hundred, FeeTier::ThreeThousand];

        let params = build_quote_exact_params(100, &token_in, &token_out, &fee_tiers, &[intermediary]);

        assert_eq!(params.len(), 1);
        assert_eq!(params[0].len(), 4);
        let fees: Vec<(FeeTier, FeeTier)> = params[0].iter().map(|param| (param.0[0].fee_tier, param.0[1].fee_tier)).collect();
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
    fn test_path_key_from_route_uses_per_hop_fee_tier() {
        let token_in = address!("0x1111111111111111111111111111111111111111");
        let intermediary = address!("0x2222222222222222222222222222222222222222");
        let token_out = address!("0x3333333333333333333333333333333333333333");
        let token_pairs = TokenPair::new_two_hop_with_fees(&token_in, &intermediary, &token_out, FeeTier::Hundred, FeeTier::ThreeThousand);
        let routes = build_swap_route(Chain::Optimism, &token_pairs, "100").unwrap();

        let first = PathKey::try_from(&routes[0]).unwrap();
        let second = PathKey::try_from(&routes[1]).unwrap();

        assert_eq!(first.fee, FeeTier::Hundred.as_u24());
        assert_eq!(second.fee, FeeTier::ThreeThousand.as_u24());
    }
}
