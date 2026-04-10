use super::StellarAssetData;
use crate::address::StellarAddress;

#[derive(Clone)]
pub enum Memo {
    None,
    Text(String),
    #[cfg_attr(not(test), allow(unused))]
    Id(u64),
}

#[derive(Clone)]
pub enum Operation {
    CreateAccount {
        destination: StellarAddress,
        amount: u64,
    },
    Payment {
        destination: StellarAddress,
        asset: Option<StellarAssetData>,
        amount: u64,
    },
    ChangeTrust {
        asset: StellarAssetData,
    },
}

impl Operation {
    pub fn operation_type(&self) -> u32 {
        match self {
            Self::CreateAccount { .. } => 0,
            Self::Payment { .. } => 1,
            Self::ChangeTrust { .. } => 6,
        }
    }
}
