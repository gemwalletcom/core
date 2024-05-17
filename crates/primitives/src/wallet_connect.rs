use serde::Serialize;
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Serialize, AsRefStr, EnumString)]
#[typeshare]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum WallletConnectCAIP2 {
    #[serde(rename = "eip155")]
    Eip155,
    #[serde(rename = "solana")]
    Solana,
    #[serde(rename = "cosmos")]
    Cosmos,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
struct WCEthereumTransaction {
    chain_id: Option<String>,
    from: String,
    to: String,
    value: Option<String>,
    gas: Option<String>,
    gas_limit: Option<String>,
    gas_price: Option<String>,
    max_fee_per_gas: Option<String>,
    max_priority_fee_per_gas: Option<String>,
    nonce: Option<String>,
    data: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
struct WCSolanaTransaction {
    transaction: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
struct WCSolanaSignMessage {
    message: String,
    pubkey: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
struct WCSolanaSignMessageResult {
    signature: String,
}
