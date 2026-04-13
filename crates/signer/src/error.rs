use primitives::SignerError;

/// Extension trait to convert `Result` / `Option` into `Result<_, SignerError>`
/// with a single `invalid_input(msg)` call.
pub trait InvalidInput {
    type Ok;

    fn invalid_input(self, msg: &'static str) -> Result<Self::Ok, SignerError>;
}

impl<T, E> InvalidInput for Result<T, E> {
    type Ok = T;

    fn invalid_input(self, msg: &'static str) -> Result<T, SignerError> {
        self.map_err(|_| SignerError::invalid_input(msg))
    }
}

impl<T> InvalidInput for Option<T> {
    type Ok = T;

    fn invalid_input(self, msg: &'static str) -> Result<T, SignerError> {
        self.ok_or_else(|| SignerError::invalid_input(msg))
    }
}
