#[cfg(feature = "client")]
use crate::RpcClientError;
#[cfg(feature = "client")]
use gem_client::ClientError;

#[cfg(feature = "client")]
impl RpcClientError for ClientError {
    fn into_client_error(self) -> ClientError {
        self
    }
}

#[cfg(feature = "reqwest")]
pub mod reqwest;
#[cfg(feature = "reqwest")]
pub use reqwest::NativeProvider;
