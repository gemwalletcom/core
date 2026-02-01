use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, Chain, Transaction, WalletId, WebSocketPricePayload};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamMessagePrices {
    #[serde(default)]
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
    pub chain: Chain,
    pub address: String,
    pub asset_id: AssetId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct StreamTransactionsUpdate {
    pub wallet_id: WalletId,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub enum StreamEvent {
    Prices(WebSocketPricePayload),
    Balances(Vec<StreamBalanceUpdate>),
    Transactions(StreamTransactionsUpdate),
}
