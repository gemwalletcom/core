use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

pub const DEEP_LINK_SCHEME: &str = "gem://";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, AsRefStr, EnumString)]
#[strum(serialize_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub enum Deeplink {
    Rewards,
}

impl Deeplink {
    pub fn to_url(&self) -> String {
        format!("{}{}", DEEP_LINK_SCHEME, self.as_ref())
    }
}
