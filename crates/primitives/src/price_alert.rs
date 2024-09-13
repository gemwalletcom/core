use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PriceAlertSubsription {
    pub asset_id: String,
}

pub type PriceAlertSubsriptions = Vec<PriceAlertSubsription>;
