use std::fmt;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, EnumIter)]
pub enum QueueName {
    // Process transactions, store and send notifications. Push assets to address_assets table and fetch new assets
    StoreTransactions,
    // Notifications for price alerts
    NotificationsPriceAlerts,
    // Notifications for transactions
    NotificationsTransactions,
    // Notifications for observers
    NotificationsObservers,
    // fetch new assets and store to db
    FetchAssets,
    // fetch new blocks and store to db
    FetchBlocks,
    // Fetch and store nft collection
    FetchNFTCollection,
    // Fetch and store nft collection assets
    FetchNFTCollectionAssets,
    // Add assets/addresses association to address_assets table
    AssetsAddressesAssociations,
    // Fetch address assets from providers and store to db
    FetchAssetsAddressesAssociations,
    // Fetch address nft assets from providers and store to db
    FetchNftAssetsAddressesAssociations,
    // Fetch transactions from providers and store to db
    FetchTransactions,
    // Process fiat order webhooks
    FiatOrderWebhooks,
}

impl QueueName {
    pub fn all() -> Vec<QueueName> {
        QueueName::iter().collect()
    }
}

impl fmt::Display for QueueName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueName::StoreTransactions => write!(f, "store_transactions"),
            QueueName::NotificationsPriceAlerts => write!(f, "notifications_price_alerts"),
            QueueName::NotificationsTransactions => write!(f, "notifications_transactions"),
            QueueName::NotificationsObservers => write!(f, "notifications_observers"),
            QueueName::FetchAssets => write!(f, "fetch_assets"),
            QueueName::FetchBlocks => write!(f, "fetch_blocks"),
            QueueName::FetchNFTCollection => write!(f, "fetch_nft_collection"),
            QueueName::FetchNFTCollectionAssets => write!(f, "fetch_nft_collection_assets"),
            QueueName::AssetsAddressesAssociations => write!(f, "assets_addresses_associations"),
            QueueName::FetchAssetsAddressesAssociations => write!(f, "fetch_assets_addresses_associations"),
            QueueName::FetchTransactions => write!(f, "fetch_transactions"),
            QueueName::FetchNftAssetsAddressesAssociations => write!(f, "fetch_nft_assets_addresses_associations"),
            QueueName::FiatOrderWebhooks => write!(f, "fiat_order_webhooks"),
        }
    }
}
