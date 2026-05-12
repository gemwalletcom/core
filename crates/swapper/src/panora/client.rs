use super::model::{QuoteRequest, QuoteResponse};
use crate::SwapperError;
use gem_client::{Client, ClientExt, build_path_with_query};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct PanoraClient<C>
where
    C: Client + Clone + Debug,
{
    client: C,
}

impl<C> PanoraClient<C>
where
    C: Client + Clone + Debug,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<QuoteResponse, SwapperError> {
        let path = build_path_with_query("/swap", request)?;
        self.client.post(&path, &serde_json::json!({})).await.map_err(SwapperError::from)
    }
}
