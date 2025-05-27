#[cfg(test)]
mod tests {
    use name_resolver::{base::Basenames, client::NameClient, ens::ENSClient, hyperliquid::HLNamesClient, injective::InjectiveNameClient, suins::SuinsClient};
    use primitives::{Chain, node_config::get_nodes_for_chain};
    use tokio_test::block_on;

    #[test]
    fn test_resolver_eth() {
        // this test is ignored from UT cause it connects to the real network
        let nodes = get_nodes_for_chain(Chain::Ethereum);
        let client = ENSClient::new(nodes[0].url.clone());
        block_on(async {
            let address = client.resolve("vitalik.eth", Chain::Ethereum).await;

            assert_eq!(address.unwrap(), "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
        });
    }

    #[test]
    fn test_resolve_basenames() {
        let nodes = get_nodes_for_chain(Chain::Base);
        let client = Basenames::new(nodes[0].url.clone());
        block_on(async {
            let address = client.resolve("h3rman.base.eth", Chain::Base).await.unwrap();

            assert_eq!(address.to_lowercase(), "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_lowercase())
        });
    }

    #[test]
    fn test_resolve_injective() {
        let nodes = get_nodes_for_chain(Chain::Injective);
        let client = InjectiveNameClient::new(nodes[0].url.clone());
        block_on(async {
            let address_result = client.resolve("test.inj", Chain::Injective).await;

            assert_eq!(address_result.unwrap(), "inj14apqz6u2nprsly3j0mqa6jwpxnmnphq3pp0q9g");
        });
    }

    #[test]
    fn test_resolve_suins() {
        let nodes = get_nodes_for_chain(Chain::Sui);
        let client = SuinsClient::new(nodes[0].url.clone());
        block_on(async {
            let address_result = client.resolve("test.sui", Chain::Sui).await;

            assert_eq!(address_result.unwrap(), "0x3e04ea76cee7d2db4f41c2972ac8d929606d89f7293320f0886abb41a578190c");
        });
    }

    #[test]
    fn test_resolve_hlnames() {
        block_on(async {
            let client = HLNamesClient::default();
            let address = client.resolve("testooor.hl", Chain::Hyperliquid).await.unwrap();

            assert_eq!(address, "0xF26F5551E96aE5162509B25925fFfa7F07B2D652");
        });
    }
}
