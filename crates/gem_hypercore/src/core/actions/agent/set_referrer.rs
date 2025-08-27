// IMPORTANT: Field order matters for msgpack serialization and hash calculation
// Do not change field order unless you know the exact order in Python SDK.

#[derive(Clone, serde::Serialize)]
pub struct SetReferrer {
    pub r#type: String,
    pub code: String,
}

impl SetReferrer {
    pub fn new(code: String) -> Self {
        Self {
            r#type: "setReferrer".to_string(),
            code,
        }
    }
}
