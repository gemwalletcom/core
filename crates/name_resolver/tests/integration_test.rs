#[cfg(test)]
mod tests {
    use name_resolver::{basenames::Basenames, client::NameClient, ens::ENSClient, injective::InjectiveNameClient, suins::SuinsClient};
    use primitives::Chain;
    use tokio_test::block_on;

    #[test]
    fn test_resolver_eth() {
        // this test is ignored from UT cause it connects to the real network
        block_on(async {
            let client = ENSClient::new(String::from("https://eth.llamarpc.com"));
            let address = client.resolve("vitalik.eth", Chain::Ethereum).await;

            assert_eq!(address.unwrap(), "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
        });
    }

    #[test]
    fn test_resolve_basenames() {
        // This test connects to the real Base network
        block_on(async {
            let client = Basenames::new(String::from("https://mainnet.base.org"));
            let address = client.resolve("h3rman.base.eth", Chain::Base).await.unwrap();

            assert_eq!(address.to_lowercase(), "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_lowercase())
        });
    }

    #[test]
    fn test_resolve_injective() {
        // This test connects to the real Injective network
        block_on(async {
            let client = InjectiveNameClient::new(String::from("https://sentry.lcd.injective.network/"));
            let address_result = client.resolve("test.inj", Chain::Injective).await;

            assert_eq!(address_result.unwrap(), "inj14apqz6u2nprsly3j0mqa6jwpxnmnphq3pp0q9g");
        });
    }

    #[test]
    fn test_resolve_suins() {
        // This test connects to the real Sui network
        block_on(async {
            let client = SuinsClient::new(String::from("https://fullnode.mainnet.sui.io:443"));
            let address_result = client.resolve("test.sui", Chain::Sui).await;

            assert_eq!(address_result.unwrap(), "0x3e04ea76cee7d2db4f41c2972ac8d929606d89f7293320f0886abb41a578190c");
        });
    }
}
