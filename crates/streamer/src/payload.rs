use primitives::{
    AssetAddress, AssetId, Chain, ChainAddress, ChartData, FailedNotification, FiatProviderName, FiatTransaction, GorushNotification, NotificationType, PriceData, Transaction,
    TransactionId, WalletId,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsPayload {
    pub chain: Chain,
    pub blocks: Vec<u64>,
    pub transactions: Vec<Transaction>,
}

impl fmt::Display for TransactionsPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chain: {}, blocks: {:?}, transactions: {}", self.chain.as_ref(), self.blocks, self.transactions.len())
    }
}

impl TransactionsPayload {
    pub fn new(chain: Chain, blocks: Vec<u64>, transactions: Vec<Transaction>) -> Self {
        Self { chain, blocks, transactions }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsPayload {
    pub notifications: Vec<GorushNotification>,
}

impl fmt::Display for NotificationsPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "notifications: {}", self.notifications.len())
    }
}

impl NotificationsPayload {
    pub fn new(notifications: Vec<GorushNotification>) -> Self {
        Self { notifications }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsFailedPayload {
    pub failures: Vec<FailedNotification>,
}

impl fmt::Display for NotificationsFailedPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failures: {}", self.failures.len())
    }
}

impl NotificationsFailedPayload {
    pub fn new(failures: Vec<FailedNotification>) -> Self {
        Self { failures }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchAssetsPayload {
    pub asset_id: AssetId,
}

impl fmt::Display for FetchAssetsPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chain: {}, token_id: {:?}", self.asset_id.chain.as_ref(), self.asset_id.token_id)
    }
}

impl FetchAssetsPayload {
    pub fn new(asset_id: AssetId) -> Self {
        Self { asset_id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchBlocksPayload {
    pub chain: Chain,
    pub block: u64,
}

impl fmt::Display for FetchBlocksPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chain: {}, block: {}", self.chain.as_ref(), self.block)
    }
}

impl FetchBlocksPayload {
    pub fn new(chain: Chain, block: u64) -> Self {
        Self { chain, block }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchNFTCollectionPayload {
    pub chain: Chain,
    pub collection_id: String,
}

impl FetchNFTCollectionPayload {
    pub fn new(chain: Chain, collection_id: String) -> Self {
        Self { chain, collection_id }
    }
}

impl fmt::Display for FetchNFTCollectionPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chain: {}, collection_id: {}", self.chain.as_ref(), self.collection_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchNFTCollectionAssetPayload {
    pub chain: Chain,
    pub collection_id: String,
    pub asset_id: String,
}

impl FetchNFTCollectionAssetPayload {
    pub fn new(chain: Chain, collection_id: String, asset_id: String) -> Self {
        Self { chain, collection_id, asset_id }
    }
}

impl fmt::Display for FetchNFTCollectionAssetPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chain: {}, collection_id: {}, asset_id: {}", self.chain.as_ref(), self.collection_id, self.asset_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetsAddressPayload {
    pub values: Vec<AssetAddress>,
}

impl AssetsAddressPayload {
    pub fn new(values: Vec<AssetAddress>) -> Self {
        Self { values }
    }
}

impl fmt::Display for AssetsAddressPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for value in self.values.iter() {
            write!(f, "address: {}, asset_id: {}", value.address, value.asset_id)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainAddressPayload {
    pub value: ChainAddress,
}

impl ChainAddressPayload {
    pub fn new(value: ChainAddress) -> Self {
        Self { value }
    }
}

impl fmt::Display for ChainAddressPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chain: {}, address: {}", self.value.chain, self.value.address)
    }
}

impl From<ChainAddress> for ChainAddressPayload {
    fn from(chain_address: ChainAddress) -> Self {
        Self::new(chain_address)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum FiatWebhook {
    OrderId(String),
    Transaction(FiatTransaction),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiatWebhookPayload {
    pub provider: FiatProviderName,
    pub data: serde_json::Value,
    pub payload: FiatWebhook,
}

impl FiatWebhookPayload {
    pub fn new(provider: FiatProviderName, data: serde_json::Value, payload: FiatWebhook) -> Self {
        Self { provider, data, payload }
    }
}

impl fmt::Display for FiatWebhookPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "provider: {}", self.provider.id())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportWebhookPayload {
    pub data: serde_json::Value,
}

impl SupportWebhookPayload {
    pub fn new(data: serde_json::Value) -> Self {
        Self { data }
    }
}

impl fmt::Display for SupportWebhookPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "size: {} bytes", serde_json::to_vec(&self.data).unwrap().len())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricesPayload {
    pub prices: Vec<PriceData>,
}

impl PricesPayload {
    pub fn new(prices: Vec<PriceData>) -> Self {
        Self { prices }
    }
}

impl fmt::Display for PricesPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "prices: {}", self.prices.len())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartsPayload {
    pub charts: Vec<ChartData>,
}

impl ChartsPayload {
    pub fn new(charts: Vec<ChartData>) -> Self {
        Self { charts }
    }
}

impl fmt::Display for ChartsPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "charts: {}", self.charts.len())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsNotificationPayload {
    pub event_id: i32,
}

impl RewardsNotificationPayload {
    pub fn new(event_id: i32) -> Self {
        Self { event_id }
    }
}

impl fmt::Display for RewardsNotificationPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "event_id: {}", self.event_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsRedemptionPayload {
    pub redemption_id: i32,
}

impl RewardsRedemptionPayload {
    pub fn new(redemption_id: i32) -> Self {
        Self { redemption_id }
    }
}

impl fmt::Display for RewardsRedemptionPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "redemption_id: {}", self.redemption_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InAppNotificationPayload {
    pub wallet_id: i32,
    pub asset_id: Option<AssetId>,
    pub notification_type: NotificationType,
    pub metadata: Option<serde_json::Value>,
}

impl InAppNotificationPayload {
    pub fn new(wallet_id: i32, notification_type: NotificationType, metadata: Option<serde_json::Value>) -> Self {
        Self {
            wallet_id,
            asset_id: None,
            notification_type,
            metadata,
        }
    }

    pub fn new_with_asset(wallet_id: i32, asset_id: AssetId, notification_type: NotificationType, metadata: Option<serde_json::Value>) -> Self {
        Self {
            wallet_id,
            asset_id: Some(asset_id),
            notification_type,
            metadata,
        }
    }
}

impl fmt::Display for InAppNotificationPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.metadata {
            Some(metadata) => write!(
                f,
                "wallet_id: {}, notification_type: {}, metadata: {}",
                self.wallet_id,
                self.notification_type.as_ref(),
                metadata
            ),
            None => write!(f, "wallet_id: {}, notification_type: {}", self.wallet_id, self.notification_type.as_ref()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCoinInfoPayload {
    pub coin_id: String,
}

impl UpdateCoinInfoPayload {
    pub fn new(coin_id: String) -> Self {
        Self { coin_id }
    }
}

impl fmt::Display for UpdateCoinInfoPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "coin_id: {}", self.coin_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchPricesPayload {
    pub price_ids: Vec<String>,
}

impl FetchPricesPayload {
    pub fn new(price_ids: Vec<String>) -> Self {
        Self { price_ids }
    }
}

impl fmt::Display for FetchPricesPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "price_ids: {}", self.price_ids.len())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStreamPayload {
    pub device_id: String,
    pub event: DeviceStreamEvent,
}

impl fmt::Display for DeviceStreamPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "device_id: {}, {}", self.device_id, self.event)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceStreamEvent {
    Transactions {
        wallet_id: WalletId,
        transaction_ids: Vec<TransactionId>,
        asset_ids: Vec<AssetId>,
    },
    Nft {
        wallet_id: WalletId,
    },
}

impl fmt::Display for DeviceStreamEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceStreamEvent::Transactions { transaction_ids, asset_ids, .. } => {
                write!(f, "transactions: {}, assets: {}", transaction_ids.len(), asset_ids.len())
            }
            DeviceStreamEvent::Nft { wallet_id } => write!(f, "nft: {}", wallet_id),
        }
    }
}
