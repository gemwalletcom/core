use std::collections::HashMap;

use num_bigint::BigInt;
use primitives::{GasPriceType, TransactionFee, TransactionLoadMetadata, UTXO};

use crate::models::utxo::UTXO as CardanoUTXO;

pub(crate) fn map_transaction_preload(utxos: Vec<CardanoUTXO>, block_number: u64) -> TransactionLoadMetadata {
    TransactionLoadMetadata::Cardano {
        utxos: utxos.into_iter().map(UTXO::from).collect(),
        block_number,
    }
}

pub(crate) fn map_transaction_fee(fee: u64) -> TransactionFee {
    TransactionFee::new_gas_price_type(GasPriceType::regular(BigInt::from(1u64)), BigInt::from(fee), BigInt::from(1u64), HashMap::new())
}
