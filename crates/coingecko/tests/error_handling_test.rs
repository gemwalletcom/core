use coingecko::CoinGeckoResponse;

#[test]
fn test_parse_api_error_response() {
    let error_json = r#"{"error": "coin not found"}"#;
    let response: CoinGeckoResponse<()> = serde_json::from_str(error_json).unwrap();

    match response {
        CoinGeckoResponse::Error(error) => {
            assert_eq!(error.error, "coin not found");
        }
        CoinGeckoResponse::Success(_) => {
            panic!("Expected error response, got success");
        }
    }
}
