#[derive(Debug)]
pub enum CacheError {
    NotFound(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::NotFound(key) => write!(f, "Key not found: {}", key),
        }
    }
}

impl std::error::Error for CacheError {}
