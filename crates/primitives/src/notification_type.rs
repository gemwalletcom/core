use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, AsRefStr, EnumString)]
#[strum(serialize_all = "camelCase")]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Equatable")]
pub enum NotificationType {
    ReferralJoined,
    RewardsEnabled,
    RewardsCodeDisabled,
    RewardsRedeemed,
    RewardsCreateUsername,
    RewardsInvite,
}
