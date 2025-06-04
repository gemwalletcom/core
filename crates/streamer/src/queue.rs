use std::fmt;

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
