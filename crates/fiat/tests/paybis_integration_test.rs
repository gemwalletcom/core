#[cfg(test)]
mod tests {
    use fiat::providers::PaybisClient;
    use primitives::FiatProviderName;

    #[test]
    fn test_map_asset() {
        use fiat::providers::paybis::model::Currency;

        let btc_currency = Currency {
            code: "BTC".to_string(),
            blockchain_name: Some("bitcoin".to_string()),
        };

        let asset = PaybisClient::map_asset(btc_currency);
        assert!(asset.is_some());

        let asset = asset.unwrap();
        assert_eq!(asset.id, "BTC");
        assert_eq!(asset.symbol, "BTC");
        assert_eq!(asset.chain, Some(primitives::Chain::Bitcoin));
        assert!(asset.enabled);

        let fiat_currency = Currency {
            code: "USD".to_string(),
            blockchain_name: None,
        };

        let asset = PaybisClient::map_asset(fiat_currency);
        assert!(asset.is_none());
    }

    #[test]
    fn test_client_name() {
        assert!(matches!(PaybisClient::NAME, FiatProviderName::Paybis));
    }
}
