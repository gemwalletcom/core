#[cfg(test)]
mod tests {
    use fiat::providers::TransakClient;
    use primitives::FiatProviderName;

    #[test]
    fn test_client_name() {
        assert!(matches!(TransakClient::NAME, FiatProviderName::Transak));
    }

    // Add more Transak tests here
}
