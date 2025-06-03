pub mod consumer;
pub mod stream_producer;
pub mod stream_reader;

pub use consumer::run_consumer;
pub use consumer::ConsumerConfig;
use primitives::{AssetId, Chain, GorushNotification, Transaction};
use serde::{Deserialize, Serialize};
use std::fmt;
pub use stream_producer::StreamProducer;
pub use stream_reader::StreamReader;

#[derive(Debug, Clone)]
pub enum ExchangeName {
    Transactions,
}

impl fmt::Display for ExchangeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExchangeName::Transactions => write!(f, "transactions"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum QueueName {
    Transactions,
    NotificationsPriceAlerts,
    NotificationsTransactions,
    // fetch new assets and blocks
    FetchAssets,
    FetchBlocks,
}

impl fmt::Display for QueueName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueName::Transactions => write!(f, "transactions"),
            QueueName::NotificationsPriceAlerts => write!(f, "notifications_price_alerts"),
            QueueName::NotificationsTransactions => write!(f, "notifications_transactions"),
            QueueName::FetchAssets => write!(f, "fetch_assets"),
            QueueName::FetchBlocks => write!(f, "fetch_blocks"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsPayload {
    pub chain: Chain,
    pub blocks: Vec<i64>,
    pub transactions: Vec<Transaction>,
}

impl TransactionsPayload {
    pub fn new(chain: Chain, blocks: Vec<i64>, transactions: Vec<Transaction>) -> Self {
        Self { chain, blocks, transactions }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsPayload {
    pub notifications: Vec<GorushNotification>,
}

impl NotificationsPayload {
    pub fn new(notifications: Vec<GorushNotification>) -> Self {
        Self { notifications }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchAssetsPayload {
    pub asset_ids: Vec<AssetId>,
}

impl FetchAssetsPayload {
    pub fn new(asset_ids: Vec<AssetId>) -> Self {
        Self { asset_ids }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchBlocksPayload {
    pub chain: Chain,
    pub blocks: Vec<i64>,
}

impl FetchBlocksPayload {
    pub fn new(chain: Chain, blocks: Vec<i64>) -> Self {
        Self { chain, blocks }
    }
}
