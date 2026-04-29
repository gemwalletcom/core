use gem_encoding::decode_base64;
use solana_primitives::VersionedTransaction;

pub fn try_decode_transaction(tx_base64: &str) -> Option<VersionedTransaction> {
    let data = decode_base64(tx_base64).ok()?;
    VersionedTransaction::deserialize_with_version(&data).ok()
}

pub fn decode_transaction(tx_base64: &str) -> Result<VersionedTransaction, String> {
    try_decode_transaction(tx_base64).ok_or_else(|| "failed to decode transaction".to_string())
}

#[cfg(all(test, feature = "signer"))]
mod tests {
    use super::*;
    use crate::signer::testkit::SINGLE_SIG_TX;

    #[test]
    fn test_decode_transaction_compute_unit_limit() {
        let transaction = decode_transaction(SINGLE_SIG_TX).unwrap();

        assert_eq!(transaction.get_compute_unit_limit(), Some(1_400_000));
    }
}
