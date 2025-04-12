#[typeshare(swift = "Equatable, Hashable, Sendable")]
struct WalletConnection {
    session: WalletConnectionSession,
    wallet: Wallet,
}
#[derive(Debug, Serialize)]
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
    eth_chain_id,
    eth_sign,
    personal_sign,
    #[serde(rename = "eth_signTypedData")]
    eth_sign_typed_data,
    #[serde(rename = "eth_signTypedData_v4")]
    eth_sign_typed_data_v4,
    #[serde(rename = "eth_signTransaction")]
    eth_sign_transaction,
    #[serde(rename = "eth_sendTransaction")]
    eth_send_transaction,
    #[serde(rename = "eth_sendRawTransaction")]
    eth_send_raw_transaction,
    #[serde(rename = "wallet_switchEthereumChain")]
    wallet_switch_ethereum_chain,
    #[serde(rename = "wallet_addEthereumChain")]
    wallet_add_ethereum_chain,
    #[serde(rename = "solana_signMessage")]
    solana_sign_message,
    #[serde(rename = "solana_signTransaction")]
    solana_sign_transaction,
    #[serde(rename = "solana_signAndSendTransaction")]
    solana_sign_and_send_transaction,
}

#[derive(Debug, Serialize)]
#[typeshare(swift = "CaseIterable, Sendable")]
pub enum WalletConnectionEvents {
    connect,
    disconnect,
    #[serde(rename = "accountsChanged")]
    accounts_changed,
    #[serde(rename = "chainChanged")]
    chain_changed,
}

#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
struct WalletConnectionSession {
    id: String,
    session_id: String,
    state: WalletConnectionState,
    chains: [Chain],
    created_at: NaiveDateTime,
    expire_at: NaiveDateTime,
    metadata: WalletConnectionSessionAppMetadata,
}

#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
struct WalletConnectionSessionAppMetadata {
    name: String,
    description: String,
    url: String,
    icon: String,
    redirect_native: Option<String>,
    redirect_universal: Option<String>,
}

#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
struct WalletConnectionSessionProposal {
    default_wallet: Wallet,
    wallets: [Wallet],
    metadata: WalletConnectionSessionAppMetadata,
}

#[derive(Debug, Serialize)]
#[typeshare(swift = "Sendable")]
struct SignMessage {
    #[serde(rename = "type")]
    sign_type: SignDigestType,
    data: Data,
}

#[derive(Debug, Serialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SignDigestType {
    Sign,
    Eip191,
    Eip712,
    Base58,
}
