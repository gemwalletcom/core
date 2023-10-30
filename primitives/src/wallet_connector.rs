#[typeshare(swift = "Equatable, Codable, Hashable")]
struct WalletConnection {
    session: WalletConnectionSession,
    wallet: Wallet,
}
#[derive(Debug, Serialize)]
#[typeshare(swift="Codable, Hashable")]
#[serde(rename_all = "lowercase")]
pub enum WalletConnectionState {
    Started,
    Active,
    Expired,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub enum WalletConnectionMethods {
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
}

#[derive(Debug, Serialize)]
#[typeshare(swift = "Codable, CaseIterable")]
pub enum WalletConnectionEvents {
    connect,
    disconnect,
    #[serde(rename = "accountsChanged")]
    accounts_changed,
    #[serde(rename = "chainChanged")]
    chain_changed,
}

#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
struct WalletConnectionSession {
    id: String,
    session_id: String,
    state: WalletConnectionState,
    chains: [Chain],
    created_at: DateTime<Utc>,
    expire_at: DateTime<Utc>,
    metadata: WalletConnectionSessionAppMetadata,
}

#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
struct WalletConnectionSessionAppMetadata {
    name: String,
    description: String,
    url: String,
    icon: String,
    redirect_native: Option<String>,
    redirect_universal: Option<String>,
}

#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
struct WalletConnectionSessionProposal {
    wallet: Wallet,
    accounts: [Account],
    metadata: WalletConnectionSessionAppMetadata,
}

#[derive(Debug, Serialize)]
#[typeshare]
struct SignMessage {
    #[serde(rename = "type")]
    sign_type: SignDigestType,
    data: Data,
}

#[derive(Debug, Serialize)]
#[typeshare]
#[serde(rename_all = "lowercase")]
pub enum SignDigestType {
    Sign,
    Eip191,
    Eip712,
}