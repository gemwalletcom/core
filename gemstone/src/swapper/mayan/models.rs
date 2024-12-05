use thiserror::Error;

#[derive(Debug, Error)]
pub enum MayanError {
    #[error("ABI Error: {msg}")]
    ABIError { msg: String },
}
