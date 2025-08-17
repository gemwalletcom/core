use crate::models::fee::NearGasPrice;
use primitives::{FeePriority, FeePriorityValue};

pub fn map_gas_price_to_priorities(gas_price: &NearGasPrice) -> Result<Vec<FeePriorityValue>, Box<dyn std::error::Error + Sync + Send>> {
    let base_price = gas_price.gas_price.parse::<u64>()?;

    Ok(vec![
        FeePriorityValue::new(FeePriority::Slow, base_price.to_string()),
        FeePriorityValue::new(FeePriority::Normal, base_price.to_string()),
        FeePriorityValue::new(FeePriority::Fast, (base_price * 2).to_string()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::fee::NearGasPrice;

    #[test]
    fn test_map_gas_price_to_priorities() {
        let gas_price = NearGasPrice {
            gas_price: "1000000000".to_string(),
        };

        let result = map_gas_price_to_priorities(&gas_price).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].value, "1000000000");
        assert_eq!(result[1].value, "1000000000");
        assert_eq!(result[2].value, "2000000000");
    }
}
