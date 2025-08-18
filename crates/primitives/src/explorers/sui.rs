use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, TX_PATH, ACCOUNT_PATH, COIN_PATH, VALIDATOR_PATH, VALIDATORS_PATH};

pub fn new_sui_scan() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata {
        name: "SuiScan",
        base_url: "https://suiscan.xyz/mainnet",
        tx_path: TX_PATH,
        address_path: ACCOUNT_PATH,
        token_path: Some(COIN_PATH),
        validator_path: Some(VALIDATOR_PATH),
    })
}

pub fn new_sui_vision() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata {
        name: "SuiVision",
        base_url: "https://suivision.xyz",
        tx_path: "/txblock",
        address_path: ACCOUNT_PATH,
        token_path: Some(COIN_PATH),
        validator_path: Some(VALIDATORS_PATH),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_token_url() {
        let sui_scan = new_sui_scan();
        let sui_vision = new_sui_vision();

        assert_eq!(sui_scan.get_token_url("token123"), Some("https://suiscan.xyz/mainnet/coin/token123".to_string()));
        assert_eq!(sui_vision.get_token_url("token123"), Some("https://suivision.xyz/coin/token123".to_string()));

        assert_eq!(sui_scan.get_validator_url("val123"), Some("https://suiscan.xyz/mainnet/validator/val123".to_string()));
        assert_eq!(sui_vision.get_validator_url("val123"), Some("https://suivision.xyz/validators/val123".to_string()));
    }
}