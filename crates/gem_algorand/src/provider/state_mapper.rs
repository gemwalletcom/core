use crate::models::rpc::TransactionsParams;
use num_bigint::BigInt;
use primitives::{FeePriority, FeeRate, GasPriceType};

pub fn map_transaction_params_to_fee(params: &TransactionsParams) -> FeeRate {
    FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(params.min_fee)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_transaction_params_to_fee() {
        let params = TransactionsParams {
            min_fee: 1000,
            genesis_id: "mainnet-v1.0".to_string(),
            genesis_hash: "hash".to_string(),
            last_round: 12345,
        };

        let result = map_transaction_params_to_fee(&params);
        assert_eq!(result.priority, FeePriority::Normal);

        match result.gas_price_type {
            GasPriceType::Regular { ref gas_price } => assert_eq!(*gas_price, BigInt::from(1000)),
            _ => panic!("Expected Regular gas price type"),
        }
    }
}
