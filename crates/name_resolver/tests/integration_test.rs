#[cfg(test)]
mod tests {
    use name_resolver::base::BNSClient;
    use name_resolver::client::NameClient;
    use name_resolver::ens_provider::provider::Provider;
    use primitives::Chain;
    use tokio_test::block_on;

    #[test]
    fn test_resolver_eth() {
        // this test is ignored from UT cause it connects to the real network
        block_on(async {
            let provider = Provider::new(String::from("https://eth.llamarpc.com"));
            let address = provider.resolve_name("vitalik.eth", Chain::Ethereum).await;
            assert_eq!(address.unwrap(), "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_lowercase())
        });
    }

    #[test]
    fn test_resolve_base() {
        block_on(async {
            let client = BNSClient::new(String::from("https://resolver-api.basename.app"));
            let address = client.resolve("hello.base", Chain::Base).await;
            assert_eq!(address.unwrap(), "0x4fb3f133951bF1B2d52fF6CEab2c703fbB6E98cC")
        });
    }
}
