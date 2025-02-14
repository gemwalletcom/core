use super::super::swap_route::get_intermediaries;
use alloy_core::primitives::Address;
use alloy_primitives::Bytes;
use gem_evm::{
    address::EthereumAddress,
    uniswap::{
        contracts::v4::{PathKey, PoolKey},
        path::{BasePair, TokenPair},
        FeeTier,
    },
};

// return (currency0, currency1)
fn sort_addresses(token_in: &EthereumAddress, token_out: &EthereumAddress) -> (Address, Address) {
    if token_in.bytes < token_out.bytes {
        (Address::from_slice(&token_in.bytes), Address::from_slice(&token_out.bytes))
    } else {
        (Address::from_slice(&token_out.bytes), Address::from_slice(&token_in.bytes))
    }
}

pub fn build_pool_keys(token_in: &EthereumAddress, token_out: &EthereumAddress, fee_tiers: &[FeeTier]) -> Vec<(Vec<TokenPair>, PoolKey)> {
    let (currency0, currency1) = sort_addresses(token_in, token_out);

    fee_tiers
        .iter()
        .map(|fee_tier| {
            let fee = fee_tier.as_u24();
            let tick_spacing = fee_tier.default_tick_spacing();
            let pool_key = PoolKey {
                currency0,
                currency1,
                fee,
                tickSpacing: tick_spacing,
                hooks: Address::ZERO,
            };
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

#[allow(unused)]
pub fn build_path_keys(
    token_in: &EthereumAddress,
    token_out: &EthereumAddress,
    fee_tiers: &[FeeTier],
    base_pair: &BasePair,
) -> Vec<(Vec<TokenPair>, Vec<PathKey>)> {
    let mut result: Vec<(Vec<TokenPair>, Vec<PathKey>)> = vec![];
    let intermediaries = get_intermediaries(token_in, token_out, base_pair);
    intermediaries.iter().for_each(|intermediary| {
        let array: Vec<Vec<TokenPair>> = fee_tiers
            .iter()
            .map(|fee_tier| TokenPair::new_two_hop(token_in, intermediary, token_out, fee_tier))
            .collect();

        for token_pairs in array.iter().skip(1) {
            let path_keys: Vec<PathKey> = token_pairs
                .iter()
                .map(|token_pair| PathKey {
                    intermediateCurrency: Address::from_slice(&intermediary.bytes),
                    fee: token_pair.fee_tier.as_u24(),
                    tickSpacing: token_pair.fee_tier.default_tick_spacing(),
                    hooks: Address::ZERO,
                    hookData: Bytes::new(),
                })
                .collect();
            result.push((token_pairs.clone(), path_keys));
        }
    });
    result
}
