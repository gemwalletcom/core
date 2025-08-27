use num_bigint::BigInt;
use primitives::{chain_cosmos::CosmosChain, FeePriority, FeeRate, GasPriceType};

pub fn calculate_fee_rates(chain: CosmosChain, base_fee: BigInt) -> Vec<FeeRate> {
    match chain {
        CosmosChain::Thorchain => {
            vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(base_fee))]
        }
        CosmosChain::Cosmos | CosmosChain::Osmosis | CosmosChain::Celestia | CosmosChain::Sei | CosmosChain::Injective | CosmosChain::Noble => {
            vec![
                FeeRate::new(FeePriority::Normal, GasPriceType::regular(base_fee.clone())),
                FeeRate::new(FeePriority::Fast, GasPriceType::regular(&base_fee * BigInt::from(2))),
            ]
        }
    }
}
