use crate::{Chain, Wallet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub struct WalletConnection {
    pub session: WalletConnectionSession,
    pub wallet: Wallet,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum WalletConnectionState {
    Started,
    Active,
    Expired,
}

#[derive(Debug, Serialize)]
#[typeshare(swift = "CaseIterable, Sendable")]
pub enum WalletConnectionMethods {
    #[serde(rename = "eth_chainId")]
    EthChainId,
    #[serde(rename = "personal_sign")]
    PersonalSign,
    #[serde(rename = "eth_signTypedData")]
    EthSignTypedData,
    #[serde(rename = "eth_signTypedData_v4")]
    EthSignTypedDataV4,
    #[serde(rename = "eth_signTransaction")]
    EthSignTransaction,
    #[serde(rename = "eth_sendTransaction")]
    EthSendTransaction,
    #[serde(rename = "eth_sendRawTransaction")]
    EthSendRawTransaction,
    #[serde(rename = "wallet_switchEthereumChain")]
    WalletSwitchEthereumChain,
    #[serde(rename = "wallet_addEthereumChain")]
    WalletAddEthereumChain,
    #[serde(rename = "solana_signMessage")]
    SolanaSignMessage,
    #[serde(rename = "solana_signTransaction")]
    SolanaSignTransaction,
    #[serde(rename = "solana_signAndSendTransaction")]
    SolanaSignAndSendTransaction,
    #[serde(rename = "solana_signAllTransactions")]
    SolanaSignAllTransactions,
    #[serde(rename = "sui_signPersonalMessage")]
    SuiSignPersonalMessage,
    #[serde(rename = "sui_signTransaction")]
    SuiSignTransaction,
    #[serde(rename = "sui_signAndExecuteTransaction")]
    SuiSignAndExecuteTransaction,
}

#[derive(Debug, Serialize)]
#[typeshare(swift = "CaseIterable, Sendable")]
pub enum WalletConnectionEvents {
    #[serde(rename = "connect")]
    Connect,
    #[serde(rename = "disconnect")]
    Disconnect,
    #[serde(rename = "accountsChanged")]
    AccountsChanged,
    #[serde(rename = "chainChanged")]
    ChainChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct WalletConnectionSession {
    pub id: String,
    pub session_id: String,
    pub state: WalletConnectionState,
    pub chains: Vec<Chain>,
    pub created_at: DateTime<Utc>,
    pub expire_at: DateTime<Utc>,
    pub metadata: WalletConnectionSessionAppMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct WalletConnectionSessionAppMetadata {
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct WalletConnectionSessionProposal {
    pub default_wallet: Wallet,
    pub wallets: Vec<Wallet>,
    pub metadata: WalletConnectionSessionAppMetadata,
}
