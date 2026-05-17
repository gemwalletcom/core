use gem_encoding::decode_base64;
use solana_primitives::VersionedTransaction;

pub fn try_decode_transaction(transaction_base64: &str) -> Option<VersionedTransaction> {
    let data = decode_base64(transaction_base64).ok()?;
    VersionedTransaction::deserialize_with_version(&data).ok()
}

pub fn decode_transaction(transaction_base64: &str) -> Result<VersionedTransaction, String> {
    try_decode_transaction(transaction_base64).ok_or_else(|| "failed to decode transaction".to_string())
}
