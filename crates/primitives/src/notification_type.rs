use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, AsRefStr, EnumString)]
#[strum(serialize_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub enum NotificationType {
    ReferralJoined,
    RewardsEnabled,
    RewardsCodeDisabled,
    RewardsRedeemed,
    RewardsCreateUsername,
    RewardsInvite,
}
