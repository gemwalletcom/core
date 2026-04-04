use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{ACCOUNT_PATH, ASSET_PATH, Explorer, Metadata, TX_PATH};

pub struct AlgorandAllo;
pub struct AlgorandPera;

impl AlgorandAllo {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Allo",
            base_url: "https://allo.info",
            tx_path: TX_PATH,
            address_path: ACCOUNT_PATH,
            token_path: None,
            nft_path: None,
            validator_path: Some(ACCOUNT_PATH),
        })
    }
}

impl AlgorandPera {
    pub fn boxed() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "Pera Explorer",
            base_url: "https://explorer.perawallet.app",
            tx_path: TX_PATH,
            address_path: ACCOUNT_PATH,
            token_path: Some(ASSET_PATH),
            nft_path: None,
            validator_path: Some(ACCOUNT_PATH),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pera_tx_url() {
        let explorer = AlgorandPera::boxed();
        assert_eq!(
            explorer.get_tx_url("NWMRGT6MV5ZQMEZBFYAS6NE6YXSSZDHMXG3OKJOSSOK6C2OCPJ4Q"),
            "https://explorer.perawallet.app/tx/NWMRGT6MV5ZQMEZBFYAS6NE6YXSSZDHMXG3OKJOSSOK6C2OCPJ4Q"
        );
    }

    #[test]
    fn test_pera_token_url() {
        let explorer = AlgorandPera::boxed();
        assert_eq!(explorer.get_token_url("13379146"), Some("https://explorer.perawallet.app/asset/13379146".to_string()));
    }
}
