use serde::Deserialize;
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub id: String,
    pub external_order_id: Option<String>,
    pub status: String,
    pub fiat: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub fiat_amount: f64,
    pub transaction_hash: Option<String>,
    pub order_type: String,
}
