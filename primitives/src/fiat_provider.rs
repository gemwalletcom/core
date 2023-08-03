#[typeshare(swift = "Equatable, Codable")]
#[serde(rename_all = "camelCase")]
pub struct FiatProvider {
    pub name: String,
    pub image_url: String,
}