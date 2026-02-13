mod memory;
mod types;

use crate::jsonrpc_types::{JsonRpcCall, RequestType};
use crate::proxy::CachedResponse;
use primitives::Chain;
use std::future::Future;
use std::time::Duration;

pub use memory::MemoryCache;

pub trait CacheProvider: Send + Sync {
    fn get(&self, chain: &Chain, key: &str) -> impl Future<Output = Option<CachedResponse>> + Send;
    fn set(&self, chain: &Chain, key: String, response: CachedResponse, ttl: Duration) -> impl Future<Output = ()> + Send;
    fn should_cache(&self, chain: &Chain, path: &str, method: &str, body: Option<&[u8]>) -> Option<Duration>;
    fn should_cache_request(&self, chain: &Chain, request_type: &RequestType) -> Option<Duration>;
    fn should_cache_call(&self, chain: &Chain, call: &JsonRpcCall) -> Option<Duration>;
    fn should_inflight_request(&self, chain: &Chain, request_type: &RequestType) -> bool;
}

pub type RequestCache = MemoryCache;
