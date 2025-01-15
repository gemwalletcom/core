use storage::DatabaseClient;

pub struct OpenSeaUpdater {
    database: DatabaseClient,
}

impl OpenSeaUpdater {
    pub fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub async fn update(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let collections = self.database.get_nft_collections()?;

        //for collection in collections.clone() {
        //
        //}

        // self.search_index.add_documents(ASSETS_INDEX_NAME, documents.clone()).await?;

        Ok(collections.len())
    }
}
