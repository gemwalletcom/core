use storage::DatabaseClient;

#[allow(dead_code)]
pub struct Client {
    database: DatabaseClient,
}

impl Client {
    pub async fn new(database: DatabaseClient) -> Self {
        Self { database }
    }
}
