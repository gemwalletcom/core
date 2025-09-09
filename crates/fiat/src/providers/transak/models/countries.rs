use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Country {
    pub alpha2: String,
    pub is_allowed: bool,
}
