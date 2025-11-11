use super::SearchRequest;

pub fn build_assets_filters(request: &SearchRequest) -> Vec<String> {
    let mut filters = vec![];
    filters.push(format!("score.rank > {}", request.rank_threshold()));

    if !request.tags.is_empty() {
        filters.push(filter_array("tags", request.tags.clone()));
    }

    if !request.chains.is_empty() {
        filters.push(filter_array("asset.chain", request.chains.clone()));
    }

    filters
}

pub fn build_filter(filters: Vec<String>) -> String {
    filters.join(" AND ")
}

fn filter_array(field: &str, values: Vec<String>) -> String {
    format!("{} IN [\"{}\"]", field, values.join("\",\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_assets_filters_short_query() {
        let request = SearchRequest::new("USDT".to_string(), None, None, None, None);
        let filters = build_assets_filters(&request);

        assert_eq!(filters[0], "score.rank > 15");
    }

    #[test]
    fn build_assets_filters_long_query() {
        let request = SearchRequest::new("ethereum".to_string(), None, None, None, None);
        let filters = build_assets_filters(&request);

        assert_eq!(filters[0], "score.rank > 0");
    }

    #[test]
    fn build_assets_filters_with_tags() {
        let request = SearchRequest::new("longquery".to_string(), None, Some("defi".to_string()), None, None);
        let filters = build_assets_filters(&request);

        assert_eq!(filters[0], "score.rank > 0");
        assert_eq!(filters[1], "tags IN [\"defi\"]");
    }

    #[test]
    fn build_assets_filters_with_chains() {
        let request = SearchRequest::new("longquery".to_string(), Some("ethereum".to_string()), None, None, None);
        let filters = build_assets_filters(&request);

        assert_eq!(filters[0], "score.rank > 0");
        assert_eq!(filters[1], "asset.chain IN [\"ethereum\"]");
    }

    #[test]
    fn build_filter_joins_with_and() {
        assert_eq!(build_filter(vec!["a".to_string(), "b".to_string()]), "a AND b");
    }

    #[test]
    fn filter_array_formats_correctly() {
        assert_eq!(filter_array("tags", vec!["defi".to_string(), "nft".to_string()]), "tags IN [\"defi\",\"nft\"]");
    }
}
