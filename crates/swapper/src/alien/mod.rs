pub mod error;
pub mod mock;
#[cfg(feature = "reqwest_provider")]
pub mod reqwest_provider;

pub use error::AlienError;
pub use gem_jsonrpc::{HttpMethod, RpcClient as GenericRpcClient, RpcProvider as GenericRpcProvider, Target, X_CACHE_TTL};

pub type RpcClient = GenericRpcClient<AlienError>;

pub trait RpcProvider: GenericRpcProvider<Error = AlienError> {}

impl<T> RpcProvider for T where T: GenericRpcProvider<Error = AlienError> {}
