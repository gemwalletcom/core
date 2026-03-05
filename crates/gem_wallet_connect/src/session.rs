use std::collections::HashMap;
use std::str::FromStr;

use primitives::Chain;

const TRON_METHOD_VERSION_KEY: &str = "tron_method_version";
const TRON_METHOD_VERSION_VALUE: &str = "v1";

pub fn config_session_properties(mut properties: HashMap<String, String>, chains: &[Chain]) -> HashMap<String, String> {
    if chains.contains(&Chain::Tron) && !properties.contains_key(TRON_METHOD_VERSION_KEY) {
        properties.insert(TRON_METHOD_VERSION_KEY.to_string(), TRON_METHOD_VERSION_VALUE.to_string());
    }
    properties
}

pub fn parse_chains(chains: &[String]) -> Vec<Chain> {
    chains.iter().filter_map(|c| Chain::from_str(c).ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_session_properties_adds_tron() {
        let result = config_session_properties(HashMap::new(), &[Chain::Tron]);
        assert_eq!(result.get("tron_method_version").unwrap(), "v1");
    }

    #[test]
    fn test_config_session_properties_preserves_existing() {
        let mut props = HashMap::new();
        props.insert("tron_method_version".to_string(), "v2".to_string());
        let result = config_session_properties(props, &[Chain::Tron]);
        assert_eq!(result.get("tron_method_version").unwrap(), "v2");
    }

    #[test]
    fn test_config_session_properties_no_tron() {
        let result = config_session_properties(HashMap::new(), &[Chain::Ethereum]);
        assert!(!result.contains_key("tron_method_version"));
    }
}
