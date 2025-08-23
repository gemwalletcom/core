use num_bigint::BigInt;
use primitives::{chain_cosmos::CosmosChain, GasPriceType, StakeType, TransactionFee, TransactionInputType};

fn get_fee(chain: CosmosChain, input_type: &TransactionInputType) -> BigInt {
    match chain {
        CosmosChain::Thorchain => BigInt::from(2_000_000u64),
        CosmosChain::Cosmos => match input_type {
            TransactionInputType::Transfer(_) | TransactionInputType::Deposit(_) | TransactionInputType::TokenApprove(_, _) | TransactionInputType::Generic(_, _, _) | TransactionInputType::Perpetual(_, _) => BigInt::from(3_000u64),
            TransactionInputType::Swap(_, _) => BigInt::from(3_000u64),
            TransactionInputType::Stake(_, _) => BigInt::from(25_000u64),
        },
        CosmosChain::Osmosis => match input_type {
            TransactionInputType::Transfer(_) | TransactionInputType::Deposit(_) | TransactionInputType::TokenApprove(_, _) | TransactionInputType::Generic(_, _, _) | TransactionInputType::Perpetual(_, _) => BigInt::from(10_000u64),
            TransactionInputType::Swap(_, _) => BigInt::from(10_000u64),
            TransactionInputType::Stake(_, _) => BigInt::from(100_000u64),
        },
        CosmosChain::Celestia => match input_type {
            TransactionInputType::Transfer(_) | TransactionInputType::Deposit(_) | TransactionInputType::TokenApprove(_, _) | TransactionInputType::Generic(_, _, _) | TransactionInputType::Perpetual(_, _) => BigInt::from(3_000u64),
            TransactionInputType::Swap(_, _) => BigInt::from(3_000u64),
            TransactionInputType::Stake(_, _) => BigInt::from(10_000u64),
        },
        CosmosChain::Sei => match input_type {
            TransactionInputType::Transfer(_) | TransactionInputType::Deposit(_) | TransactionInputType::TokenApprove(_, _) | TransactionInputType::Generic(_, _, _) | TransactionInputType::Perpetual(_, _) => BigInt::from(100_000u64),
            TransactionInputType::Swap(_, _) => BigInt::from(100_000u64),
            TransactionInputType::Stake(_, _) => BigInt::from(200_000u64),
        },
        CosmosChain::Injective => match input_type {
            TransactionInputType::Transfer(_) | TransactionInputType::Deposit(_) | TransactionInputType::TokenApprove(_, _) | TransactionInputType::Generic(_, _, _) | TransactionInputType::Perpetual(_, _) => BigInt::from(100_000_000_000_000u64),
            TransactionInputType::Swap(_, _) => BigInt::from(100_000_000_000_000u64),
            TransactionInputType::Stake(_, _) => BigInt::from(1_000_000_000_000_000u64),
        },
        CosmosChain::Noble => BigInt::from(25_000u64),
    }
}

fn get_gas_limit(input_type: &TransactionInputType, _chain: CosmosChain) -> u64 {
    match input_type {
        TransactionInputType::Transfer(_) | TransactionInputType::Deposit(_) | TransactionInputType::TokenApprove(_, _) | TransactionInputType::Generic(_, _, _) | TransactionInputType::Perpetual(_, _) => 200_000,
        TransactionInputType::Swap(_, _) => 200_000,
        TransactionInputType::Stake(_, operation) => match operation {
            StakeType::Stake(_) | StakeType::Unstake(_) => 1_000_000,
            StakeType::Redelegate(_) => 1_250_000,
            StakeType::Rewards(_) => 750_000,
            StakeType::Withdraw(_) => 750_000,
        },
    }
}

pub fn calculate_transaction_fee(input_type: &TransactionInputType, chain: CosmosChain, gas_price_type: &GasPriceType) -> TransactionFee {
    let gas_limit = get_gas_limit(input_type, chain);
    let fee = get_fee(chain, input_type);

    TransactionFee {
        fee,
        gas_price_type: gas_price_type.clone(),
        gas_limit: BigInt::from(gas_limit),
        options: std::collections::HashMap::new(),
    }
}
