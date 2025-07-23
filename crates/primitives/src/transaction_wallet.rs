use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Transaction, Wallet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
struct TransactionWallet {
    pub transaction: Transaction,
    pub wallet: Wallet,
}
