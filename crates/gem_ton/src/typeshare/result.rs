#[derive(serde::Deserialize, serde::Serialize)]
#[typeshare::typeshare(swift = "Sendable")]
#[typeshare::typeshare(swiftGenericConstraints = "T: Sendable")]
pub struct TonResult<T> {
    pub result: T,
}
