use crate::{GasPriceType, TransactionFee};

impl TransactionFee {
    pub fn mock_eip1559(gas_limit: u64) -> Self {
        TransactionFee::new_gas_price_type(
            GasPriceType::eip1559(20_000_000_000u64, 1_000_000_000u64),
            (20_000_000_000u64 * gas_limit).into(),
            gas_limit.into(),
            Default::default(),
        )
    }
}
