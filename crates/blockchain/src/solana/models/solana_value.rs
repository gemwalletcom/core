#[typeshare(swift = "Sendable")]
pub struct SolanaValue<T> {
    pub value: T,
}
