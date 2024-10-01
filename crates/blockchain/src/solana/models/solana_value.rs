#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
pub struct SolanaValue<T> {
    pub value: T,
}
