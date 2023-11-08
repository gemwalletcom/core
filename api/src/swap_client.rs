extern crate rocket;

use storage::DatabaseClient;

pub struct SwapClient {
    database: DatabaseClient,
}

impl SwapClient {
    pub async fn new(
        database_url: &str
    ) -> Self {
        let database = DatabaseClient::new(database_url);
        Self {
            database,
        }
    }
}