use alloy_primitives::{Address, Bytes};
use gem_evm::uniswap::{
    contracts::v4::{IV4Quoter::QuoteExactParams, PathKey, PoolKey},
    path::TokenPair,
    FeeTier,
};

use crate::swapper::{eth_address, uniswap::swap_route::RouteData, SwapRoute, SwapperError};

// return (currency0, currency1)
fn sort_addresses(token_in: &Address, token_out: &Address) -> (Address, Address) {
    if token_in.0 < token_out.0 {
        (*token_in, *token_out)
    } else {
        (*token_out, *token_in)
    }
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
                .map(|fee_tier| TokenPair::new_two_hop(token_in, intermediary, token_out, *fee_tier))
                .filter(|token_pairs| token_pairs.len() >= 2)
                .map(|token_pairs| {
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
                .collect()
        })
        .collect()
}

impl TryFrom<&SwapRoute> for PathKey {
    type Error = SwapperError;

    fn try_from(value: &SwapRoute) -> Result<Self, Self::Error> {
        let token_id = value.output.token_id.as_ref().ok_or(SwapperError::InvalidAddress(value.output.to_string()))?;
        let currency = eth_address::parse_str(token_id)?;

        let route_data: RouteData = serde_json::from_str(&value.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let fee_tier = FeeTier::try_from(route_data.fee_tier.as_str()).map_err(|_| SwapperError::InvalidAmount("invalid fee tier".into()))?;
        Ok(PathKey {
            intermediateCurrency: currency,
            fee: fee_tier.as_u24(),
            tickSpacing: fee_tier.default_tick_spacing(),
            hooks: Address::ZERO,
            hookData: Bytes::new(),
        })
    }
}
