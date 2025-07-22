#[cfg(test)]
mod tests {
    use fiat::providers::MoonPayClient;
    use primitives::FiatProviderName;

    #[test]
    fn test_client_name() {
        assert!(matches!(MoonPayClient::NAME, FiatProviderName::MoonPay));
    }

    // Add more MoonPay tests here
}
