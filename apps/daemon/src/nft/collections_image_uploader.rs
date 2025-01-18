use storage::{models::nft_collection::UpdateNftCollectionImageUrl, DatabaseClient};

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

    pub async fn update(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.image_uploader.bucket.url.clone();

        let collections = self
            .database
            .get_nft_collections()?
            .into_iter()
            .map(|x| x.as_primitive())
            .filter(|x| !x.image.image_url.contains(&url))
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
                image_url: Some(uploaded_image_url),
            })?;
        }

        Ok(collections.len())
    }
}
