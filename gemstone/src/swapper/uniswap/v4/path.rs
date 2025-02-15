use alloy_core::primitives::Address;
use alloy_primitives::Bytes;
use gem_evm::{
    address::EthereumAddress,
    uniswap::{
        contracts::v4::{IV4Quoter::QuoteExactParams, PathKey, PoolKey},
        path::{TokenPair, TokenPairs},
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

pub fn build_quote_exact_params(
    amount_in: u128,
    token_in: &EthereumAddress,
    token_out: &EthereumAddress,
    fee_tiers: &[FeeTier],
    intermediaries: &[EthereumAddress],
) -> Vec<Vec<(Vec<TokenPair>, QuoteExactParams)>> {
    let mut results: Vec<Vec<(Vec<TokenPair>, QuoteExactParams)>> = vec![];
    intermediaries.iter().for_each(|intermediary| {
        let mut result: Vec<(Vec<TokenPair>, QuoteExactParams)> = vec![];
        let array: Vec<Vec<TokenPair>> = fee_tiers
            .iter()
            .map(|fee_tier| TokenPair::new_two_hop(token_in, intermediary, token_out, fee_tier))
            .collect();

        array.iter().for_each(|token_pairs| {
            println!("token_pairs: {:}", TokenPairs(token_pairs.clone()));

            if token_pairs.len() < 2 {
                return;
            }

            let mut quote_exact_params = QuoteExactParams {
                exactCurrency: Address::ZERO,
                path: vec![],
                exactAmount: amount_in,
            };
            for (idx, token_pair) in token_pairs.iter().enumerate() {
                if idx == 0 {
                    quote_exact_params.exactCurrency = Address::from_slice(&token_pair.token_in.bytes);
                }
                let path_key = PathKey {
                    intermediateCurrency: Address::from_slice(&token_pair.token_out.bytes),
                    fee: token_pair.fee_tier.as_u24(),
                    tickSpacing: token_pair.fee_tier.default_tick_spacing(),
                    hooks: Address::ZERO,
                    hookData: Bytes::new(),
                };
                quote_exact_params.path.push(path_key);
            }
            result.push((token_pairs.clone(), quote_exact_params));
        });
        results.push(result);
    });
    results
}
