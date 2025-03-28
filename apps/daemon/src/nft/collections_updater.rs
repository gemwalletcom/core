use std::str::FromStr;

use nft::get_image_mime_type;
use nft::opensea::{model::Collection, OpenSeaClient};
use primitives::{Chain, LinkType};
use storage::{
    models::{nft_collection::UpdateNftCollectionImageUrl, NftCollection, NftLink},
    DatabaseClient,
};

pub struct OpenSeaUpdater {
    database: DatabaseClient,
    opensea_client: OpenSeaClient,
}

impl OpenSeaUpdater {
    pub fn new(database_url: &str, opensea_client: OpenSeaClient) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database, opensea_client }
    }

    pub async fn update_collections(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let collections = self.database.get_nft_collections_all()?;

        for collection in collections.clone() {
            let chain = Chain::from_str(collection.chain.as_str())?;
            match chain {
                Chain::Ethereum => {
                    let opensea_collection = self.opensea_client.get_collection_id(chain.as_ref(), &collection.contract_address).await?;
                    let _ = self.update_collection(collection.clone(), opensea_collection);

                    // update mime types
                    if collection.image_preview_mime_type.is_none() {
                        let image_preview_mime_type = get_image_mime_type(&collection.clone().image_preview_url.unwrap_or_default()).await?;

                        let update = UpdateNftCollectionImageUrl {
                            id: collection.id.clone(),
                            image_preview_url: collection.image_preview_url.clone(),
                            image_preview_mime_type: Some(image_preview_mime_type),
                        };

                        self.database.update_nft_collection_image_url(update)?;
                    }

                    println!("Updating collection: {}", collection.name);
                }
                _ => continue,
            }
        }

        Ok(collections.len())
    }

    fn update_collection(&mut self, collection: NftCollection, opensea_collection: Collection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        self.database.add_nft_collections_links(links.clone())?;
        Ok(())
    }
}
