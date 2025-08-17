use primitives::{FeePriorityValue, FeePriority};
use crate::rpc::model::TransactionParams;

pub fn map_transaction_params_to_fee(params: &TransactionParams) -> FeePriorityValue {
    FeePriorityValue {
        priority: FeePriority::Normal,
        value: params.min_fee.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::TransactionParams;

    #[test]
    fn test_map_transaction_params_to_fee() {
        let params = TransactionParams {
            min_fee: 1000,
            genesis_id: "mainnet-v1.0".to_string(),
            last_round: 12345,
        };
        
        let result = map_transaction_params_to_fee(&params);
        assert_eq!(result.value, "1000");
        assert_eq!(result.priority, FeePriority::Normal);
    }
}