use std::fmt;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, EnumIter)]
pub enum QueueName {
    // Process transactions, store and send notifications. Push assets to address_assets table and fetch new assets
    Transactions,
    // Notifications for price alerts
    NotificationsPriceAlerts,
    // Notifications for transactions
    NotificationsTransactions,
    // fetch new assets and store to db
    FetchAssets,
    // fetch new blocks and store to db
    FetchBlocks,
    // Fetch and store nft collection
    FetchNFTCollection,
    // Fetch and store nft collection assets
    FetchNFTCollectionAssets,
    // Add assets/addresses association to address_assets table
    AddressAssets,
}

impl QueueName {
    pub fn all() -> Vec<QueueName> {
        QueueName::iter().collect()
    }
}

impl fmt::Display for QueueName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueName::Transactions => write!(f, "transactions"),
            QueueName::NotificationsPriceAlerts => write!(f, "notifications_price_alerts"),
            QueueName::NotificationsTransactions => write!(f, "notifications_transactions"),
            QueueName::FetchAssets => write!(f, "fetch_assets"),
            QueueName::FetchBlocks => write!(f, "fetch_blocks"),
            QueueName::FetchNFTCollection => write!(f, "fetch_nft_collection"),
            QueueName::FetchNFTCollectionAssets => write!(f, "fetch_nft_collection_assets"),
            QueueName::AddressAssets => write!(f, "address_assets"),
        }
    }
}
