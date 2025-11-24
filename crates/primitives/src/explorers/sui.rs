use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{ACCOUNT_PATH, COIN_PATH, Explorer, Metadata, TX_PATH, VALIDATOR_PATH};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sui_scan_urls() {
        let explorer = new_sui_scan();

        assert_eq!(
            explorer.get_token_url("token123"),
            Some("https://suiscan.xyz/mainnet/coin/token123".to_string())
        );

        assert_eq!(
            explorer.get_validator_url("val123"),
            Some("https://suiscan.xyz/mainnet/validator/val123".to_string())
        );
    }
}
