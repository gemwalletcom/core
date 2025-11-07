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
    // Notifications for support messages
    NotificationsSupport,
    // Failed notifications to handle device disabling
    NotificationsFailed,
    // fetch new assets and store to db
    FetchAssets,
    // fetch new blocks and store to db
    FetchBlocks,
    // Fetch and store nft collection
    FetchNFTCollection,
    // Fetch and store nft collection assets
    FetchNFTCollectionAssets,
    // Store assets associations to address_assets table
    StoreAssetsAssociations,
    // Fetch address token balances from providers and store to db
    FetchTokenAssociations,
    // Fetch address coin balances from providers and store to db
    FetchCoinAssociations,
    // Fetch address nft assets from providers and store to db
    FetchNftAssociations,
    // Fetch address transactions from providers and store to db
    FetchAddressTransactions,
    // Process fiat order webhooks
    FiatOrderWebhooks,
    // Process support webhooks
    SupportWebhooks,
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
            QueueName::StoreAssetsAssociations => write!(f, "store_assets_associations"),
            QueueName::FetchTokenAssociations => write!(f, "fetch_token_associations"),
            QueueName::FetchCoinAssociations => write!(f, "fetch_coin_associations"),
            QueueName::FetchAddressTransactions => write!(f, "fetch_address_transactions"),
            QueueName::FetchNftAssociations => write!(f, "fetch_nft_associations"),
            QueueName::FiatOrderWebhooks => write!(f, "fiat_order_webhooks"),
            QueueName::SupportWebhooks => write!(f, "support_webhooks"),
            QueueName::NotificationsSupport => write!(f, "notifications_support"),
            QueueName::NotificationsFailed => write!(f, "notifications_failed"),
        }
    }
}
