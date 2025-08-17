use std::error::Error;

pub fn map_transaction_broadcast(hash: String) -> Result<String, Box<dyn Error + Sync + Send>> {
    if hash.is_empty() {
        Err("Empty transaction hash".into())
    } else {
        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_transaction_broadcast() {
        let hash = "test_hash_123".to_string();
        let result = map_transaction_broadcast(hash).unwrap();
        assert_eq!(result, "test_hash_123");
    }
}
