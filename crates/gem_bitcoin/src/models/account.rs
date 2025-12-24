use num_bigint::{BigInt, BigUint};
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_bigint_from_str, deserialize_biguint_from_str};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitcoinAccount {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub balance: BigUint,
    #[serde(default, deserialize_with = "deserialize_bigint_from_str")]
    pub unconfirmed_balance: BigInt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_positive_unconfirmed() {
        let json = r#"{"balance": "16097910", "unconfirmedBalance": "10000000"}"#;
        let account: BitcoinAccount = serde_json::from_str(json).unwrap();
        assert_eq!(account.balance, BigUint::from(16097910_u64));
        assert_eq!(account.unconfirmed_balance, BigInt::from(10000000_i64));
    }

    #[test]
    fn test_deserialize_negative_unconfirmed() {
        let json = r#"{"balance": "20000000", "unconfirmedBalance": "-10001045"}"#;
        let account: BitcoinAccount = serde_json::from_str(json).unwrap();
        assert_eq!(account.balance, BigUint::from(20000000_u64));
        assert_eq!(account.unconfirmed_balance, BigInt::from(-10001045_i64));
    }
}
