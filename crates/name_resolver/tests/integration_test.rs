#[cfg(test)]
mod tests {
    use name_resolver::base::BNSClient;
    use name_resolver::client::NameClient;
    use name_resolver::ens_provider::provider::Provider;
    use primitives::Chain;
    use tokio_test::block_on;

    #[test]
    fn test_resolver_eth() {
        block_on(async {
            let provider = Provider::new(String::from("https://eth.llamarpc.com"));
            let addres = provider.resolve_name("vitalik.eth", Chain::Ethereum).await;
            assert_eq!(
                addres.unwrap(),
                "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_lowercase()
            )
        });
    }

    #[test]
    fn test_resolve_base() {
        block_on(async {
            let client = BNSClient::new(String::from("https://resolver-api.basename.app"));
            let addres = client.resolve("hello.base", Chain::Base).await;
            assert_eq!(
                addres.unwrap(),
                "0x4fb3f133951bF1B2d52fF6CEab2c703fbB6E98cC"
            )
        });
    }
}
