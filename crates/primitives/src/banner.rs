#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Banner {
    wallet: Option<Wallet>,
    asset: Option<Asset>,
    chain: Option<Chain>,
    event: BannerEvent,
    state: BannerState,
}

#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum BannerEvent {
    Stake,
    AccountActivation,
    EnableNotifications,
    AccountBlockedMultiSignature,
    ActivateAsset,
    SuspiciousAsset,
    Onboarding,
}

#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum BannerState {
    Active,
    Cancelled,
    AlwaysActive,
}
