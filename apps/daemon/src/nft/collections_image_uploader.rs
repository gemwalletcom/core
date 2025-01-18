use storage::DatabaseClient;

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
        let collections = self.database.get_nft_collections()?.into_iter().map(|x| x.as_primitive()).collect::<Vec<_>>();

        for collection in collections.clone() {
            println!("Uploading image collection: {}", collection.name);

            let path = format!("{}/{}/collection_original.png", collection.chain.as_ref(), collection.contract_address);
            let image_url = collection.image.image_url.clone();

            self.image_uploader.upload_image_from_url(image_url.as_str(), path.as_str()).await?;

            //let _res = self.upload_collection(path.as_str(), image_url.as_str()).await?;
            // Ok(_) => {
            //     println!("Uploaded image collection: {}", collection.name);
            // }
            // Err(e) => {
            //     println!("Failed to upload image collection: {}", e);
            // }
            // };
        }

        //self.upload_collection("test", "test").await?;

        Ok(0)
    }
}
