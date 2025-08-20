use num_bigint::BigInt;
use primitives::{chain_cosmos::CosmosChain, FeePriority, FeeRate};

pub fn calculate_fee_rates(chain: CosmosChain, base_fee: BigInt) -> Vec<FeeRate> {
    match chain {
        CosmosChain::Thorchain => {
            vec![FeeRate::regular(FeePriority::Normal, base_fee)]
        }
        CosmosChain::Cosmos | CosmosChain::Osmosis | CosmosChain::Celestia | CosmosChain::Sei | CosmosChain::Injective | CosmosChain::Noble => {
            vec![
                FeeRate::regular(FeePriority::Normal, base_fee.clone()),
                FeeRate::regular(FeePriority::Fast, &base_fee * BigInt::from(2)),
            ]
        }
    }
}
