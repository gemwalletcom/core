use serde::Serialize;

/// Build a path with query parameters from a serializable struct
pub fn build_path_with_query<T: Serialize>(path: &str, query: &T) -> Result<String, serde_urlencoded::ser::Error> {
    let query_string = serde_urlencoded::to_string(query)?;
    Ok(format!("{}?{}", path, query_string))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct CoinQuery {
        pub market_data: bool,
        pub community_data: bool,
        pub tickers: bool,
        pub localization: bool,
        pub developer_data: bool,
    }

    #[test]
    fn test_build_path_with_query_coingecko_case() {
        let id = "bitcoin";
        let query = CoinQuery {
            market_data: false,
            community_data: true,
            tickers: false,
            localization: true,
            developer_data: true,
        };
        let base_path = format!("/api/v3/coins/{}", id);
        let result = build_path_with_query(&base_path, &query).unwrap();

        let expected = "/api/v3/coins/bitcoin?market_data=false&community_data=true&tickers=false&localization=true&developer_data=true";
        assert_eq!(result, expected);
    }
}
