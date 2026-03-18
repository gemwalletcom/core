use super::sync::{SearchSyncClient, SearchSyncResult};
use primitives::ConfigKey;
use search_index::{NFTDocument, NFTS_INDEX_NAME, SearchIndexClient};
use storage::models::NftCollectionRow;
use storage::{Database, NftRepository};

pub struct NftsIndexUpdater {
    database: Database,
    sync_client: SearchSyncClient,
}

impl NftsIndexUpdater {
    pub fn new(database: Database, search_index: &SearchIndexClient) -> Self {
        Self {
            sync_client: SearchSyncClient::new(database.clone(), search_index),
            database,
        }
    }

    pub async fn update(&self) -> Result<SearchSyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let collections = self.database.nft()?.get_nft_collections_all()?;

        let sync = self.sync_client.for_key(ConfigKey::SearchNftsLastUpdatedAt)?;
        let documents = if sync.should_replace_index() {
            Self::build_documents(collections.iter())
        } else {
            Self::build_documents(collections.iter().filter(|c| c.updated_at > sync.last_updated_at()))
        };

        sync.write(NFTS_INDEX_NAME, documents).await
    }

    fn build_documents<'a>(collections: impl IntoIterator<Item = &'a NftCollectionRow>) -> Vec<NFTDocument> {
        collections.into_iter().map(|c| NFTDocument::from(c.as_primitive(vec![]))).collect()
    }
}
