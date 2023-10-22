#[typeshare(swift = "Equatable, Codable")]
struct WalletConnection {
    session: WalletConnectionSession,
    wallet: Wallet,
}
#[derive(Debug, Serialize)]
#[typeshare(swift="Codable")]
#[serde(rename_all = "lowercase")]
pub enum WalletConnectionState {
    Started,
    Active,
    Expired,
}

#[typeshare(swift = "Equatable, Codable")]
#[serde(rename_all = "camelCase")]
struct WalletConnectionSession {
    id: String,
    session_id: String,
    state: WalletConnectionState,
    created_at: DateTime<Utc>,
    expire_at: DateTime<Utc>,
    app_name: String,
    app_description: String,
    app_url: String,
    app_icon: String,
    redirect_native: Option<String>,
    redirect_universal: Option<String>,
}