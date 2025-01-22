use nft::opensea::OpenSeaClient;
use primitives::LinkType;
use storage::{models::NftLink, DatabaseClient};

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
            let opensea_collection = self.opensea_client.get_collection(&collection.contrtact_address).await?;

            let mut links: Vec<NftLink> = vec![];

            if !opensea_collection.opensea_url.is_empty() {
                links.push(NftLink {
                    collection_id: collection.id.clone(),
                    link_type: LinkType::OpenSea.as_ref().to_string(),
                    url: opensea_collection.opensea_url,
                });
            }
            if !opensea_collection.project_url.is_empty() {
                links.push(NftLink {
                    collection_id: collection.id.clone(),
                    link_type: LinkType::Website.as_ref().to_string(),
                    url: opensea_collection.project_url,
                });
            }
            if !opensea_collection.twitter_username.is_empty() {
                links.push(NftLink {
                    collection_id: collection.id.clone(),
                    link_type: LinkType::X.as_ref().to_string(),
                    url: format!("https://x.com/{}", opensea_collection.twitter_username),
                });
            }
            if !opensea_collection.instagram_username.is_empty() {
                links.push(NftLink {
                    collection_id: collection.id.clone(),
                    link_type: LinkType::Instagram.as_ref().to_string(),
                    url: format!("https://instagram.com/{}", opensea_collection.instagram_username),
                });
            }
            self.database.add_nft_links(links.clone())?;

            println!("Updating collection: {}, links: {:?}", collection.name, links);
        }

        // self.search_index.add_documents(ASSETS_INDEX_NAME, documents.clone()).await?;

        Ok(collections.len())
    }
}
