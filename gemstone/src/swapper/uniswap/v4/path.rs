use alloy_core::primitives::Address;
use alloy_primitives::Bytes;
use gem_evm::{
    address::EthereumAddress,
    uniswap::{
        contracts::v4::{IV4Quoter::QuoteExactParams, PathKey, PoolKey},
        path::TokenPair,
        FeeTier,
    },
};

use crate::swapper::{SwapRoute, SwapperError};

// return (currency0, currency1)
fn sort_addresses(token_in: &EthereumAddress, token_out: &EthereumAddress) -> (Address, Address) {
    if token_in.bytes < token_out.bytes {
        (Address::from_slice(&token_in.bytes), Address::from_slice(&token_out.bytes))
    } else {
        (Address::from_slice(&token_out.bytes), Address::from_slice(&token_in.bytes))
    }
}

pub fn build_pool_key(token_in: &EthereumAddress, token_out: &EthereumAddress, fee_tier: &FeeTier) -> (PoolKey, bool) {
    let (currency0, currency1) = sort_addresses(token_in, token_out);
    let zero_for_one = currency0 == Address::from_slice(&token_in.bytes);
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

pub fn build_pool_keys(token_in: &EthereumAddress, token_out: &EthereumAddress, fee_tiers: &[FeeTier]) -> Vec<(Vec<TokenPair>, PoolKey)> {
    fee_tiers
        .iter()
        .map(|fee_tier| {
            let (pool_key, _) = build_pool_key(token_in, token_out, fee_tier);
            (
                vec![TokenPair {
                    token_in: token_in.clone(),
                    token_out: token_out.clone(),
                    fee_tier: fee_tier.clone(),
                }],
                pool_key,
            )
        })
        .collect()
}

pub fn build_quote_exact_params(
    amount_in: u128,
    token_in: &EthereumAddress,
    token_out: &EthereumAddress,
    fee_tiers: &[FeeTier],
    intermediaries: &[EthereumAddress],
) -> Vec<Vec<(Vec<TokenPair>, QuoteExactParams)>> {
    intermediaries
        .iter()
        .map(|intermediary| {
            fee_tiers
                .iter()
                .map(|fee_tier| TokenPair::new_two_hop(token_in, intermediary, token_out, fee_tier))
                .filter(|token_pairs| token_pairs.len() >= 2)
                .map(|token_pairs| {
                    let quote_exact_params = QuoteExactParams {
                        exactCurrency: Address::from_slice(&token_pairs[0].token_in.bytes),
                        path: token_pairs
                            .iter()
                            .map(|token_pair| PathKey {
                                intermediateCurrency: Address::from_slice(&token_pair.token_out.bytes),
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
        let token_id = value.output.token_id.as_ref().ok_or(SwapperError::InvalidAddress {
            address: value.output.to_string(),
        })?;
        let currency = token_id
            .parse::<Address>()
            .map_err(|_| SwapperError::InvalidAddress { address: token_id.clone() })?;
        let fee_tier = FeeTier::try_from(value.route_data.as_str()).map_err(|_| SwapperError::InvalidRoute)?;
        Ok(PathKey {
            intermediateCurrency: currency,
            fee: fee_tier.as_u24(),
            tickSpacing: fee_tier.default_tick_spacing(),
            hooks: Address::ZERO,
            hookData: Bytes::new(),
        })
    }
}
