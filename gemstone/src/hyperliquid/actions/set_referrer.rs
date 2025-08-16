// IMPORTANT: Field order matters for msgpack serialization and hash calculation
// Do not change field order unless you know the exact order in Python SDK.

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
