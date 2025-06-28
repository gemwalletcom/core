#[derive(serde::Deserialize, serde::Serialize)]
#[typeshare::typeshare(swift = "Sendable")]
#[typeshare::typeshare(swiftGenericConstraints = "T: Sendable")]
pub struct XRPResult<T> {
    pub result: T,
}
