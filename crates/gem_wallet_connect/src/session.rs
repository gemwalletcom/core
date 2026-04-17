use std::collections::HashMap;
use std::str::FromStr;

use gem_ton::signer::wallet_state_init_base64_from_public_key;
use primitives::{Account, Chain};

const TON_PUBLIC_KEY_KEY: &str = "ton_getPublicKey";
const TON_STATE_INIT_KEY: &str = "ton_getStateInit";
const TRON_METHOD_VERSION_KEY: &str = "tron_method_version";
const TRON_METHOD_VERSION_VALUE: &str = "v1";

pub fn config_session_properties(mut properties: HashMap<String, String>, chains: &[Chain], accounts: &[Account]) -> HashMap<String, String> {
    if chains.contains(&Chain::Ton) {
        if !properties.contains_key(TON_PUBLIC_KEY_KEY)
            && let Some(public_key) = accounts
                .iter()
                .find(|account| account.chain == Chain::Ton)
                .and_then(|account| account.public_key.as_ref().filter(|value| decode_public_key(value).is_some()).cloned())
        {
            properties.insert(TON_PUBLIC_KEY_KEY.to_string(), public_key);
        }
        if !properties.contains_key(TON_STATE_INIT_KEY)
            && let Some(public_key) = properties.get(TON_PUBLIC_KEY_KEY).and_then(|value| decode_public_key(value))
            && let Ok(state_init) = wallet_state_init_base64_from_public_key(public_key)
        {
            properties.insert(TON_STATE_INIT_KEY.to_string(), state_init);
        }
    }
    if chains.contains(&Chain::Tron) && !properties.contains_key(TRON_METHOD_VERSION_KEY) {
        properties.insert(TRON_METHOD_VERSION_KEY.to_string(), TRON_METHOD_VERSION_VALUE.to_string());
    }
    properties
}

pub fn parse_chains(chains: &[String]) -> Vec<Chain> {
    chains.iter().filter_map(|c| Chain::from_str(c).ok()).collect()
}

fn decode_public_key(value: &str) -> Option<[u8; 32]> {
    let value = value.strip_prefix("0x").unwrap_or(value);
    let bytes = hex::decode(value).ok()?;
    bytes.try_into().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TON_PUBLIC_KEY: &str = "d369452197c2a56481e5e2d3e8bf03de2349f67a63151956822208c2334adee2";

    #[test]
    fn test_config_session_properties_tron() {
        let result = config_session_properties(HashMap::new(), &[Chain::Tron], &[]);
        assert_eq!(result.get("tron_method_version").unwrap(), "v1");
        let mut props = HashMap::new();
        props.insert("tron_method_version".to_string(), "v2".to_string());
        let result = config_session_properties(props, &[Chain::Tron], &[]);
        assert_eq!(result.get("tron_method_version").unwrap(), "v2");
        let result = config_session_properties(HashMap::new(), &[Chain::Ethereum], &[]);
        assert!(!result.contains_key("tron_method_version"));
    }

    #[test]
    fn test_config_session_properties_ton() {
        let accounts = [Account {
            chain: Chain::Ton,
            address: "EQCEX-MyMiEhdrqxDQ5zFfDIuJ2l8wtsNxkhp4-PNxiL06UX".to_string(),
            derivation_path: "m/44'/607'/0'".to_string(),
            public_key: Some(TON_PUBLIC_KEY.to_string()),
            extended_public_key: None,
        }];
        let result = config_session_properties(HashMap::new(), &[Chain::Ton], &accounts);
        assert_eq!(result.get("ton_getPublicKey").unwrap(), TON_PUBLIC_KEY);
        assert!(result.get("ton_getStateInit").unwrap().starts_with("te6cc"));
        let mut properties = HashMap::new();
        properties.insert("ton_getPublicKey".to_string(), TON_PUBLIC_KEY.to_string());
        properties.insert("ton_getStateInit".to_string(), "existing".to_string());
        let result = config_session_properties(properties, &[Chain::Ton], &accounts);
        assert_eq!(result.get("ton_getStateInit").unwrap(), "existing");
        let result = config_session_properties(HashMap::new(), &[Chain::Ton], &[]);
        assert!(!result.contains_key("ton_getPublicKey"));
        assert!(!result.contains_key("ton_getStateInit"));
    }
}
