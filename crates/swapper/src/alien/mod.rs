pub mod mock;

pub use gem_client::ClientError as AlienError;
pub use gem_jsonrpc::{HttpMethod, RpcClient as GenericRpcClient, RpcProvider as GenericRpcProvider, Target};
pub type RpcClient = GenericRpcClient<AlienError>;

pub trait RpcProvider: GenericRpcProvider<Error = AlienError> {}

impl<T> RpcProvider for T where T: GenericRpcProvider<Error = AlienError> {}
