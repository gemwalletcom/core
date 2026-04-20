pub fn validate_network(network: Option<&str>, expected: &str) -> Result<(), String> {
    let Some(network) = network.filter(|network| !network.is_empty()) else {
        return Ok(());
    };
    if network != expected {
        return Err(format!("Network mismatch: TON network {} does not match wallet network {}", network, expected));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_network() {
        assert!(validate_network(None, "-239").is_ok());
        assert!(validate_network(Some(""), "-239").is_ok());
        assert!(validate_network(Some("-239"), "-239").is_ok());
        assert_eq!(
            validate_network(Some("-3"), "-239").unwrap_err(),
            "Network mismatch: TON network -3 does not match wallet network -239"
        );
    }
}
