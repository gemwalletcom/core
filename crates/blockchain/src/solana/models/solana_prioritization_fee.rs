#[typeshare(swift = "Sendable")]
struct SolanaPrioritizationFee {
    #[serde(rename = "prioritizationFee")]
    prioritization_fee: i32,
}
