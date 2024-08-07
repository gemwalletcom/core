#[typeshare(swift = "Equatable, Codable")]
#[serde(rename_all = "camelCase")]
pub struct Banner {
    wallet: Option<Wallet>,
    asset: Option<Asset>,
    event: BannerEvent,
    state: BannerState,
}

#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "camelCase")]
pub enum BannerEvent {
    stake,
    account_activation,
}

#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "camelCase")]
pub enum BannerState {
    active,
    cancelled,
    always_active,
}
