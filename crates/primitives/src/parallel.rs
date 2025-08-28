use futures::future::try_join_all;
use std::error::Error;

/// Generic parallel mapping utility
/// Maps over items in parallel and collects results
pub async fn parallel_map<I, T, F, Fut, It>(items: It, map_fn: F) -> Result<Vec<T>, Box<dyn Error + Send + Sync>>
where
    I: Send + 'static,
    T: Send + 'static,
    It: IntoIterator<Item = I>,
    F: Fn(I) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<T, Box<dyn Error + Send + Sync>>> + Send,
{
    let futures = items.into_iter().map(map_fn);
    try_join_all(futures).await
}
