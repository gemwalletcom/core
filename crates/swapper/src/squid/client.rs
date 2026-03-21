use std::fmt::Debug;

use gem_client::{Client, ClientExt};

use super::model::{SquidRouteRequest, SquidRouteResponse, SquidStatusResponse};
use crate::SwapperError;

#[derive(Clone, Debug)]
pub struct SquidClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    client: C,
}

impl<C> SquidClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_route(&self, request: &SquidRouteRequest) -> Result<SquidRouteResponse, SwapperError> {
        self.client.post("/v2/route", request).await.map_err(SwapperError::from)
    }

    pub async fn get_status(&self, tx_hash: &str) -> Result<SquidStatusResponse, SwapperError> {
        let path = format!("/v2/status?transactionId={tx_hash}");
        self.client.get(&path).await.map_err(SwapperError::from)
    }
}
