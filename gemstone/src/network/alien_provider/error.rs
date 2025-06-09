#[derive(Debug, Clone, thiserror::Error, uniffi::Error)]
pub enum AlienError {
    #[error("Request is invalid: {msg}")]
    RequestError { msg: String },
    #[error("Request error: {msg}")]
    ResponseError { msg: String },
}
