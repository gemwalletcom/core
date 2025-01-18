use nft::opensea::OpenSeaClient;
use storage::DatabaseClient;

pub struct OpenSeaUpdater {
    database: DatabaseClient,
    opensea_client: OpenSeaClient,
}

impl OpenSeaUpdater {
    pub fn new(database_url: &str, opensea_client: OpenSeaClient) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database, opensea_client }
    }

    pub async fn update(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let collections = self.database.get_nft_collections()?;

        for collection in collections.clone() {
            let collection = self.opensea_client.get_collection(&collection.contrtact_address).await?;

            println!("Updating collection: {}", collection.name);
        }

        // self.search_index.add_documents(ASSETS_INDEX_NAME, documents.clone()).await?;

        Ok(collections.len())
    }
}
