use std::fmt::Debug;

pub mod jsonrpc;
pub mod mock;
pub mod provider;
pub mod target;
pub use jsonrpc::*;
pub use provider::*;
pub use target::*;

#[derive(Debug, Clone, uniffi::Error, thiserror::Error)]
pub enum AlienError {
    #[error("Request is invalid: {msg}")]
    RequestError { msg: String },
    #[error("Request error: {msg}")]
    ResponseError { msg: String },
}
