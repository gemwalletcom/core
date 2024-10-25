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
    stake,
    account_activation,
    enable_notifications,
    account_blocked_multisignature,
}

#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum BannerState {
    active,
    cancelled,
    always_active,
}
