use super::model::{SimulateSwapRequest, SwapSimulation};
use crate::SwapperError;
use gem_client::{Client, ClientExt, build_path_with_query};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct StonfiClient<C>
where
    C: Client + Clone + Debug,
{
    client: C,
}

impl<C> StonfiClient<C>
where
    C: Client + Clone + Debug,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn simulate_swap(&self, request: &SimulateSwapRequest) -> Result<SwapSimulation, SwapperError> {
        let path = build_path_with_query("/v1/swap/simulate", request)?;
        self.client.post(&path, &"").await.map_err(SwapperError::from)
    }
}
