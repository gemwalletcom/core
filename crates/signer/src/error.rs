pub use primitives::SignerError;

pub fn format_err(e: impl std::fmt::Display) -> SignerError {
    SignerError::invalid_input(e.to_string())
}
