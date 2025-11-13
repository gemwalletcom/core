use primitives::{AssetId, ScanAddressTarget, ScanTransaction, ScanTransactionPayload, TransactionType};

pub type GemScanTransaction = ScanTransaction;
pub type GemScanTransactionPayload = ScanTransactionPayload;
pub type GemScanAddressTarget = ScanAddressTarget;

#[uniffi::remote(Record)]
pub struct ScanTransaction {
    pub is_malicious: bool,
    pub is_memo_required: bool,
}

#[uniffi::remote(Record)]
pub struct ScanTransactionPayload {
    pub origin: ScanAddressTarget,
    pub target: ScanAddressTarget,
    pub website: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
}

#[uniffi::remote(Record)]
pub struct ScanAddressTarget {
    pub asset_id: AssetId,
    pub address: String,
}

