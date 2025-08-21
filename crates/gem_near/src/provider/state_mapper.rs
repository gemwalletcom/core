use crate::models::fee::NearGasPrice;
use primitives::{FeePriority, FeeRate};

pub fn map_gas_price_to_priorities(gas_price: &NearGasPrice) -> Result<Vec<FeeRate>, Box<dyn std::error::Error + Sync + Send>> {
    let base_price = gas_price.gas_price.parse::<u64>()?;

    Ok(vec![
        FeeRate::regular(FeePriority::Slow, base_price),
        FeeRate::regular(FeePriority::Normal, base_price),
        FeeRate::regular(FeePriority::Fast, base_price * 2),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::fee::NearGasPrice;
    use num_bigint::BigInt;
    use primitives::GasPriceType;

    #[test]
    fn test_map_gas_price_to_priorities() {
        let gas_price = NearGasPrice {
            gas_price: "1000000000".to_string(),
        };

        let result = map_gas_price_to_priorities(&gas_price).unwrap();
        assert_eq!(result.len(), 3);
        match &result[0].gas_price_type {
            GasPriceType::Regular { gas_price } => assert_eq!(gas_price, &BigInt::from(1000000000u64)),
            _ => panic!("Expected Regular gas price"),
        }
        match &result[1].gas_price_type {
            GasPriceType::Regular { gas_price } => assert_eq!(gas_price, &BigInt::from(1000000000u64)),
            _ => panic!("Expected Regular gas price"),
        }
        match &result[2].gas_price_type {
            GasPriceType::Regular { gas_price } => assert_eq!(gas_price, &BigInt::from(2000000000u64)),
            _ => panic!("Expected Regular gas price"),
        }
    }
}
