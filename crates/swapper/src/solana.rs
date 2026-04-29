use crate::SwapperError;
use gem_solana::decode_transaction;

pub use gem_solana::DEFAULT_SWAP_GAS_LIMIT;

pub fn gas_limit_from_transaction(tx_base64: &str) -> Result<u64, SwapperError> {
    let transaction = decode_transaction(tx_base64).map_err(SwapperError::TransactionError)?;

    Ok(u64::from(transaction.get_compute_unit_limit().unwrap_or(DEFAULT_SWAP_GAS_LIMIT)))
}
