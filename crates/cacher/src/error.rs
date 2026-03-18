#[derive(Debug, Clone)]
pub enum CacheError {
    NotFound { resource: &'static str, lookup: String },
    ResourceNotFound(&'static str),
    KeyNotFound(String),
}

impl CacheError {
    pub fn not_found(resource: &'static str, lookup: impl Into<String>) -> Self {
        Self::NotFound { resource, lookup: lookup.into() }
    }

    pub fn not_found_resource(resource: &'static str) -> Self {
        Self::ResourceNotFound(resource)
    }
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::NotFound { resource, lookup } => write!(f, "{resource} {lookup} not found"),
            CacheError::ResourceNotFound(resource) => write!(f, "{resource} not found"),
            CacheError::KeyNotFound(_) => write!(f, "Cache key not found"),
        }
    }
}

impl std::error::Error for CacheError {}

#[cfg(test)]
mod tests {
    use super::CacheError;

    #[test]
    fn test_cache_error_display_not_found() {
        assert_eq!(CacheError::not_found("FiatQuote", "abc").to_string(), "FiatQuote abc not found");
    }

    #[test]
    fn test_cache_error_display_resource_not_found() {
        assert_eq!(CacheError::not_found_resource("FiatRates").to_string(), "FiatRates not found");
    }

    #[test]
    fn test_cache_error_display_key_not_found_does_not_expose_key() {
        assert_eq!(CacheError::KeyNotFound("fiat:quote:abc".to_string()).to_string(), "Cache key not found");
    }
}
