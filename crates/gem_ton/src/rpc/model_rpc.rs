use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonWallet {
    pub balance: String,
    pub jetton: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetsBalances {
    pub jetton_wallets: Vec<JettonWallet>,
}
