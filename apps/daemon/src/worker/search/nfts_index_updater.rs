use search_index::{NFTDocument, NFTS_INDEX_NAME, SearchIndexClient};
use storage::Database;

pub struct NftsIndexUpdater {
    database: Database,
    search_index: SearchIndexClient,
}

impl NftsIndexUpdater {
    pub fn new(database: Database, search_index: &SearchIndexClient) -> Self {
        Self {
            database,
            search_index: search_index.clone(),
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let collections = self.database.client()?.nft().get_nft_collections_all()?;

        let documents: Vec<NFTDocument> = collections
            .into_iter()
            .map(|collection| {
                let primitive_collection = collection.as_primitive(vec![]);
                NFTDocument::from(primitive_collection)
            })
            .collect();

        self.search_index.sync_documents(NFTS_INDEX_NAME, documents, |doc| doc.id.clone()).await
    }
}
