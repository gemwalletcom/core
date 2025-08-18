use serde::{Deserialize, Serialize};

use super::transaction::TonTransactionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonWalletInfo {
    pub seqno: Option<i32>,
    pub last_transaction_id: TonTransactionId,
}
