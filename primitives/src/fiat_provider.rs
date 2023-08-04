
use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable")]
#[serde(rename_all = "camelCase")]
pub struct FiatProvider {
    pub name: String,
    pub image_url: String,
}