use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GasPriceType {
    Regular { gas_price: BigInt },
    Eip1559 { gas_price: BigInt, priority_fee: BigInt },
}

impl GasPriceType {
    pub fn regular<T: Into<BigInt>>(gas_price: T) -> Self {
        Self::Regular { gas_price: gas_price.into() }
    }

    pub fn eip1559<T: Into<BigInt>, U: Into<BigInt>>(gas_price: T, priority_fee: U) -> Self {
        Self::Eip1559 {
            gas_price: gas_price.into(),
            priority_fee: priority_fee.into(),
        }
    }

    pub fn gas_price(&self) -> BigInt {
        match self {
            GasPriceType::Regular { gas_price } => gas_price.clone(),
            GasPriceType::Eip1559 { gas_price, .. } => gas_price.clone(),
        }
    }

    pub fn priority_fee(&self) -> BigInt {
        match self {
            GasPriceType::Regular { .. } => BigInt::from(0),
            GasPriceType::Eip1559 { priority_fee, .. } => priority_fee.clone(),
        }
    }

    pub fn total_fee(&self) -> BigInt {
        self.gas_price() + self.priority_fee()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gas_price() {
        let regular = GasPriceType::regular(BigInt::from(1000u64));
        assert_eq!(regular.gas_price(), BigInt::from(1000u64));

        let eip1559 = GasPriceType::eip1559(BigInt::from(2000u64), BigInt::from(500u64));
        assert_eq!(eip1559.gas_price(), BigInt::from(2000u64));
    }

    #[test]
    fn priority_fee() {
        let regular = GasPriceType::regular(BigInt::from(1000u64));
        assert_eq!(regular.priority_fee(), BigInt::from(0));

        let eip1559 = GasPriceType::eip1559(BigInt::from(2000u64), BigInt::from(500u64));
        assert_eq!(eip1559.priority_fee(), BigInt::from(500u64));
    }

    #[test]
    fn total_fee() {
        let regular = GasPriceType::regular(BigInt::from(1000u64));
        assert_eq!(regular.total_fee(), BigInt::from(1000u64));

        let eip1559 = GasPriceType::eip1559(BigInt::from(2000u64), BigInt::from(500u64));
        assert_eq!(eip1559.total_fee(), BigInt::from(2500u64));
    }
}