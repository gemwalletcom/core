use primitives::MIME_TYPE_PNG;
use storage::{
    models::{nft_asset::UpdateNftAssetImageUrl, nft_collection::UpdateNftCollectionImageUrl},
    DatabaseClient,
};

use super::image_uploader::ImageUploaderClient;

pub struct CollectionsImageUploader {
    database: DatabaseClient,
    image_uploader: ImageUploaderClient,
}

impl CollectionsImageUploader {
    pub fn new(database_url: &str, image_uploader: ImageUploaderClient) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database, image_uploader }
    }

    fn image_filter(image_url: &str, url: &str) -> bool {
        !image_url.contains(url) && image_url.contains(".png")
    }

    pub async fn update_collections(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.image_uploader.bucket.url.clone();

        let collections = self
            .database
            .get_nft_collections_all()?
            .into_iter()
            .map(|x| x.as_primitive(vec![]))
            .filter(|x| Self::image_filter(x.image.image_url.as_str(), &url))
            .collect::<Vec<_>>();

        for collection in collections.clone() {
            println!("Uploading image collection: {}", collection.name);

            let path = collection.image_path().image_url;
            let uploaded_image_url = self
                .image_uploader
                .upload_image_from_url(collection.image.image_url.clone().as_str(), path.as_str())
                .await?;

            println!("Image uploaded: {}", uploaded_image_url);

            self.database.update_nft_collection_image_url(UpdateNftCollectionImageUrl {
                id: collection.id,
                image_preview_url: Some(uploaded_image_url.clone()),
                image_preview_mime_type: Some(MIME_TYPE_PNG.to_string()),
            })?;
        }

        Ok(collections.len())
    }

    pub async fn update_collection_assets(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.image_uploader.bucket.url.clone();

        let assets = self
            .database
            .get_nft_assets_all()?
            .into_iter()
            .map(|x| x.as_primitive())
            .filter(|x| Self::image_filter(x.image.image_url.as_str(), &url))
            .collect::<Vec<_>>();

        for asset in assets.clone() {
            println!("Uploading image asset: {}, name: {}", asset.collection_id, asset.name);

            let path = asset.image_path().image_url;
            let uploaded_image_url = self
                .image_uploader
                .upload_image_from_url(asset.image.image_url.clone().as_str(), path.as_str())
                .await?;

            self.database.update_nft_asset_image_url(UpdateNftAssetImageUrl {
                id: asset.id,
                image_preview_url: Some(uploaded_image_url.clone()),
                image_preview_mime_type: Some(MIME_TYPE_PNG.to_string()),
            })?;
        }

        Ok(assets.len())
    }
}
