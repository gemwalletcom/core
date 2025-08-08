#[derive(uniffi::Record, serde::Serialize)]
pub struct HyperSetReferrer {
    pub r#type: String,
    pub code: String,
}

impl HyperSetReferrer {
    pub fn new(code: String) -> Self {
        Self {
            r#type: "setReferrer".to_string(),
            code,
        }
    }
}
