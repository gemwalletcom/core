#[cfg(test)]
mod tests {
    use std::env;
    use std::time::Duration;

    use primitives::Chain;
    use security_provider::{AddressTarget, ScanProvider, TokenTarget, providers::goplus::GoPlusProvider, providers::hashdit::HashDitProvider};
    use settings::Settings;

    use gem_client::ReqwestClient;

    fn load_settings() -> Settings {
        let current_dir = env::current_dir().unwrap();
        let path = current_dir.join("../../Settings.yaml");
        Settings::new_setting_path(path).unwrap()
    }

    fn build_client(base_url: String, timeout_ms: u64) -> ReqwestClient {
        let timeout = Duration::from_millis(timeout_ms);
        let http = reqwest::Client::builder().timeout(timeout).build().expect("failed to build reqwest client");
        ReqwestClient::new(base_url, http)
    }

    #[tokio::test]
    async fn test_goplus_scan_address_eth() {
        let settings = load_settings();
        let client = build_client(settings.scan.goplus.url.clone(), settings.scan.timeout_ms);
        let provider = GoPlusProvider::new(client, &settings.scan.goplus.key.secret);

        let target = AddressTarget {
            address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(), // Vitalik.eth
            chain: Chain::Ethereum,
        };

        let result = provider.scan_address(&target).await.expect("goplus address scan failed");

        assert_eq!(result.provider, "GoPlus");
        assert_eq!(result.is_malicious, false);
    }

    #[tokio::test]
    async fn test_goplus_scan_token_eth_usdc() {
        let settings = load_settings();
        let client = build_client(settings.scan.goplus.url.clone(), settings.scan.timeout_ms);
        let provider = GoPlusProvider::new(client, &settings.scan.goplus.key.secret);

        let target = TokenTarget {
            token_id: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(), // USDT
            chain: Chain::Ethereum,
        };

        let result = provider.scan_token(&target).await.expect("goplus token scan failed");

        assert_eq!(result.provider, "GoPlus");
        assert_eq!(result.is_malicious, false);
    }

    #[tokio::test]
    async fn test_hashdit_scan_address_eth() {
        let settings = load_settings();
        let client = build_client(settings.scan.hashdit.url.clone(), settings.scan.timeout_ms);
        let app_id = &settings.scan.hashdit.key.public;
        let app_secret = &settings.scan.hashdit.key.secret;
        let provider = HashDitProvider::new(client, app_id, app_secret);

        let target = AddressTarget {
            address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(), // Vitalik.eth
            chain: Chain::Ethereum,
        };

        let result = provider.scan_address(&target).await.expect("hashdit address scan failed");

        assert_eq!(result.provider, "HashDit");
        assert_eq!(result.is_malicious, false);
    }

    #[tokio::test]
    async fn test_hashdit_scan_token_eth_usdc() {
        let settings = load_settings();
        let client = build_client(settings.scan.hashdit.url.clone(), settings.scan.timeout_ms);
        let app_id = &settings.scan.hashdit.key.public;
        let app_secret = &settings.scan.hashdit.key.secret;
        let provider = HashDitProvider::new(client, app_id, app_secret);

        let target = TokenTarget {
            token_id: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(), // USDC
            chain: Chain::Ethereum,
        };

        let result = provider.scan_token(&target).await.expect("hashdit token scan failed");

        assert_eq!(result.provider, "HashDit");
        assert_eq!(result.is_malicious, false);
    }
}
