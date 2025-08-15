use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Timeout error")]
    Timeout,
    #[error("HTTP error: status {status}, body: {body}")]
    Http { status: u16, body: String },
    #[error("Serialization error: {0}")]
    Serialization(String),
}
