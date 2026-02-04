use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, InAppNotification, TransactionId, WalletId, WebSocketPricePayload};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
#[allow(clippy::large_enum_variant)]
pub enum StreamEvent {
    Prices(WebSocketPricePayload),
    Balances(Vec<StreamBalanceUpdate>),
    Transactions(StreamTransactionsUpdate),
    PriceAlerts(StreamPriceAlertUpdate),
    Nft(StreamNftUpdate),
    Perpetual(StreamPerpetualUpdate),
    InAppNotification(StreamNotificationlUpdate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamMessagePrices {
    pub assets: Vec<AssetId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub enum StreamMessage {
    SubscribePrices(StreamMessagePrices),
    UnsubscribePrices(StreamMessagePrices),
    AddPrices(StreamMessagePrices),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamBalanceUpdate {
    pub wallet_id: WalletId,
    pub asset_id: AssetId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamTransactionsUpdate {
    pub wallet_id: WalletId,
    pub transactions: Vec<TransactionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamPriceAlertUpdate {
    pub assets: Vec<AssetId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamNftUpdate {
    pub wallet_id: WalletId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamPerpetualUpdate {
    pub wallet_id: WalletId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamNotificationlUpdate {
    pub wallet_id: WalletId,
    pub notification: InAppNotification,
}
