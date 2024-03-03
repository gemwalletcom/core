use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};

#[repr(u32)]
#[derive(uniffi::Enum, Debug, Clone, PartialEq, Eq, AsRefStr, EnumString)]
#[strum(serialize_all = "lowercase")]
#[non_exhaustive]
pub enum Chain {
    Bitcoin,
    Ethereum,
    Solana,
}

#[derive(Debug, Clone, uniffi::Object)]
pub struct ChainWrapper {
    pub value: Chain,
}

#[uniffi::export]
impl ChainWrapper {
    #[uniffi::constructor]
    pub fn new(value: Chain) -> Self {
        Self { value }
    }

    #[uniffi::constructor]
    pub fn new_str(value: String) -> Self {
        Self {
            value: Chain::from_str(&value).unwrap(),
        }
    }

    pub fn value(&self) -> Chain {
        self.value.clone()
    }

    pub fn string(&self) -> String {
        self.value.as_ref().to_string()
    }
}

mod tests {
    #[test]
    fn test_chain() {
        use crate::chain::*;
        let chain = ChainWrapper::new_str("ethereum".to_string());
        assert_eq!(chain.value(), Chain::Ethereum);
    }
}
