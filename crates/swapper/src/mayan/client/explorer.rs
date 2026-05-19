use super::MayanClient;
use crate::{
    SwapperError,
    mayan::model::{MayanChain, MayanTransactionResult},
};
use gem_client::{Client, ClientExt};
use std::fmt::Debug;

impl<C> MayanClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub async fn get_chains(&self) -> Result<Vec<MayanChain>, SwapperError> {
        self.client.get("/chains").await.map_err(SwapperError::from)
    }

    pub async fn get_transaction_status(&self, tx_hash: &str) -> Result<MayanTransactionResult, SwapperError> {
        self.client.get(&format!("/swap/trx/{tx_hash}")).await.map_err(SwapperError::from)
    }
}
