use super::MayanClient;
use crate::SwapperError;
use gem_client::{Client, ClientExt, build_path_with_query};
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

impl<C> MayanClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub(in crate::mayan) async fn get_swap<T, U>(&self, path: &str, params: T) -> Result<U, SwapperError>
    where
        T: Serialize,
        U: DeserializeOwned + Send,
    {
        let path = build_path_with_query(path, &params)?;
        self.client.get(&path).await.map_err(SwapperError::from)
    }

    pub(in crate::mayan) async fn post_swap<T, U>(&self, path: &str, params: T) -> Result<U, SwapperError>
    where
        T: Serialize + Send + Sync,
        U: DeserializeOwned + Send,
    {
        self.client.post(path, &params).await.map_err(SwapperError::from)
    }
}
