#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Banner {
    wallet: Option<Wallet>,
    asset: Option<Asset>,
    event: BannerEvent,
    state: BannerState,
}

#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum BannerEvent {
    stake,
    account_activation,
    enable_notifications,
}

#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum BannerState {
    active,
    cancelled,
    always_active,
}
