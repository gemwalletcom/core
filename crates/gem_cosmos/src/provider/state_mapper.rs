use primitives::{FeePriorityValue, FeePriority, chain_cosmos::CosmosChain};
use num_bigint::BigInt;

pub fn calculate_fee_rates(chain: CosmosChain, base_fee: BigInt) -> Vec<FeePriorityValue> {
    match chain {
        CosmosChain::Thorchain => {
            vec![FeePriorityValue {
                priority: FeePriority::Normal,
                value: base_fee.to_string(),
            }]
        }
        CosmosChain::Cosmos | 
        CosmosChain::Osmosis | 
        CosmosChain::Celestia | 
        CosmosChain::Sei | 
        CosmosChain::Injective | 
        CosmosChain::Noble => {
            vec![
                FeePriorityValue {
                    priority: FeePriority::Normal,
                    value: base_fee.to_string(),
                },
                FeePriorityValue {
                    priority: FeePriority::Fast,
                    value: (&base_fee * BigInt::from(2)).to_string(),
                },
            ]
        }
    }
}