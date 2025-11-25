use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{ACCOUNT_PATH, COIN_PATH, Explorer, Metadata, VALIDATORS_PATH};

pub struct BlockVision;

impl BlockVision {
    pub fn new_monad() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata::full("MonadVision", "https://monadvision.com"))
    }

    pub fn new_sui() -> Box<dyn BlockExplorer> {
        Explorer::boxed(Metadata {
            name: "SuiVision",
            base_url: "https://suivision.xyz",
            tx_path: "/txblock",
            address_path: ACCOUNT_PATH,
            token_path: Some(COIN_PATH),
            validator_path: Some(VALIDATORS_PATH),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monad_urls() {
        let explorer = BlockVision::new_monad();

        assert_eq!(explorer.name(), "MonadVision");
        assert_eq!(explorer.get_address_url("0xabc"), "https://monadvision.com/address/0xabc");
        assert_eq!(explorer.get_tx_url("0x123"), "https://monadvision.com/tx/0x123");
        assert_eq!(
            explorer.get_token_url("0x754704Bc059F8C67012fEd69BC8A327a5aafb603"),
            Some("https://monadvision.com/token/0x754704Bc059F8C67012fEd69BC8A327a5aafb603".to_string())
        );
        assert_eq!(
            explorer.get_validator_url("0xC11Ae71884A76744Fa7976e09AC5441F1233Ef6F"),
            Some("https://monadvision.com/validator/0xC11Ae71884A76744Fa7976e09AC5441F1233Ef6F".to_string())
        );
    }

    #[test]
    fn test_sui_vision_urls() {
        let explorer = BlockVision::new_sui();

        assert_eq!(explorer.name(), "SuiVision");
        assert_eq!(
            explorer.get_address_url("0x6f02af629f66a13c5b8cb857cddf43804422d205b0bb9bda9db98b2635fe59bb"),
            "https://suivision.xyz/account/0x6f02af629f66a13c5b8cb857cddf43804422d205b0bb9bda9db98b2635fe59bb"
        );
        assert_eq!(
            explorer.get_tx_url("ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos"),
            "https://suivision.xyz/txblock/ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos"
        );
        assert_eq!(explorer.get_token_url("token123"), Some("https://suivision.xyz/coin/token123".to_string()));
        assert_eq!(
            explorer.get_validator_url("val123"),
            Some("https://suivision.xyz/validators/val123".to_string())
        );
    }
}
