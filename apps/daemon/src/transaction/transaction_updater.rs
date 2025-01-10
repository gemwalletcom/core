use std::collections::HashMap;
use std::error::Error;
use storage::models::SubscriptionAddressExclude;
use storage::DatabaseClient;

pub struct TransactionUpdater {
    database: DatabaseClient,
}

impl TransactionUpdater {
    pub fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }
    pub async fn update(&mut self) -> Result<HashMap<String, usize>, Box<dyn Error + Send + Sync>> {
        let addresses_result = self.database.get_transactions_addresses(5000, 5)?;
        let subscriptions_exclude = addresses_result
            .clone()
            .into_iter()
            .map(|x| SubscriptionAddressExclude {
                address: x.address,
                chain: x.chain_id,
            })
            .collect();
        let subscriptions_excluded_added = self.database.add_subscriptions_address_exclude(subscriptions_exclude)?;

        let addresses = addresses_result.clone().into_iter().map(|x| x.address).collect::<Vec<_>>();
        let result = self.database.delete_transactions_addresses(addresses.clone())?;

        let transactions_without_addresses = self.database.get_transactions_without_addresses(10000)?;
        let transactions = self.database.delete_transactions_by_ids(transactions_without_addresses.clone())?;

        let result = HashMap::from([
            ("addresses".to_string(), addresses.len()),
            ("transactions_addresses".to_string(), result),
            ("transactions_without_addresses".to_string(), transactions_without_addresses.len()),
            ("subscriptions_excluded_added".to_string(), subscriptions_excluded_added),
            ("transactions_deleted".to_string(), transactions),
        ]);

        Ok(result)
    }
}
