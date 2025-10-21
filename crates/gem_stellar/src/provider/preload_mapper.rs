use num_bigint::BigInt;
use primitives::{FeeOption, TransactionLoadData, TransactionLoadInput};
use std::error::Error;

pub fn map_transaction_load(input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Send + Sync>> {
    let fee = if input.metadata.get_is_destination_address_exist()? {
        input.default_fee()
    } else {
        primitives::TransactionFee::new_from_fee_with_option(input.gas_price.gas_price(), FeeOption::TokenAccountCreation, BigInt::ZERO)
    };

    Ok(TransactionLoadData { fee, metadata: input.metadata })
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Asset, Chain, GasPriceType, TransactionInputType, TransactionLoadMetadata};

    #[test]
    fn test_map_transaction_load_destination_exists() {
        let input = TransactionLoadInput {
            metadata: TransactionLoadMetadata::Stellar {
                sequence: 1,
                is_destination_address_exist: true,
            },
            ..TransactionLoadInput::mock()
        };

        let result = map_transaction_load(input).unwrap();

        assert_eq!(result.fee.fee, BigInt::from(1000));
        assert!(result.fee.options.is_empty());
    }

    #[test]
    fn test_map_transaction_load_destination_not_exist() {
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Stellar)),
            value: "15000000".to_string(),
            gas_price: GasPriceType::regular(BigInt::from(100)),
            metadata: TransactionLoadMetadata::Stellar {
                sequence: 1,
                is_destination_address_exist: false,
            },
            ..TransactionLoadInput::mock()
        };

        let result = map_transaction_load(input).unwrap();

        assert_eq!(result.fee.fee, BigInt::from(100));
        assert!(result.fee.options.contains_key(&FeeOption::TokenAccountCreation));
        assert_eq!(result.fee.options.get(&FeeOption::TokenAccountCreation), Some(&BigInt::ZERO));
    }
}
