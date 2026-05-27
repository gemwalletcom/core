mod chatwoot;
mod client;
mod error;
mod model;

pub use chatwoot::ChatwootClient;
pub use client::{SupportClient, SupportProcessResult};
pub use error::ChatwootError;
pub use model::*;
