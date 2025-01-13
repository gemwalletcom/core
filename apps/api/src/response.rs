use primitives::ResponseError;

impl From<Box<dyn std::error::Error + Send + Sync>> for ResponseError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self { error: error.to_string() }
    }
}
