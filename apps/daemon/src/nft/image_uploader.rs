use aws_sdk_s3::{
    config::{Credentials, Region},
    primitives::ByteStream,
    Client, Config,
};

use settings::BucketConfiguration;
use std::error::Error;

pub struct ImageUploaderClient {
    pub bucket: BucketConfiguration,
}

impl ImageUploaderClient {
    pub fn new(bucket: BucketConfiguration) -> Self {
        Self { bucket }
    }

    fn file_url(&self, path: &str) -> String {
        format!("{}/{}", self.bucket.url, path)
    }

    pub async fn upload_image_from_url(&self, image_url: &str, path: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let response = reqwest::get(image_url).await?;
        if !response.status().is_success() {
            return Err(format!("Failed to download image: {}", response.status()).into());
        }
        let content_type = match response.headers().get(reqwest::header::CONTENT_TYPE) {
            Some(header_value) => header_value.to_str()?.to_string(),
            None => return Err("Content-Type header not found".into()),
        };

        let image_bytes = response.bytes().await?;
        let data = ByteStream::from(image_bytes.to_vec());

        self.upload_data(path, data, content_type.as_str()).await
    }

    async fn upload_data(&self, path: &str, data: ByteStream, content_type: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let credentials = Credentials::new(self.bucket.key.public.clone(), self.bucket.key.secret.clone(), None, None, "custom");
        let config = Config::builder()
            .region(Region::new(self.bucket.region.clone()))
            .endpoint_url(self.bucket.endpoint.clone())
            .credentials_provider(credentials)
            .build();

        let client = Client::from_conf(config);

        client
            .put_object()
            .bucket(self.bucket.name.clone())
            .key(path)
            .content_type(content_type)
            .body(data)
            .send()
            .await?;

        Ok(self.file_url(path))
    }
}
