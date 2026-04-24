use std::error::Error;

use gem_client::{ClientExt, ReqwestClient};

use super::model::PricesResponse;

pub struct DefiLlamaClient {
    client: ReqwestClient,
}

impl DefiLlamaClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client }
    }

    pub async fn get_prices(&self, coins: &[String]) -> Result<PricesResponse, Box<dyn Error + Send + Sync>> {
        let path = format!("/prices/current/{}", coins.join(","));
        Ok(self.client.get::<PricesResponse>(&path).await?)
    }
}
