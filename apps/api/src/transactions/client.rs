use std::error::Error;

use primitives::{Transaction, TransactionsFetchOption, TransactionsResponse};
use storage::DatabaseClient;

pub struct TransactionsClient {
    database: Box<DatabaseClient>,
}

impl TransactionsClient {
    pub async fn new(database_url: &str) -> Self {
        let database = Box::new(DatabaseClient::new(database_url));
        Self { database }
    }

    pub fn get_transactions_by_device_id(
        &mut self,
        device_id: &str,
        options: TransactionsFetchOption,
    ) -> Result<TransactionsResponse, Box<dyn Error + Send + Sync>> {
        let wallet_index = options.wallet_index;
        let subscriptions = self.database.subscriptions().get_subscriptions_by_device_id(device_id, Some(wallet_index))?;

        let addresses = subscriptions.clone().into_iter().map(|x| x.address).collect::<Vec<String>>();
        let chains = subscriptions.clone().into_iter().map(|x| x.chain.as_ref().to_string()).collect::<Vec<String>>();

        let transactions = self
            .database
            .transactions()
            .get_transactions_by_device_id(device_id, addresses.clone(), chains.clone(), options)?
            .into_iter()
            .map(|x| x.as_primitive(addresses.clone()).finalize(addresses.clone()))
            .collect::<Vec<Transaction>>();

        let scan_addresses = transactions.iter().flat_map(|x| x.addresses()).collect::<Vec<String>>();

        let address_names = self
            .database
            .scan_addresses()
            .get_scan_addresses_by_addresses(scan_addresses.clone())?
            .into_iter()
            .flat_map(|x| x.as_primitive())
            .collect();

        Ok(TransactionsResponse::new(transactions, address_names))
    }

    pub fn get_transaction_by_id(&mut self, id: &str) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        Ok(self.database.transactions().get_transaction_by_id(id)?.as_primitive(vec![]))
    }
}
