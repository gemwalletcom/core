use chrono::Utc;
use primitives::ConfigKey;
use search_index::{NFTDocument, NFTS_INDEX_NAME, SearchIndexClient};
use storage::models::NftCollectionRow;
use storage::{ConfigCacher, Database, NftRepository};

pub struct NftsIndexUpdater {
    database: Database,
    config: ConfigCacher,
    search_index: SearchIndexClient,
}

impl NftsIndexUpdater {
    pub fn new(database: Database, search_index: &SearchIndexClient) -> Self {
        let config = ConfigCacher::new(database.clone());
        Self {
            database,
            config,
            search_index: search_index.clone(),
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let collections = self.database.nft()?.get_nft_collections_all()?;

        let now = Utc::now().naive_utc();
        let last_updated_at = self.config.get_datetime(ConfigKey::SearchNftsLastUpdatedAt)?;

        let updated_collections: Vec<_> = collections.into_iter().filter(|c| c.updated_at > last_updated_at).collect();
        let documents = Self::build_documents(&updated_collections);
        let count = documents.len();
        if count > 0 {
            self.search_index.add_documents(NFTS_INDEX_NAME, documents).await?;
        }

        self.config.set_datetime(ConfigKey::SearchNftsLastUpdatedAt, now)?;
        Ok(count)
    }

    fn build_documents(collections: &[NftCollectionRow]) -> Vec<NFTDocument> {
        collections.iter().map(|c| NFTDocument::from(c.as_primitive(vec![]))).collect()
    }
}
