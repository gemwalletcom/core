use crate::SUI_COIN_TYPE;
use crate::models::{Coin, Object, OwnedCoins};

impl Object {
    pub fn mock() -> Self {
        Self {
            object_id: "0xabcdef1234567890abcdef1234567890abcdef12".parse().unwrap(),
            digest: "HdfF7hswRuvbXbEXjGjmUCt7gLybhvbPvvK8zZbCqyD8".parse().unwrap(),
            version: 100,
        }
    }
}

impl Coin {
    pub fn mock_sui() -> Self {
        Self {
            coin_type: SUI_COIN_TYPE.to_string(),
            balance: 5_000_000_000,
            object: Object::mock(),
        }
    }
}

impl OwnedCoins<Coin> {
    pub fn mock_sui() -> Self {
        Self::new(SUI_COIN_TYPE.to_string(), vec![Coin::mock_sui()], 0)
    }
}
