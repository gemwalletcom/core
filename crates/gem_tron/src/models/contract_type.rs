use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use strum::{Display, EnumString};

#[cfg(feature = "signer")]
const TYPE_URL_PREFIX: &str = "type.googleapis.com/protocol.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[repr(u64)]
pub enum TronContractType {
    #[strum(serialize = "TransferContract")]
    Transfer = 1,
    #[strum(serialize = "TransferAssetContract")]
    TransferAsset = 2,
    #[strum(serialize = "VoteWitnessContract")]
    VoteWitness = 4,
    #[strum(serialize = "WithdrawBalanceContract")]
    WithdrawBalance = 13,
    #[strum(serialize = "TriggerSmartContract")]
    TriggerSmart = 31,
    #[strum(serialize = "FreezeBalanceV2Contract")]
    FreezeBalanceV2 = 54,
    #[strum(serialize = "UnfreezeBalanceV2Contract")]
    UnfreezeBalanceV2 = 55,
    #[strum(serialize = "WithdrawExpireUnfreezeContract")]
    WithdrawExpireUnfreeze = 56,
    #[strum(serialize = "DelegateResourceContract")]
    DelegateResource = 57,
    #[strum(serialize = "UnDelegateResourceContract")]
    UnDelegateResource = 58,
}

impl TronContractType {
    #[cfg(feature = "signer")]
    pub(crate) fn id(self) -> u64 {
        self as u64
    }

    #[cfg(feature = "signer")]
    pub(crate) fn type_url(self) -> String {
        format!("{TYPE_URL_PREFIX}{self}")
    }
}

impl Serialize for TronContractType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for TronContractType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(|_| D::Error::custom(format!("unsupported Tron contract type: {value}")))
    }
}
