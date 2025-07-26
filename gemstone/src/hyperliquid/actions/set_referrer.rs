#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperSetReferrer {
    #[serde(rename = "type")]
    pub action_type: String,
    pub code: String,
}

impl HyperSetReferrer {
    pub fn new(code: String) -> Self {
        Self {
            action_type: "setReferrer".to_string(),
            code,
        }
    }
}
