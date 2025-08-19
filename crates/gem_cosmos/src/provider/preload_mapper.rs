use primitives::{TransactionInputType, TransactionFee, StakeOperation, chain_cosmos::CosmosChain};
use num_bigint::BigInt;

fn get_fee(chain: CosmosChain, input_type: &TransactionInputType) -> BigInt {
    match chain {
        CosmosChain::Thorchain => BigInt::from(2_000_000u64),
        CosmosChain::Cosmos => match input_type {
            TransactionInputType::Transfer(_) => BigInt::from(3_000u64),
            TransactionInputType::Swap(_, _) => BigInt::from(3_000u64),
            TransactionInputType::Stake(_) => BigInt::from(25_000u64),
        },
        CosmosChain::Osmosis => match input_type {
            TransactionInputType::Transfer(_) => BigInt::from(10_000u64),
            TransactionInputType::Swap(_, _) => BigInt::from(10_000u64),
            TransactionInputType::Stake(_) => BigInt::from(100_000u64),
        },
        CosmosChain::Celestia => match input_type {
            TransactionInputType::Transfer(_) => BigInt::from(3_000u64),
            TransactionInputType::Swap(_, _) => BigInt::from(3_000u64),
            TransactionInputType::Stake(_) => BigInt::from(10_000u64),
        },
        CosmosChain::Sei => match input_type {
            TransactionInputType::Transfer(_) => BigInt::from(100_000u64),
            TransactionInputType::Swap(_, _) => BigInt::from(100_000u64),
            TransactionInputType::Stake(_) => BigInt::from(200_000u64),
        },
        CosmosChain::Injective => match input_type {
            TransactionInputType::Transfer(_) => BigInt::from(100_000_000_000_000u64),
            TransactionInputType::Swap(_, _) => BigInt::from(100_000_000_000_000u64),
            TransactionInputType::Stake(_) => BigInt::from(1_000_000_000_000_000u64),
        },
        CosmosChain::Noble => BigInt::from(25_000u64),
    }
}

fn get_gas_limit(input_type: &TransactionInputType, _chain: CosmosChain) -> u64 {
    match input_type {
        TransactionInputType::Transfer(_) => 200_000,
        TransactionInputType::Swap(_, _) => 200_000,
        TransactionInputType::Stake(operation) => match operation {
            StakeOperation::Delegate(_, _) | StakeOperation::Undelegate(_, _) => 1_000_000,
            StakeOperation::Redelegate(_, _, _) => 1_250_000,
            StakeOperation::WithdrawRewards(_) => 750_000,
        }
    }
}

pub fn calculate_transaction_fee(input_type: &TransactionInputType, chain: CosmosChain, gas_price: &primitives::GasPrice) -> TransactionFee {
    let gas_limit = get_gas_limit(input_type, chain);
    let fee = get_fee(chain, input_type);
    
    TransactionFee {
        fee,
        gas_price: gas_price.gas_price.clone(),
        gas_limit: BigInt::from(gas_limit),
    }
}