mod asset;
mod chain;
mod client;
mod model;
mod provider;

use chain::THORChainName;
use num_bigint::BigInt;
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct ThorChain {}

const QUOTE_MINIMUM: i64 = 0;
const QUOTE_INTERVAL: i64 = 1;
const QUOTE_QUANTITY: i64 = 0;
const DEFAULT_DEPOSIT_GAS_LIMIT: u64 = 80_000;

impl ThorChain {
    fn data(&self, chain: THORChainName, memo: String) -> String {
        if chain.is_evm_chain() {
            return hex::encode(memo.as_bytes());
        }
        memo
    }

    fn value_from(&self, value: String, decimals: i32) -> BigInt {
        let decimals = decimals - 8;
        if decimals > 0 {
            BigInt::from_str(value.as_str()).unwrap() / BigInt::from(10).pow(decimals as u32)
        } else {
            BigInt::from_str(value.as_str()).unwrap() * BigInt::from(10).pow(decimals.unsigned_abs())
        }
    }

    fn value_to(&self, value: String, decimals: i32) -> BigInt {
        let decimals = decimals - 8;
        if decimals > 0 {
            BigInt::from_str(value.as_str()).unwrap() * BigInt::from(10).pow((decimals).unsigned_abs())
        } else {
            BigInt::from_str(value.as_str()).unwrap() / BigInt::from(10).pow((decimals).unsigned_abs())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data() {
        let thorchain = ThorChain::default();
        let memo = "test".to_string();

        let result = thorchain.data(THORChainName::Ethereum, memo.clone());
        assert_eq!(result, hex::encode(memo.as_bytes()));

        let result = thorchain.data(THORChainName::Bitcoin, memo.clone());
        assert_eq!(result, memo);
    }

    #[test]
    fn test_value_from() {
        let thorchain = ThorChain::default();

        let value = "1000000000".to_string();

        let result = thorchain.value_from(value.clone(), 18);
        assert_eq!(result, BigInt::from_str("0").unwrap());

        let result = thorchain.value_from(value.clone(), 10);
        assert_eq!(result, BigInt::from_str("10000000").unwrap());

        let result = thorchain.value_from(value.clone(), 6);
        assert_eq!(result, BigInt::from_str("100000000000").unwrap());

        let result = thorchain.value_from(value.clone(), 8);
        assert_eq!(result, BigInt::from(1000000000));
    }

    #[test]
    fn test_value_to() {
        let thorchain = ThorChain::default();

        let value = "10000000".to_string();

        let result = thorchain.value_to(value.clone(), 18);
        assert_eq!(result, BigInt::from_str("100000000000000000").unwrap());

        let result = thorchain.value_to(value.clone(), 10);
        assert_eq!(result, BigInt::from(1000000000));

        let result = thorchain.value_to(value.clone(), 6);
        assert_eq!(result, BigInt::from(100000));

        let result = thorchain.value_to(value.clone(), 8);
        assert_eq!(result, BigInt::from(10000000));
    }
}
