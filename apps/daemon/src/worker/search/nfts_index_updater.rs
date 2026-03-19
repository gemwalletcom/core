use super::sync::{SearchSyncClient, SearchSyncResult};
use primitives::ConfigKey;
use search_index::{NFTDocument, NFTS_INDEX_NAME, SearchIndexClient};
use storage::models::NftCollectionRow;
use storage::{Database, NftCollectionFilter, NftRepository};

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
        let sync = self.sync_client.for_key(ConfigKey::SearchNftsLastUpdatedAt)?;
        let filters = sync.since().map(NftCollectionFilter::UpdatedSince).into_iter().collect();
        let collections = self.database.nft()?.get_nft_collections_by_filter(filters)?;

        let documents = Self::build_documents(collections.iter());

        sync.write(NFTS_INDEX_NAME, documents).await
    }

    fn build_documents<'a>(collections: impl IntoIterator<Item = &'a NftCollectionRow>) -> Vec<NFTDocument> {
        collections.into_iter().map(|c| NFTDocument::from(c.as_primitive(vec![]))).collect()
    }
}
