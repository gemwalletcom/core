use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::ton_transaction::TonTransactionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonWalletInfo {
    pub seqno: Option<i32>,
    pub last_transaction_id: TonTransactionId,
}