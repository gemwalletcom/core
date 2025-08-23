use crate::GasPriceType;
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::{AsRefStr, Display, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, Display, EnumString, PartialEq, Eq, Hash)]
pub enum FeeOption {
    TokenAccountCreation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionFee {
    pub fee: BigInt,
    pub gas_price_type: GasPriceType,
    pub gas_limit: BigInt,
    pub options: HashMap<FeeOption, BigInt>,
}

impl Default for TransactionFee {
    fn default() -> Self {
        Self {
            fee: BigInt::from(0),
            gas_price_type: GasPriceType::regular(BigInt::from(0)),
            gas_limit: BigInt::from(0),
            options: HashMap::new(),
        }
    }
}

impl TransactionFee {
    pub fn new_from_fee(fee: BigInt) -> Self {
        Self {
            fee: fee.clone(),
            gas_price_type: GasPriceType::regular(fee),
            gas_limit: BigInt::from(0),
            options: HashMap::new(),
        }
    }

    pub fn new_from_gas_price_and_limit(gas_price: BigInt, gas_limit: BigInt) -> Self {
        Self {
            fee: gas_price.clone() * &gas_limit,
            gas_price_type: GasPriceType::regular(gas_price),
            gas_limit,
            options: HashMap::new(),
        }
    }

    pub fn new_from_fee_with_option(fee: BigInt, option: FeeOption, option_value: BigInt) -> Self {
        Self {
            fee: fee.clone(),
            gas_price_type: GasPriceType::regular(fee.clone()),
            gas_limit: BigInt::from(0),
            options: HashMap::from([(option, option_value)]),
        }
    }

    pub fn new_gas_price_type(gas_price_type: GasPriceType, gas_limit: BigInt, options: HashMap<FeeOption, BigInt>) -> Self {
        Self {
            fee: gas_price_type.total_fee() + options.values().sum::<BigInt>(),
            gas_price_type,
            gas_limit,
            options,
        }
    }

    pub fn calculate(gas_limit: u64, gas_price_type: &GasPriceType) -> Self {
        let gas_limit = BigInt::from(gas_limit);
        let gas_price = gas_price_type.gas_price();
        let total_fee = gas_price.clone() * &gas_limit;

        Self {
            fee: total_fee,
            gas_price_type: gas_price_type.clone(),
            gas_limit,
            options: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_fee_calculate() {
        let gas_price_type = GasPriceType::regular(BigInt::from(100u64));
        let gas_limit = 1000u64;

        let fee = TransactionFee::calculate(gas_limit, &gas_price_type);

        assert_eq!(fee.fee, BigInt::from(100000u64)); // 100 * 1000
        assert_eq!(fee.gas_price_type.gas_price(), BigInt::from(100u64));
        assert_eq!(fee.gas_limit, BigInt::from(1000u64));
    }
}
