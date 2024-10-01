#[typeshare(swift = "Sendable")]
struct AssetMetaData {
    #[serde(rename = "isEnabled")]
    is_enabled: bool,
    #[serde(rename = "isBuyEnabled")]
    is_buy_enabled: bool,
    #[serde(rename = "isSwapEnabled")]
    is_swap_enabled: bool,
    #[serde(rename = "isStakeEnabled")]
    is_stake_enabled: bool,
    #[serde(rename = "isPinned")]
    is_pinned: bool,
}
