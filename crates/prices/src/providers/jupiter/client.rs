use std::error::Error;

use super::model::{VerifiedToken, VerifiedTokensResponse};
use gem_client::{ClientExt, ReqwestClient};

pub struct JupiterClient {
    client: ReqwestClient,
}

impl JupiterClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client }
    }

    pub async fn get_verified_tokens(&self) -> Result<Vec<VerifiedToken>, Box<dyn Error + Send + Sync>> {
        let query = vec![("query".to_string(), "verified".to_string())];
        Ok(self.client.get_with_query::<VerifiedTokensResponse>("/tokens/v2/tag", &query).await?)
    }
}
